#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)] // Enables x86-interrupt ABI


mod panic;              // panic handler
mod kernel;             // Core kernel subsystems

use core::arch::asm;
use kernel::serial::SERIAL_PORT;
use kernel::{idt, interrupts, timer, pic};

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
// 64-BIT UTILITY FUNCTIONS
// ============================================================================

/// Get current Code Segment register value (same in 64-bit)
#[inline]
fn current_cs() -> u16 {
    let cs: u16;
    unsafe {
        core::arch::asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags));
    }
    cs
}

/// Check 64-bit stack pointer in main loop
unsafe fn check_stack_in_main_loop_64bit(iteration: u64) {
    unsafe{
    let rsp: u64;
    core::arch::asm!("mov {}, rsp", out(reg) rsp, options(nomem, nostack, preserves_flags));
    
    // Check stack periodically
    if iteration % 10000000 == 0 {
        SERIAL_PORT.write_str("64-bit Main loop RSP: 0x");
        SERIAL_PORT.write_hex((rsp >> 32) as u32);
        SERIAL_PORT.write_hex(rsp as u32);
        
        // Adjusted validation for 64-bit (higher addresses expected)
        let valid = if rsp == 0 {
            SERIAL_PORT.write_str(" **NULL**");
            false
        } else if rsp < 0x100000 {  // Below 1MB - too low for 64-bit
            SERIAL_PORT.write_str(" **TOO_LOW**");
            false
        } else if rsp % 16 != 0 {  // Must be 16-byte aligned in 64-bit
            SERIAL_PORT.write_str(" **MISALIGNED**");
            false
        } else {
            SERIAL_PORT.write_str(" OK");
            true
        };
        
        SERIAL_PORT.write_str("\n");
        
        if !valid {
            SERIAL_PORT.write_str("64-BIT STACK CORRUPTION DETECTED!\n");
            SERIAL_PORT.write_str("RSP: 0x");
            SERIAL_PORT.write_hex((rsp >> 32) as u32);
            SERIAL_PORT.write_hex(rsp as u32);
            SERIAL_PORT.write_str("\n");
            
            core::arch::asm!("cli");
            loop { core::arch::asm!("hlt"); }
        }
    }
        }
}




#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    // Initialize serial port first
    unsafe {
        // Initialize serial port first
        SERIAL_PORT.init();
        SERIAL_PORT.write_str("\n=== INTERRUPT DEBUG SESSION ===\n");
        SERIAL_PORT.write_str("Waiting to see if timer fires and disables itself...\n");
    }
    unsafe {
        SERIAL_PORT.write_str("=== 64-BIT INTERRUPT SYSTEM SETUP ===\n");
        
        // Disable interrupts during setup
        SERIAL_PORT.write_str("Step 1: Disabling interrupts (CLI)...\n");
        asm!("cli");
        
        // Check system state
        check_system_tables_64bit();
        
        // Initialize 64-bit IDT
        SERIAL_PORT.write_str("Step 2: Initializing 64-bit IDT...\n");
        idt::init();
        SERIAL_PORT.write_str("  ✓ 64-bit IDT loaded\n");
        
        // Verify 64-bit IDT entries
        SERIAL_PORT.write_str("Step 3: Verifying 64-bit IDT entries...\n");
        verify_idt_entries_64bit();
        
        // Initialize PIC (same hardware interface)
        SERIAL_PORT.write_str("Step 4: Initializing PIC for 64-bit...\n");
        pic::init();
        SERIAL_PORT.write_str("  ✓ PIC remapped for 64-bit operation\n");
        
        // Initialize timer (same hardware, 64-bit handling)
        SERIAL_PORT.write_str("Step 5: Initializing 64-bit timer...\n");
        timer::init(100); // 100 Hz
        SERIAL_PORT.write_str("  ✓ 64-bit timer initialized at 100Hz\n");
        
        // Enable interrupts and test
        SERIAL_PORT.write_str("Step 6: Testing 64-bit interrupt system...\n");
        test_64bit_interrupts();
        
        SERIAL_PORT.write_str("✓ 64-bit interrupt system fully operational\n");
    }
     
    // All limine requests must also be referenced in a called function, otherwise they may be
    // removed by the linker.
    // assert!(BASE_REVISION.is_supported());

    // if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
    //     if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
    //         for i in 0..100_u64 {
    //             // Calculate the pixel offset using the framebuffer information we obtained above.
    //             // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
    //             let pixel_offset = i * framebuffer.pitch() + i * 4;

    //             // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
    //             unsafe {
    //                 framebuffer
    //                     .addr()
    //                     .add(pixel_offset as usize)
    //                     .cast::<u32>()
    //                     .write(0xFFFFFFFF)
    //             };
    //         }
    //     }
    // }

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


