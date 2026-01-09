# Phase 4: Integration & Testing — ELF Dynamic Loader

**TEAM_352** | ELF Loader Feature  
**Created:** 2026-01-09  
**Depends on:** Phase 3 (Implementation)

---

## 1. Objective

Verify the ELF dynamic loader works correctly with:
1. PIE binaries (Eyra)
2. Existing ET_EXEC binaries (shell, init)
3. Both architectures (aarch64, x86_64)

---

## 2. Test Matrix

| Binary | Type | Architecture | Expected Result |
|--------|------|--------------|-----------------|
| eyra-hello | ET_DYN (PIE) | aarch64 | Prints output, exits 0 |
| eyra-hello | ET_DYN (PIE) | x86_64 | Prints output, exits 0 |
| shell | ET_EXEC | aarch64 | Works as before |
| init | ET_EXEC | aarch64 | Boots normally |

---

## 3. Test Procedures

### 3.1 Eyra PIE Test (aarch64)

```bash
# Build eyra-hello
cd userspace/eyra-hello
cargo build --release --target aarch64-unknown-linux-gnu -Zbuild-std=std,panic_abort

# Add to initramfs
mkdir -p ../../initramfs
cp target/aarch64-unknown-linux-gnu/release/eyra-hello ../../initramfs/

# Build and run
cd ../..
cargo xtask build --arch aarch64
cargo xtask run-vnc --arch aarch64

# In LevitateOS shell:
/eyra-hello
```

**Expected Output:**
```
=== Eyra Test on LevitateOS ===
[OK] println! works
[OK] argc = 1
[OK] Instant::now() works
[OK] elapsed = <time>
[OK] HashMap works (getrandom ok), value = 42
=== Eyra Test Complete ===
```

### 3.2 Eyra PIE Test (x86_64)

```bash
# Build for x86_64
cd userspace/eyra-hello
cargo build --release --target x86_64-unknown-linux-gnu -Zbuild-std=std,panic_abort

# Copy and test
cp target/x86_64-unknown-linux-gnu/release/eyra-hello ../../initramfs/
cd ../..
cargo xtask build --arch x86_64
cargo xtask run-vnc --arch x86_64
```

### 3.3 Regression Test (ET_EXEC)

```bash
# Boot normally
cargo xtask run-vnc --arch aarch64

# Shell should work
ls
cat /etc/motd

# Spawn processes
echo test
```

### 3.4 Golden Log Verification

```bash
cargo xtask test golden --arch aarch64
```

If golden logs changed due to new ELF loader messages, update with `--update`.

---

## 4. Debug Procedures

### 4.1 If Binary Doesn't Load

Check kernel logs for:
```
[ELF] Processing N relocations at offset 0xXXXX
[ELF] Applied N relocations
```

If missing, PIE detection or relocation parsing failed.

### 4.2 If Binary Crashes

Check for:
```
[EXCEPTION] Data Abort at 0xXXXX
```

Common causes:
- Missing relocations (address not adjusted)
- Wrong load base (collision with other mappings)
- Unsupported relocation type

### 4.3 Debug Commands

Add to kernel for debugging:
```rust
log::debug!("[ELF] load_base = 0x{:x}", load_base);
log::debug!("[ELF] entry_point = 0x{:x}", entry_point);
log::debug!("[ELF] segment at 0x{:x}, size 0x{:x}", vaddr, memsz);
```

---

## 5. Known Limitations

Document any limitations discovered:

| Limitation | Impact | Future Work |
|------------|--------|-------------|
| No ASLR | Security | Add randomization |
| R_*_RELATIVE only | Some binaries may fail | Add more reloc types |
| Fixed load base | May conflict | Dynamic base selection |

---

## 6. Success Criteria

- [ ] eyra-hello runs on aarch64
- [ ] eyra-hello runs on x86_64
- [ ] Existing shell works (regression)
- [ ] Init process boots (regression)
- [ ] No kernel panics
- [ ] Golden logs pass (or updated)

---

## 7. Next Phase

**Phase 5: Polish & Documentation** will finalize the feature.
