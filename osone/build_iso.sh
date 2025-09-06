#!/usr/bin/env bash
# set -e  # exit on first error

TARGET="osone.json"
KERNEL_NAME="osone"
BUILD_DIR="os_iso"
ISO_NAME="osone.iso"

cargo clean
rm -rf $BUILD_DIR
rm $ISO_NAME

# 1. Build kernel ELF
echo "[*] Building kernel..."
cargo build --target $TARGET -Zbuild-std=core,alloc

# 2. Prepare ISO directory
echo "[*] Setting up ISO directory structure..."
rm -rf $BUILD_DIR
mkdir -p $BUILD_DIR/boot/grub

# 3. Copy kernel
cp target/osone/debug/$KERNEL_NAME $BUILD_DIR/boot/kernel.elf

# 4. Write grub.cfg
cat > $BUILD_DIR/boot/grub/grub.cfg <<'EOF'
set timeout=5 # pause screen for selection of OS
set default=0

menuentry "osone Auto" {
    insmod all_video
    insmod gfxterm
    insmod vbe
    set gfxmode=1024x768x32
    set gfxpayload=keep
    terminal_output gfxterm
    multiboot2 /boot/kernel.elf
    boot
}

menuentry "osone 1024x768x32" {
    insmod all_video
    insmod gfxterm
    insmod vbe
    insmod vga
    # (optional) insmod multiboot2
    set gfxmode=1024x768x32
    set gfxpayload=keep
    terminal_output gfxterm

    echo "You are seeing this because it supports 1024x768x32"
    echo "Press any key to boot"
    pause

    multiboot2 /boot/kernel.elf
    boot
}
EOF


# 5. Build ISO
echo "[*] Creating ISO..."
grub-mkrescue -o $ISO_NAME $BUILD_DIR

echo "[*] Done. ISO available as $ISO_NAME"
echo "Run with: qemu-system-i386 -cdrom $ISO_NAME"
# qemu-system-i386 -cdrom OxideOS.iso -serial stdio
qemu-system-i386 -cdrom $ISO_NAME
