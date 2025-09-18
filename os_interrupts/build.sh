#!/bin/bash

echo "Building Rust OS from scratch..."

# Make sure we're using nightly Rust
rustup default nightly 2>/dev/null || echo "Make sure you have Rust nightly installed"

# Add the target if not already added
rustup target add x86_64-unknown-none

# Build the kernel with build-std to compile core library from source
RUSTFLAGS="-C link-arg=-T -C link-arg=linker.ld" \
cargo build \
    --target x86_64-unknown-none \
    --release \
    -Z build-std=core,alloc \
    -Z build-std-features=compiler-builtins-mem

# Check if build succeeded
if [ ! -f "target/x86_64-unknown-none/release/rust_interrupt_test" ]; then
    echo "❌ Build failed - kernel binary not found"
    exit 1
fi

# Create bootable image directory
mkdir -p iso/boot/grub

# Copy kernel to ISO directory
cp target/x86_64-unknown-none/release/rust_interrupt_test iso/boot/kernel.bin

echo "✓ Kernel binary copied"

# Create GRUB configuration
cat > iso/boot/grub/grub.cfg << 'EOF'
set timeout=0
set default=0

menuentry "Rust OS Interrupt Test" {
    multiboot2 /boot/kernel.bin
    boot
}
EOF

# Create bootable ISO using grub-mkrescue
if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o rust_os.iso iso/
    echo "✓ Bootable ISO created: rust_os.iso"
else
    echo "! grub-mkrescue not found. Install GRUB tools."
    echo "  Ubuntu/Debian: sudo apt install grub-pc-bin grub-common xorriso"
    echo "  Arch: sudo pacman -S grub libisoburn"
    echo "  macOS: brew install i386-elf-grub xorriso"
fi

echo "Build complete!"
echo ""
echo "To run in QEMU:"
echo "  qemu-system-x86_64 -cdrom rust_os.iso"
echo ""
echo "To run in QEMU with debugging:"
echo "  qemu-system-x86_64 -cdrom rust_os.iso -s -S"
echo "  Then connect GDB: target remote :1234"