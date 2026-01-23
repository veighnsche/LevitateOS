# TEAM_091: Download Subsystem Bug Fixes

**Started:** 2026-01-23
**Status:** Complete

## Summary

Comprehensive audit and fix of the `leviso/src/deps/` download subsystem. Multiple critical bugs discovered during code review.

## Critical Bugs Fixed

### 1. Torrent download path ignored (`rocky.rs:189`)
- **Bug:** `download::torrent()` returns actual downloaded file path, but it was ignored with `Ok(_path)`
- **Impact:** If torrent filename differs from `config.filename`, code points to wrong/non-existent file
- **Fix:** Use returned path, rename to expected filename if different

### 2. HTTP resume corrupts files (`download.rs:210`)
- **Bug:** When resuming download, if server ignores Range header and returns 200 OK instead of 206, code appends full file to partial
- **Impact:** Corrupted downloads that pass existence checks
- **Fix:** Detect 200 response when Range was requested, truncate and restart

### 3. Existing Rocky ISO never checksum verified (`rocky.rs:114`)
- **Bug:** `find_existing()` only checks `path.exists()`, partial/corrupted files accepted
- **Impact:** Corrupted ISO used for build
- **Fix:** Verify checksum on existing files, delete and re-download if mismatch

## High-Risk Edge Cases Fixed

### 4. Torrent picks wrong file in shared directory (`download.rs:381`)
- **Bug:** Uses most recently modified file in directory
- **Fix:** Download to unique temp directory, then move to final location

### 5. Git clone fails on existing broken directory (`linux.rs:132`)
- **Bug:** If `downloads/linux` exists but invalid, clone fails
- **Fix:** Remove invalid directory before clone

### 6. extract_file_from_tarball hardcodes gzip (`download.rs:520`)
- **Bug:** Always uses `xzf` regardless of actual compression
- **Fix:** Detect compression from extension like `extract_tarball()` does

## Robustness Improvements

### 7. GitHub rate limit error unhelpful (`tools.rs:342`)
- **Fix:** Detect 403/429 and suggest GITHUB_TOKEN

### 8. No disk space check before 8.6GB download
- **Fix:** Check available space before starting large downloads
- **Fix (round 2):** Use POSIX-compatible `df -k` instead of GNU-specific flags

### 9. No git clone timeout (`download.rs:413`)
- **Fix:** Add configurable timeout (default 10 minutes)
- **Fix (round 2):** Remove kernel-specific `Makefile` check from generic git_clone function

### 10. Partial downloads not cleaned up
- **Fix:** Delete partial file on download failure

### 11. Cached tool binary not validated (`tools.rs:261`)
- **Fix (round 2):** Validate cached binary with `is_valid()` before use, re-download if invalid

### 12. HTTP download doesn't verify final size (`download.rs`)
- **Fix (round 2):** Verify downloaded file size matches `expected_size` if provided

### 13. Torrent assumes single file (`download.rs:418`)
- **Fix (round 2):** Handle directory torrents by finding largest file inside

## Files Modified

- `leviso/src/deps/download.rs`
- `leviso/src/deps/rocky.rs`
- `leviso/src/deps/linux.rs`
- `leviso/src/deps/tools.rs`

## Testing

- [x] All 34 unit tests pass
- [x] Code compiles without errors
- [ ] Rocky ISO download with checksum verification (manual test needed)
- [ ] HTTP resume with non-supporting server (manual test needed)
- [ ] Git clone with existing broken directory (manual test needed)
- [ ] Tool download with GitHub rate limit simulation (manual test needed)
