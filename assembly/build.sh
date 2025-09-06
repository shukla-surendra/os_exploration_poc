nasm -f bin boot.asm -o target/boot.bin          # produce 512 byte binary
# create empty 1.44MB floppy image (only if needed)
dd if=/dev/zero of=target/floppy.img bs=512 count=2880
# write our boot sector into the image (safe)
dd if=target/boot.bin of=target/floppy.img bs=512 count=1 conv=notrunc
# run it in QEMU
qemu-system-x86_64 -fda target/floppy.img
