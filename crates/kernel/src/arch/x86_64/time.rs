//! TEAM_222: x86_64 time stubs

pub fn read_timer_counter() -> u64 {
    // rdtsc or similar
    0
}

pub fn read_timer_frequency() -> u64 {
    // tsc frequency
    0
}
