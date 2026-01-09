//! TEAM_247: Pseudo-Terminal (PTY) Implementation
//!
//! PTY consists of a master side and a slave side.
//! Master acts as the terminal emulator (hardware).
//! Slave acts as the TTY device with line discipline.

use crate::fs::tty::TtyState;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use los_utils::Mutex;

/// TEAM_247: A pair of PTY master and slave.
pub struct PtyPair {
    pub id: usize,
    pub tty: Arc<Mutex<TtyState>>,
    /// Buffer for output from slave to master (terminal emulator read)
    pub master_read_buffer: Arc<Mutex<VecDeque<u8>>>,
    pub locked: Mutex<bool>,
}

impl PtyPair {
    pub fn new(id: usize) -> Arc<Self> {
        let master_read_buffer = Arc::new(Mutex::new(VecDeque::new()));
        let tty_state = TtyState::new();
        let tty = Arc::new(Mutex::new(tty_state));

        // Set the master buffer for the slave's TTY state
        tty.lock().master_buffer = Some(master_read_buffer.clone());

        Arc::new(Self {
            id,
            tty,
            master_read_buffer,
            locked: Mutex::new(true),
        })
    }
}

/// TEAM_247: Global list of allocated PTYs.
static PTYS: Mutex<Vec<Arc<PtyPair>>> = Mutex::new(Vec::new());

pub fn allocate_pty() -> Option<Arc<PtyPair>> {
    let mut ptys = PTYS.lock();
    let id = ptys.len();
    let pair = PtyPair::new(id);
    ptys.push(pair.clone());
    Some(pair)
}

pub fn get_pty(id: usize) -> Option<Arc<PtyPair>> {
    let ptys = PTYS.lock();
    ptys.get(id).cloned()
}
