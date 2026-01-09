# SimpleGPU Driver Stub

## Overview
This crate provides a generic framebuffer driver using UEFI GOP (Graphics Output Protocol).

## Features
- Native framebuffer access via Limine GOP response.
- Integration with the kernel `gpu` trait.

## Implementation Plan
1. Retrieve framebuffer info from bootloader.
2. Map framebuffer memory into kernel address space.
3. Implement `Gpu` trait for basic pixel blitting.
