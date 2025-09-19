# Rust OS - Interrupt Test from Scratch

A minimal 64-bit Rust operating system kernel focused purely on interrupt handling, built completely from scratch with **zero external dependencies**.

## Features

- ✅ **Pure from scratch** - No bootloader, x86_64, or other crates
- ✅ **64-bit x86_64 architecture**
- ✅ **Custom VGA text driver**
- ✅ **Interrupt Descriptor Table (IDT) setup**
- ✅ **PIC (8259) configuration** 
- ✅ **Multiple interrupt handlers**:
  - Software interrupt (INT 0x80)
  - Timer interrupt (IRQ0)
  - Keyboard interrupt (IRQ1)
  - Page fault exception
  - General protection fault
- ✅ **Multiboot2 compliant**

## Prerequisites

```bash
# Install Rust nightly (required for naked functions and asm features)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install nightly
rustup default nightly

# Add required components
rustup component add rust-src
rustup component add llvm-tools-preview

# Install build tools (Ubuntu/Debian)
sudo apt update
sudo apt install build-essential grub-pc-bin grub-common xorriso qemu-system-x86

# Or on Arch Linux
sudo pacman -S base-devel grub libisoburn qemu

# Or on macOS
brew install i386-elf-grub xorriso qemu
```

## Quick Start

```bash
# Make scripts executable
chmod +x build.sh run.sh

# Build and run immediately
./run.sh

# Or build manually
./build.sh
qemu-system-x86_64 -cdrom rust_os.iso
```

## What You'll See

When you boot the kernel:

1. **Initialization messages** showing IDT and PIC setup
2. **Test interrupt trigger** - fires INT 0x80 to verify interrupt handling
3. **Timer interrupts** - periodic timer ticks displayed every second
4. **Keyboard interrupts** - press keys to see scancode values
5. **Interrupt handlers** working in real-time

## Project Structure

```
├── src/
│   ├── main.rs          # Kernel entry point and main loop
│   ├── vga.rs           # VGA text mode driver (pure assembly)
│   ├── idt.rs           # Interrupt Descriptor Table setup
│   ├── pic.rs           # Programmable Interrupt Controller
│   └── boot.s           # Multiboot2 header
├── .cargo/config.toml   # Cargo configuration
├── x86_64-unknown-none.json  # Custom target specification
├── linker.ld           # Linker script
├── build.sh            # Build automation
└── run.sh              # Build and run automation
```

## Understanding the Code

### Interrupt Flow

1. **IDT Setup** (`idt.rs`): Creates interrupt descriptor table with handlers
2. **PIC Setup** (`pic.rs`): Configures 8259 PIC for hardware interrupts  
3. **Handler Registration**: Maps interrupt vectors to Rust functions
4. **Naked Functions**: Assembly wrappers that save/restore CPU state
5. **Handler Logic**: Pure Rust code that responds to interrupts

### Key Learning Points

- **No external dependencies** - everything implemented from scratch
- **Inline assembly** for low-level hardware interaction
- **Memory-mapped I/O** for VGA and hardware ports
- **Interrupt handling** without any abstraction layers
- **Bare metal programming** with direct hardware access

## Testing Interrupts

The kernel provides several ways to test interrupts:

1. **Software Interrupt**: Automatically triggered on boot (`INT 0x80`)
2. **Timer Interrupt**: Hardware timer generates periodic interrupts
3. **Keyboard Interrupt**: Press any key to trigger keyboard interrupt
4. **Exception Handling**: Page faults and protection violations

## Debugging

```bash
# Run with GDB support
qemu-system-x86_64 -cdrom rust_os.iso -s -S

# In another terminal
gdb
(gdb) target remote :1234
(gdb) continue
```

## Next Steps

This minimal setup gives you a solid foundation to