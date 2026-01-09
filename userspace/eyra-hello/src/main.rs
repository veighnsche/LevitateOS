// TEAM_351: Eyra test binary for LevitateOS
//
// This binary uses Eyra to provide Rust's std library via direct Linux syscalls.
// If this runs successfully, LevitateOS has basic std support.

// Required for -Zbuild-std compatibility (see Eyra README)
extern crate eyra;

use std::time::Instant;

fn main() {
    println!("=== Eyra Test on LevitateOS ===");
    println!();

    // Tier 1: Basic output
    println!("[OK] println! works");

    // Tier 2: Environment
    let args: Vec<String> = std::env::args().collect();
    println!("[OK] argc = {}", args.len());
    for (i, arg) in args.iter().enumerate() {
        println!("     argv[{}] = '{}'", i, arg);
    }

    // Tier 2: Time
    let start = Instant::now();
    println!("[OK] Instant::now() works");
    
    // Small delay via busy loop
    for _ in 0..1000 {
        core::hint::black_box(0);
    }
    
    let elapsed = start.elapsed();
    println!("[OK] elapsed = {:?}", elapsed);

    // Tier 2: Random (used by HashMap)
    // This tests getrandom syscall
    let random_val: u64 = {
        let mut buf = [0u8; 8];
        // Use inline syscall-like approach if std::collections works
        // For now just check HashMap compiles
        use std::collections::HashMap;
        let mut map: HashMap<&str, i32> = HashMap::new();
        map.insert("test", 42);
        *map.get("test").unwrap_or(&0) as u64
    };
    println!("[OK] HashMap works (getrandom ok), value = {}", random_val);

    // Tier 3: Threading (optional - may fail)
    #[cfg(feature = "test_threads")]
    {
        let handle = std::thread::spawn(|| {
            println!("[OK] Thread spawned!");
            42
        });
        match handle.join() {
            Ok(val) => println!("[OK] Thread joined, returned {}", val),
            Err(_) => println!("[FAIL] Thread join failed"),
        }
    }

    // Tier 4: File I/O (optional - may fail)
    #[cfg(feature = "test_fs")]
    {
        match std::fs::write("/tmp/eyra-test.txt", "Hello from Eyra!") {
            Ok(_) => println!("[OK] File write works"),
            Err(e) => println!("[FAIL] File write: {}", e),
        }
        
        match std::fs::read_to_string("/tmp/eyra-test.txt") {
            Ok(content) => println!("[OK] File read: '{}'", content),
            Err(e) => println!("[FAIL] File read: {}", e),
        }
    }

    println!();
    println!("=== Eyra Test Complete ===");
}
