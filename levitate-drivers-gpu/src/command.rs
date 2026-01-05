//! Async Command Traits for VirtIO GPU
//!
//! TEAM_098: Created as part of VirtIO GPU refactor.
//!
//! This module defines the async-first command handling per user requirement Q4:
//! "DO IT RIGHT FROM THE START!!! NO MORE SIMPLER IMPLEMENTATIONS!"

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use crate::protocol::{CtrlHeader, GpuError};

/// Trait for types that can be sent as GPU commands.
///
/// All command structs implement this trait to enable generic command handling.
pub trait GpuRequest: Sized {
    /// The expected response type for this command.
    type Response: GpuResponse;

    /// Get a reference to the command header.
    fn header(&self) -> &CtrlHeader;

    /// Get the command as a byte slice for transmission.
    ///
    /// # Safety
    ///
    /// The returned slice must be valid for the lifetime of self.
    fn as_bytes(&self) -> &[u8] {
        // SAFETY: repr(C) structs have predictable layout
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }

    /// Get the size of this command in bytes.
    fn size(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}

/// Trait for types that are received as GPU responses.
pub trait GpuResponse: Sized + Default {
    /// Get a mutable reference to the response header.
    fn header(&self) -> &CtrlHeader;

    /// Get the response as a mutable byte slice for receiving.
    ///
    /// # Safety
    ///
    /// The returned slice must be valid for the lifetime of self.
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        // SAFETY: repr(C) structs have predictable layout
        unsafe {
            core::slice::from_raw_parts_mut(
                (self as *mut Self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }

    /// Get the size of this response in bytes.
    fn size() -> usize {
        core::mem::size_of::<Self>()
    }

    /// Validate the response header.
    fn validate(&self) -> Result<(), GpuError> {
        let header = self.header();
        if header.is_err() {
            if let Some(ctrl_type) = header.ctrl_type() {
                return Err(GpuError::DeviceError(ctrl_type));
            }
        }
        Ok(())
    }
}

/// State of an in-flight command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandState {
    /// Command has not been sent yet.
    Pending,
    /// Command is in the virtqueue waiting for device.
    InFlight,
    /// Device has completed the command.
    Complete,
    /// Command failed with an error.
    Failed,
}

/// A pending command that can be polled for completion.
///
/// This is the core async primitive for GPU commands.
pub struct PendingCommand<R: GpuResponse> {
    state: CommandState,
    response: R,
    waker: Option<Waker>,
    descriptor_idx: u16,
}

impl<R: GpuResponse> PendingCommand<R> {
    /// Create a new pending command.
    pub fn new(descriptor_idx: u16) -> Self {
        Self {
            state: CommandState::InFlight,
            response: R::default(),
            waker: None,
            descriptor_idx,
        }
    }

    /// Get the descriptor index for this command.
    pub fn descriptor_idx(&self) -> u16 {
        self.descriptor_idx
    }

    /// Get the current state.
    pub fn state(&self) -> CommandState {
        self.state
    }

    /// Mark the command as complete with the given response data.
    pub fn complete(&mut self, response_data: &[u8]) {
        let dest = self.response.as_bytes_mut();
        let len = dest.len().min(response_data.len());
        dest[..len].copy_from_slice(&response_data[..len]);
        self.state = CommandState::Complete;

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    /// Mark the command as failed.
    pub fn fail(&mut self) {
        self.state = CommandState::Failed;

        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    /// Get the response (only valid after completion).
    pub fn response(&self) -> Option<&R> {
        if self.state == CommandState::Complete {
            Some(&self.response)
        } else {
            None
        }
    }

    /// Take the response (consumes the pending command).
    pub fn into_response(self) -> Result<R, GpuError> {
        match self.state {
            CommandState::Complete => {
                self.response.validate()?;
                Ok(self.response)
            }
            CommandState::Failed => Err(GpuError::TransportError),
            _ => Err(GpuError::NotInitialized),
        }
    }
}

/// Future for awaiting a GPU command response.
pub struct CommandFuture<'a, R: GpuResponse> {
    pending: &'a mut PendingCommand<R>,
}

impl<'a, R: GpuResponse> CommandFuture<'a, R> {
    /// Create a new command future.
    pub fn new(pending: &'a mut PendingCommand<R>) -> Self {
        Self { pending }
    }
}

impl<R: GpuResponse> Future for CommandFuture<'_, R> {
    type Output = Result<(), GpuError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.pending.state {
            CommandState::Complete => {
                self.pending.response.validate()?;
                Poll::Ready(Ok(()))
            }
            CommandState::Failed => Poll::Ready(Err(GpuError::TransportError)),
            CommandState::Pending | CommandState::InFlight => {
                self.pending.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// Simple response containing only a header (for OK_NODATA responses).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct HeaderOnlyResponse {
    pub header: CtrlHeader,
}

impl GpuResponse for HeaderOnlyResponse {
    fn header(&self) -> &CtrlHeader {
        &self.header
    }
}
