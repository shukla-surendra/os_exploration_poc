use core::arch::{asm, global_asm};
use crate::vga;

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry {
    offset_low: 0,
    selector: 0,
    ist: 0,
    type_attr: 0,
    offset_mid: 0,
    offset_high: 0,
    zero: 0,
}; 256];

impl IdtEntry {
    fn new(handler: u64, selector: u16) -> Self {
        Self {
            offset_low: (handler & 0xFFFF) as u16,
            selector,
            ist: 0,
            type_attr: 0x8E, // Present, ring 0, interrupt gate
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: ((handler >> 32) & 0xFFFFFFFF) as u32,
            zero: 0,
        }
    }
}

// Define interrupt handlers in assembly
global_asm!(
    r#"
    .section .text
    .global test_interrupt_handler
    .global keyboard_interrupt_handler  
    .global timer_interrupt_handler
    .global page_fault_handler
    .global general_protection_fault_handler

    test_interrupt_handler:
        push rax
        push rcx
        push rdx
        push rsi
        push rdi
        push r8
        push r9
        push r10
        push r11
        call test_interrupt_handler_inner
        pop r11
        pop r10
        pop r9
        pop r8
        pop rdi
        pop rsi
        pop rdx
        pop rcx
        pop rax
        iretq

    keyboard_interrupt_handler:
        push rax
        push rcx
        push rdx
        call keyboard_interrupt_handler_inner
        mov al, 0x20
        out 0x20, al
        pop rdx
        pop rcx
        pop rax
        iretq

    timer_interrupt_handler:
        push rax
        call timer_interrupt_handler_inner
        mov al, 0x20
        out 0x20, al
        pop rax
        iretq

    page_fault_handler:
        push rax
        push rcx
        push rdx
        mov rax, cr2
        push rax
        call page_fault_handler_inner
        add rsp, 8
        pop rdx
        pop rcx
        pop rax
        iretq

    general_protection_fault_handler:
        push rax
        call general_protection_fault_handler_inner
        pop rax
        iretq
    "#
);

// External declarations for assembly handlers
unsafe extern "C" {
    unsafe fn test_interrupt_handler();
    unsafe fn keyboard_interrupt_handler();
    unsafe fn timer_interrupt_handler();
    unsafe fn page_fault_handler();
    unsafe fn general_protection_fault_handler();
}

pub fn init_idt() {
    unsafe {
        // Set up some basic interrupt handlers
        IDT[0x80] = IdtEntry::new(test_interrupt_handler as u64, 0x08);
        IDT[33] = IdtEntry::new(keyboard_interrupt_handler as u64, 0x08); // Keyboard
        IDT[32] = IdtEntry::new(timer_interrupt_handler as u64, 0x08);    // Timer
        IDT[14] = IdtEntry::new(page_fault_handler as u64, 0x08);         // Page fault
        IDT[13] = IdtEntry::new(general_protection_fault_handler as u64, 0x08); // GPF
        
        let idt_descriptor = IdtDescriptor {
            limit: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };

        asm!("lidt [{}]", in(reg) &idt_descriptor);
    }
}

#[unsafe(no_mangle)]
extern "C" fn test_interrupt_handler_inner() {
    vga::print_string("TEST INTERRUPT TRIGGERED! (0x80)\n");
}

#[unsafe(no_mangle)]
extern "C" fn keyboard_interrupt_handler_inner() {
    unsafe {
        let scancode: u8;
        asm!("in al, 0x60", out("al") scancode);
        
        vga::print_string("Keyboard: ");
        vga::print_hex(scancode as u64);
        vga::print_string("\n");
    }
}

static mut TIMER_TICKS: u64 = 0;

#[unsafe(no_mangle)]
extern "C" fn timer_interrupt_handler_inner() {
    unsafe {
        TIMER_TICKS += 1;
        if TIMER_TICKS % 100 == 0 { // Print every ~1 second
            vga::print_string("Timer tick: ");
            vga::print_hex(TIMER_TICKS);
            vga::print_string("\n");
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn page_fault_handler_inner(fault_addr: u64) {
    vga::print_string("PAGE FAULT at address: ");
    vga::print_hex(fault_addr);
    vga::print_string("\n");
}

#[unsafe(no_mangle)]
extern "C" fn general_protection_fault_handler_inner() {
    vga::print_string("GENERAL PROTECTION FAULT!\n");
}