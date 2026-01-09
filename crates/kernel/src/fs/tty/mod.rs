//! TEAM_247: TTY and Terminal support for LevitateOS.
//!
//! Implements POSIX terminal features including line discipline,
//! termios configuration, and signal generation.

pub use crate::arch::{NCCS, Termios};
use alloc::sync::Arc;
use los_utils::Mutex;

pub use crate::arch::{
    ECHO, ECHOE, ECHOK, ECHONL, ICANON, IEXTEN, ISIG, NOFLSH, ONLCR, OPOST, TCGETS, TCSETS,
    TCSETSF, TCSETSW, TIOCGPTN, TIOCGWINSZ, TIOCSPTLCK, TIOCSWINSZ, TOSTOP, VEOF, VERASE, VINTR,
    VKILL, VMIN, VQUIT, VSTART, VSTOP, VSUSP, VTIME,
};

pub mod pty;

use alloc::collections::VecDeque;

/// TEAM_247: TTY state for a terminal device.
pub struct TtyState {
    pub termios: Termios,
    /// Chars ready to be read by the process
    pub input_buffer: VecDeque<u8>,
    /// Chars being edited in canonical mode
    pub canon_buffer: VecDeque<u8>,
    /// Whether output is stopped by IXON (Ctrl+S)
    pub stopped: bool,
    /// TEAM_247: Optional buffer for terminal emulator (PTY master)
    pub master_buffer: Option<Arc<Mutex<VecDeque<u8>>>>,
}

impl TtyState {
    pub fn new() -> Self {
        Self {
            termios: Termios::INITIAL_TERMIOS,
            input_buffer: VecDeque::new(),
            canon_buffer: VecDeque::new(),
            stopped: false,
            master_buffer: None,
        }
    }

    /// Process a raw input byte from hardware.
    /// Returns true if something was added to input_buffer.
    pub fn process_input(&mut self, mut byte: u8) -> bool {
        // 1. Input processing (iflag)
        const IXON: u32 = 0x0400;
        if (self.termios.c_iflag & IXON) != 0 {
            if byte == self.termios.c_cc[VSTOP] {
                self.stopped = true;
                return false;
            }
            if byte == self.termios.c_cc[VSTART] {
                self.stopped = false;
                return false;
            }
        }

        // If we were stopped and any char is received (with IXANY? but we'll just resume on any for now or specific)
        // Actually IXON usually means VSTART resumes.

        if byte == b'\r' && (self.termios.c_iflag & 0x0100) != 0 {
            // ICRNL
            byte = b'\n';
        }

        // 2. Local processing/Signals (lflag)
        if (self.termios.c_lflag & ISIG) != 0 {
            if byte == self.termios.c_cc[VINTR] {
                crate::syscall::signal::signal_foreground_process(crate::syscall::signal::SIGINT);
                return false;
            }
            if byte == self.termios.c_cc[VQUIT] {
                crate::syscall::signal::signal_foreground_process(3); // SIGQUIT = 3
                return false;
            }
            if byte == self.termios.c_cc[VSUSP] {
                crate::syscall::signal::signal_foreground_process(20); // SIGTSTP = 20
                return false;
            }
        }

        // TEAM_327: Handle erase characters BEFORE generic echo
        // This prevents ^H from being echoed before the visual erase
        if (self.termios.c_lflag & ICANON) != 0 {
            // Handle both BS (0x08) and DEL (0x7F) as erase characters
            if byte == self.termios.c_cc[VERASE] || byte == 0x08 || byte == 0x7F {
                if let Some(_last) = self.canon_buffer.pop_back() {
                    if (self.termios.c_lflag & ECHOE) != 0 {
                        // Visual erase: backspace - space - backspace
                        self.echo(b'\x08');
                        self.echo(b' ');
                        self.echo(b'\x08');
                    }
                }
                return false;
            }
        }

        // 3. Echoing (after special character handling)
        if (self.termios.c_lflag & ECHO) != 0 {
            if byte == b'\n' {
                self.echo(b'\r');
                self.echo(b'\n');
            } else if byte < 32 && byte != b'\t' {
                // Echo as ^X
                self.echo(b'^');
                self.echo(byte + 64);
            } else {
                self.echo(byte);
            }
        }

        // 4. Line Discipline (Canonical vs Non-canonical)
        if (self.termios.c_lflag & ICANON) != 0 {
            // Erase already handled above
            if byte == self.termios.c_cc[VKILL] {
                self.canon_buffer.clear();
                // TODO: Visual kill
                return false;
            }
            if byte == self.termios.c_cc[VEOF] {
                // EOF - flush canon_buffer to input_buffer
                while let Some(b) = self.canon_buffer.pop_front() {
                    self.input_buffer.push_back(b);
                }
                return true;
            }

            self.canon_buffer.push_back(byte);
            if byte == b'\n' {
                // EOL - flush canon_buffer to input_buffer
                while let Some(b) = self.canon_buffer.pop_front() {
                    self.input_buffer.push_back(b);
                }
                return true;
            }
        } else {
            // Non-canonical: straight to input buffer
            self.input_buffer.push_back(byte);
            return true;
        }

        false
    }

    fn echo(&mut self, byte: u8) {
        if let Some(ref buffer) = self.master_buffer {
            buffer.lock().push_back(byte);
        } else {
            los_hal::print!("{}", byte as char);
        }
    }
}

/// TEAM_247: Global console TTY state.
pub static CONSOLE_TTY: Mutex<TtyState> = Mutex::new(TtyState {
    termios: Termios::INITIAL_TERMIOS,
    input_buffer: VecDeque::new(),
    canon_buffer: VecDeque::new(),
    stopped: false,
    master_buffer: None,
});
