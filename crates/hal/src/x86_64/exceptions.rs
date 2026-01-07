// TEAM_259: CPU Exception Handlers for x86_64.

use core::arch::{asm, naked_asm};
use crate::x86_64::idt::IDT;

#[repr(C)]
#[derive(Debug)]
pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

macro_rules! exception_handler {
    ($name:ident, $handler:ident) => {
        #[unsafe(naked)]
        pub unsafe extern "C" fn $name() {
            naked_asm!(
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "mov rdi, rsp",
                "add rdi, 72", // Point to ExceptionStackFrame
                "call {handler}",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",
                "iretq",
                handler = sym $handler,
            );
        }
    };
}

macro_rules! exception_handler_err {
    ($name:ident, $handler:ident) => {
        #[unsafe(naked)]
        pub unsafe extern "C" fn $name() {
            naked_asm!(
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "mov rdi, rsp",
                "add rdi, 72", // Point to error code
                "mov rsi, [rdi]", // Error code
                "add rdi, 8",  // Point to ExceptionStackFrame
                "call {handler}",
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rdi",
                "pop rsi",
                "pop rdx",
                "pop rcx",
                "pop rax",
                "add rsp, 8", // Clean up error code
                "iretq",
                handler = sym $handler,
            );
        }
    };
}

extern "C" fn divide_error_handler(frame: &ExceptionStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n{:#?}", frame);
}

extern "C" fn debug_handler(frame: &ExceptionStackFrame) {
    panic!("EXCEPTION: DEBUG\n{:#?}", frame);
}

extern "C" fn breakpoint_handler(frame: &ExceptionStackFrame) {
    // Just print for now to verify IDT works
    // TODO: Use a proper logger once serial/VGA are wired
    let _ = frame;
}

extern "C" fn invalid_opcode_handler(frame: &ExceptionStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{instruction_pointer:x}", instruction_pointer = frame.instruction_pointer);
}

extern "C" fn double_fault_handler(frame: &ExceptionStackFrame, error_code: u64) {
    panic!("EXCEPTION: DOUBLE FAULT\nError Code: {}\n{:#?}", error_code, frame);
}

extern "C" fn general_protection_fault_handler(frame: &ExceptionStackFrame, error_code: u64) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {}\n{:#?}", error_code, frame);
}

extern "C" fn page_fault_handler(frame: &ExceptionStackFrame, error_code: u64) {
    let cr2: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2);
    }
    panic!("EXCEPTION: PAGE FAULT\nAccessed Address: {cr2:x}\nError Code: {error_code:?}\n{frame:#?}");
}

exception_handler!(de_wrapper, divide_error_handler);
exception_handler!(db_wrapper, debug_handler);
exception_handler!(bp_wrapper, breakpoint_handler);
exception_handler!(ud_wrapper, invalid_opcode_handler);
exception_handler_err!(df_wrapper, double_fault_handler);
exception_handler_err!(gp_wrapper, general_protection_fault_handler);
exception_handler_err!(pf_wrapper, page_fault_handler);

pub fn init() {
    let mut idt = IDT.lock();
    idt.set_handler(0, de_wrapper as *const () as u64);
    idt.set_handler(1, db_wrapper as *const () as u64);
    idt.set_handler(3, bp_wrapper as *const () as u64);
    idt.set_handler(6, ud_wrapper as *const () as u64);
    idt.set_handler(8, df_wrapper as *const () as u64);
    idt.set_handler(13, gp_wrapper as *const () as u64);
    idt.set_handler(14, pf_wrapper as *const () as u64);
}
