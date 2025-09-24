#![no_std]
#![no_main]


mod panic;              // panic handler
mod kernel;             // Core kernel subsystems

use core::arch::asm;
use kernel::serial::SERIAL_PORT;

use limine::BaseRevision;
use limine::request::{FramebufferRequest, RequestsEndMarker, RequestsStartMarker};

/// Sets the base revision to the latest revision supported by the crate.
/// See specification for further info.
/// Be sure to mark all limine requests with #[used], otherwise they may be removed by the compiler.
#[used]
// The .requests section allows limine to find the requests faster and more safely.
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Define the stand and end markers for Limine requests.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

// ============================================================================
// ULTRA-MINIMAL INTERRUPT SETUP FOR DEBUGGING
// ============================================================================

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,     
    selector: u16,       
    ist: u8,            
    type_attributes: u8, 
    offset_mid: u16,     
    offset_high: u32,    
    reserved: u32,       
}

impl IdtEntry {
    const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attributes: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: unsafe extern "C" fn()) {
        let addr = handler as usize as u64;
        self.offset_low = (addr & 0xFFFF) as u16;
        self.offset_mid = ((addr >> 16) & 0xFFFF) as u16;
        self.offset_high = ((addr >> 32) & 0xFFFFFFFF) as u32;
        self.selector = 0x08; // kernel code segment
        self.type_attributes = 0x8E; // present, ring 0, 64-bit interrupt gate
        self.ist = 0;
        self.reserved = 0;
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];

// ULTRA-SIMPLE interrupt handlers - just send EOI and return
#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn simple_timer_handler() {
    core::arch::naked_asm!(
        // Save only what we absolutely need
        "push rax",
        
        // Send EOI immediately
        "mov al, 0x20",
        "out 0x20, al",
        
        // Restore and return
        "pop rax", 
        "iretq"
    );
}

// Even simpler - disable timer interrupts entirely
#[unsafe(naked)]
#[unsafe(no_mangle)] 
unsafe extern "C" fn disable_timer_handler() {
    core::arch::naked_asm!(
        "push rax",
        "mov al, 0x20",
        "out 0x20, al",     // Just send EOI, don't disable
        "pop rax",
        "iretq"
    );
}

// Universal fault handler - just halt
#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn fault_handler() {
    core::arch::naked_asm!(
        "cli",              // Disable interrupts
        "hlt",              // Halt CPU
        "jmp fault_handler" // Infinite loop if hlt fails
    );
}

// Port I/O helpers
#[inline]
unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nostack, nomem));
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nostack, nomem));
    value
}

// Minimal interrupt setup
unsafe fn init_minimal_interrupts() {
    unsafe{

        SERIAL_PORT.write_str("=== MINIMAL INTERRUPT SETUP ===\n");
        // Set ALL entries to fault handler first
        for i in 0..256 {
            IDT[i].set_handler(fault_handler);
        }
        SERIAL_PORT.write_str("All IDT entries set to fault handler\n");
        
        // Set timer to our simple handler
        IDT[32].set_handler(disable_timer_handler); // This will disable itself
        SERIAL_PORT.write_str("Timer handler set (will self-disable)\n");

    }

    
    // Load IDT
    let idt_descriptor = IdtDescriptor {
        size: (core::mem::size_of::<[IdtEntry; 256]>() - 1) as u16,
        offset: core::ptr::addr_of!(IDT) as u64,
    };
    
    asm!("lidt [{}]", in(reg) &idt_descriptor, options(nostack));
    unsafe {
        SERIAL_PORT.write_str("IDT loaded\n");
    
        // Initialize PIC very conservatively
        init_minimal_pic();
        
        SERIAL_PORT.write_str("About to enable interrupts (minimal)...\n");
        
        // Enable interrupts
        asm!("sti", options(nostack, nomem));
        SERIAL_PORT.write_str("Interrupts enabled!\n");
        SERIAL_PORT.write_str("Timer should fire once then disable itself...\n");

    }

}

// Ultra-conservative PIC init
unsafe fn init_minimal_pic() {
    unsafe{
        SERIAL_PORT.write_str("Minimal PIC setup...\n");    
        // Disable ALL interrupts first
        outb(0x21, 0xFF);
        outb(0xA1, 0xFF);
        
        // Reinitialize PIC
        outb(0x20, 0x11); // ICW1
        outb(0xA0, 0x11); // ICW1
        outb(0x21, 32);   // ICW2 - master offset  
        outb(0xA1, 40);   // ICW2 - slave offset
        outb(0x21, 4);    // ICW3 - master
        outb(0xA1, 2);    // ICW3 - slave
        outb(0x21, 0x01); // ICW4 - master
        outb(0xA1, 0x01); // ICW4 - slave
        
        // Enable ONLY timer (IRQ0) 
        outb(0x21, 0xFE); // 11111110 - enable IRQ0 only
        outb(0xA1, 0xFF); // Disable all slave interrupts
        
        SERIAL_PORT.write_str("PIC configured (timer only)\n");

    }

}



#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
            // Initialize serial port first
    unsafe {
        // Initialize serial port first
        SERIAL_PORT.init();
        SERIAL_PORT.write_str("\n=== INTERRUPT DEBUG SESSION ===\n");
        // Second test: minimal interrupt setup (uncomment after first test works)
        init_minimal_interrupts();
        // 
        SERIAL_PORT.write_str("Waiting to see if timer fires and disables itself...\n");
        
        let mut count = 0;
        loop {
            asm!("hlt");
            count += 1;
            if count % 100 == 0 {
                SERIAL_PORT.write_str("Still running after timer interrupt\n");
            }
        }
    }
        
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    assert!(BASE_REVISION.is_supported());

    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..100_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                unsafe {
                    framebuffer
                        .addr()
                        .add(pixel_offset as usize)
                        .cast::<u32>()
                        .write(0xFFFFFFFF)
                };
            }
        }
    }

    hcf();
}



fn hcf() -> ! {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            asm!("hlt");
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            asm!("wfi");
            #[cfg(target_arch = "loongarch64")]
            asm!("idle 0");
        }
    }
}