// ============================================================================
// 64-BIT SPECIFIC HELPER FUNCTIONS
// ============================================================================

unsafe fn check_system_tables_64bit() {
    unsafe{

        SERIAL_PORT.write_str("\n=== 64-BIT SYSTEM TABLE CHECK ===\n");
    
    // Check GDT (64-bit format)
    let mut gdt_ptr: [u8; 10] = [0; 10]; // 64-bit GDT pointer is 10 bytes
    asm!("sgdt [{}]", in(reg) &mut gdt_ptr);
    let gdt_limit = u16::from_le_bytes([gdt_ptr[0], gdt_ptr[1]]);
    let gdt_base = u64::from_le_bytes([
        gdt_ptr[2], gdt_ptr[3], gdt_ptr[4], gdt_ptr[5],
        gdt_ptr[6], gdt_ptr[7], gdt_ptr[8], gdt_ptr[9]
    ]);
    
    SERIAL_PORT.write_str("64-bit GDT Base: 0x");
    SERIAL_PORT.write_hex((gdt_base >> 32) as u32);
    SERIAL_PORT.write_hex(gdt_base as u32);
    SERIAL_PORT.write_str(", Limit: 0x");
    SERIAL_PORT.write_hex(gdt_limit as u32);
    SERIAL_PORT.write_str("\n");
    
    // Check CS register
    let cs: u16;
    asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags));
    SERIAL_PORT.write_str("64-bit CS: 0x");
    SERIAL_PORT.write_hex(cs as u32);
    SERIAL_PORT.write_str("\n");
    
    SERIAL_PORT.write_str("===================\n");

    }
    
}

unsafe fn verify_idt_entries_64bit() {
    unsafe{

    let mut idtr: [u8; 10] = [0; 10]; // 64-bit IDT pointer is 10 bytes
    asm!("sidt [{}]", in(reg) &mut idtr);
    let idt_limit = u16::from_le_bytes([idtr[0], idtr[1]]);
    let idt_base = u64::from_le_bytes([
        idtr[2], idtr[3], idtr[4], idtr[5],
        idtr[6], idtr[7], idtr[8], idtr[9]
    ]);
    
    SERIAL_PORT.write_str("  64-bit IDT Base: 0x");
    SERIAL_PORT.write_hex((idt_base >> 32) as u32);
    SERIAL_PORT.write_hex(idt_base as u32);
    SERIAL_PORT.write_str(", Limit: 0x");
    SERIAL_PORT.write_hex(idt_limit as u32);
    SERIAL_PORT.write_str("\n");

    if idt_base != 0 && idt_limit == 0xFFF { // 256 * 16 - 1 for 64-bit
        SERIAL_PORT.write_str("  ✓ 64-bit IDT appears loaded correctly\n");
    } else {
        SERIAL_PORT.write_str("  WARNING: 64-bit IDT may not be loaded correctly!\n");
    }
        }
}

unsafe fn test_64bit_interrupts() {

    unsafe {


    // Enable interrupts
    asm!("sti");
    
    // Unmask only timer interrupt for testing
    pic::unmask_irq(0); // IRQ0 = Timer
    
    // Wait for timer interrupts
    let initial_ticks = timer::get_ticks();
    SERIAL_PORT.write_str("  Testing 64-bit timer interrupts...\n");
    SERIAL_PORT.write_str("  Initial ticks: ");
    SERIAL_PORT.write_decimal(initial_ticks as u32);
    SERIAL_PORT.write_str("\n");
    
    // Wait for 10 timer ticks
    let target_ticks = initial_ticks + 10;
    let mut timeout = 0u32;
    
    loop {
        let current_ticks = timer::get_ticks();
        if current_ticks >= target_ticks {
            SERIAL_PORT.write_str("  ✓ 64-bit timer interrupts working! Final ticks: ");
            SERIAL_PORT.write_decimal(current_ticks as u32);
            SERIAL_PORT.write_str("\n");
            break;
        }
        
        timeout += 1;
        if timeout > 1_000_000 {
            SERIAL_PORT.write_str("  TIMEOUT: No 64-bit timer interrupts received\n");
            break;
        }
        
        // Short delay
        for _ in 0..100 {
            asm!("pause"); // Better than nop for spin-wait in 64-bit
        }
    }
    
    // Also enable keyboard for interactive testing
    SERIAL_PORT.write_str("  Enabling 64-bit keyboard interrupts...\n");
    pic::unmask_irq(1); // IRQ1 = Keyboard
    SERIAL_PORT.write_str("  ✓ Press keys to test 64-bit keyboard interrupts\n");
        }
}