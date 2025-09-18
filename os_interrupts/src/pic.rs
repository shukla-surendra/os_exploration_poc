use core::arch::asm;

// PIC ports
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

// PIC commands
const PIC_EOI: u8 = 0x20;

// Initialization Command Words
const ICW1_ICW4: u8 = 0x01;
const ICW1_SINGLE: u8 = 0x02;
const ICW1_INTERVAL4: u8 = 0x04;
const ICW1_LEVEL: u8 = 0x08;
const ICW1_INIT: u8 = 0x10;

const ICW4_8086: u8 = 0x01;
const ICW4_AUTO: u8 = 0x02;
const ICW4_BUF_SLAVE: u8 = 0x08;
const ICW4_BUF_MASTER: u8 = 0x0C;
const ICW4_SFNM: u8 = 0x10;

pub fn init_pic() {
    unsafe {
        // Save masks (we'll ignore them for simplicity)
        let _mask1 = inb(PIC1_DATA);
        let _mask2 = inb(PIC2_DATA);

        // Start initialization sequence
        outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();
        outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
        io_wait();

        // Set vector offsets
        outb(PIC1_DATA, 32);  // Master PIC vector offset (32-39)
        io_wait();
        outb(PIC2_DATA, 40);  // Slave PIC vector offset (40-47)
        io_wait();

        // Tell Master PIC about slave PIC at IRQ2
        outb(PIC1_DATA, 4);
        io_wait();
        // Tell Slave PIC its cascade identity
        outb(PIC2_DATA, 2);
        io_wait();

        // Set mode to 8086
        outb(PIC1_DATA, ICW4_8086);
        io_wait();
        outb(PIC2_DATA, ICW4_8086);
        io_wait();

        // Unmask IRQ0 (timer) and IRQ1 (keyboard)
        outb(PIC1_DATA, 0xFC); // Mask all except IRQ0 and IRQ1
        outb(PIC2_DATA, 0xFF); // Mask all slave PIC interrupts for now
    }
}

pub fn send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_COMMAND, PIC_EOI);
        }
        outb(PIC1_COMMAND, PIC_EOI);
    }
}

pub fn set_mask(irq_line: u8) {
    unsafe {
        let port = if irq_line < 8 { PIC1_DATA } else { PIC2_DATA };
        let value = inb(port);
        outb(port, value | (1 << (irq_line % 8)));
    }
}

pub fn clear_mask(irq_line: u8) {
    unsafe {
        let port = if irq_line < 8 { PIC1_DATA } else { PIC2_DATA };
        let value = inb(port);
        outb(port, value & !(1 << (irq_line % 8)));
    }
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value);
    }
}

unsafe fn inb(port: u16) -> u8 {
    unsafe {
        let result: u8;
        asm!("in al, dx", out("al") result, in("dx") port);
        result
    }
}

unsafe fn io_wait() {
    unsafe {
        // Wait by doing a dummy write to port 0x80
        asm!("out 0x80, al", in("al") 0u8);
    }
}