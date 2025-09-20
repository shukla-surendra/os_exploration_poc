#![no_std]
#![no_main]
#![feature(naked_functions)]

// ============================================================================
// MODULE DECLARATIONS - Core kernel modules
// ============================================================================
mod panic;              // panic handler
mod kernel;             // Core kernel subsystems

// ============================================================================
// IMPORTS - Only what we need for early boot
// ============================================================================
use core::arch::asm;
use kernel::serial::SERIAL_PORT;

// Multiboot2 constants (keep your existing ones)
const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
const MB_ARCH_I386:    u32 = 0; // architecture field (0 = i386 in multiboot2)
const MB_TAG_END_TYPE: u16 = 0;
const FB_TAG_SIZE: u32 = 20;
const FB_TAG_PAD: u32  = (FB_TAG_SIZE + 7) & !7; // -> 24
const MB_TAG_END_SIZE: u32 = 8; // end tag is 8 bytes (type(2)+flags(2)+size(4))
const END_TAG_SIZE: u32 = 8;

// Total header length = 16 bytes for the 4 header words + size of tags (we only have end tag = 8)
const HEADER_LEN: u32 = (16 + MB_TAG_END_SIZE) as u32;

// checksum so (magic + arch + len + checksum) == 0 (u32 wrap)
const MB_CHECKSUM: u32 = (0u32).wrapping_sub(MULTIBOOT2_MAGIC.wrapping_add(MB_ARCH_I386).wrapping_add(HEADER_LEN));

// 8-byte alignment required by the spec
#[repr(C, align(8))]
struct MbHeader {
    pub magic:    u32,
    pub arch:     u32,
    pub len:      u32,
    pub checksum: u32,

    pub fb_type:   u16,
    pub fb_flags:  u16,
    pub fb_size:   u32,
    pub fb_width:  u32,
    pub fb_height: u32,
    pub fb_depth:  u32,

    pub fb_pad:    [u8; (FB_TAG_PAD - FB_TAG_SIZE) as usize], // 4 bytes

    pub end_type:  u16,
    pub end_flags: u16,
    pub end_size:  u32,
}

// Place header in a named section and keep it used so the linker does not discard it.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".multiboot2")]
#[used]
pub static MULTIBOOT2_HEADER: MbHeader = MbHeader {
    magic: MULTIBOOT2_MAGIC,
    arch:  MB_ARCH_I386,
    len:   HEADER_LEN,
    checksum: MB_CHECKSUM,

    fb_type:  5,
    fb_flags: 0,
    fb_size:  FB_TAG_SIZE,
    fb_width: 1024,
    fb_height: 768,
    fb_depth: 32,

    fb_pad: [0; 4],

    end_type: 0,
    end_flags: 0,
    end_size: END_TAG_SIZE,
};

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
pub extern "C" fn _start() -> ! {
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
            asm!("hlt", options(nostack, nomem));
            count += 1;
            if count % 100 == 0 {
                SERIAL_PORT.write_str("Still running after timer interrupt\n");
            }
        }
    }
    
    // Kernal main loop
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem));
        }
    }
}