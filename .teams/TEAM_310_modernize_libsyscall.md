# Team 310: Modernize libsyscall

## Goal
Replace hand-rolled syscall constants and structs in `userspace/libsyscall` with `linux-raw-sys` definitions.

## Plan
- [ ] Add `linux-raw-sys` dependency to `userspace/libsyscall/Cargo.toml`.
- [ ] Refactor `libsyscall` to use external definitions.
- [ ] Verify `levbox` compilation.
