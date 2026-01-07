# Question: Boot Protocol and Toolchain

## Feature
Team 227: x86_64 Support

## Uncertainties

### 1. Boot Protocol
I have designed the boot process to use **Multiboot2**. This is supported natively by QEMU's `-kernel` flag (which loads multiboot kernels) and is the standard for bare-metal x86 development. 
- **Recommendation**: Proceed with Multiboot2.
- **Alternative**: Use UEFI or Limine protocol (requires more complex build/image creation).

### 2. Toolchain
Does your environment already have the `x86_64-unknown-none` target installed?
- **Action**: I will assume "Try to build, if fail, ask user to install".

## Decision Needed
Confirm **Multiboot2** is acceptable.
