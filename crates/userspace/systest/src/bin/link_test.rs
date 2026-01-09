//! TEAM_253: Test links (symlink, link, readlink)
#![no_std]
#![no_main]

extern crate ulib;
use ulib::libsyscall::{
    self, close, linkat, openat, println, read, readlinkat, symlinkat, unlinkat, write, O_CREAT,
    O_RDWR,
};

#[no_mangle]
pub fn main() -> i32 {
    println!("[link_test] Starting...");

    let target = "/tmp/target_file";
    let link = "/tmp/hard_link";
    let sym = "/tmp/sym_link";

    // Cleanup
    let _ = unlinkat(0, target, 0);
    let _ = unlinkat(0, link, 0);
    let _ = unlinkat(0, sym, 0);

    // Create target
    let fd = openat(target, O_CREAT | O_RDWR);
    if fd < 0 {
        println!("[link_test] FAIL: create target failed");
        return 1;
    }
    write(fd as usize, b"DATA");
    close(fd as usize);

    // 1. Hard Link
    if linkat(0, target, 0, link, 0) < 0 {
        println!("[link_test] FAIL: linkat failed");
        return 1;
    }

    let fd2 = openat(link, 0);
    if fd2 < 0 {
        println!("[link_test] FAIL: open hardlink failed");
        return 1;
    }
    let mut buf = [0u8; 4];
    read(fd2 as usize, &mut buf);
    close(fd2 as usize);
    if &buf != b"DATA" {
        println!("[link_test] FAIL: hardlink data mismatch");
        return 1;
    }

    // 2. Symlink
    if symlinkat(target, 0, sym) < 0 {
        println!("[link_test] FAIL: symlinkat failed");
        return 1;
    }

    // Readlink
    let mut linkbuf = [0u8; 64];
    let n = readlinkat(0, sym, &mut linkbuf);
    if n < 0 {
        println!("[link_test] FAIL: readlinkat failed");
        return 1;
    }
    let link_content = core::str::from_utf8(&linkbuf[..n as usize]).unwrap_or("");
    if link_content != target {
        println!(
            "[link_test] FAIL: readlink content mismatch. Got '{}'",
            link_content
        );
        return 1;
    }

    // Cleanup
    let _ = unlinkat(0, target, 0);
    let _ = unlinkat(0, link, 0);
    let _ = unlinkat(0, sym, 0);

    println!("[link_test] PASS");
    0
}
