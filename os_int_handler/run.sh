#!/bin/bash

# Quick build and run script
echo "Building and running Rust OS..."

# Build
./build.sh

# Check if build was successful
if [ -f "rust_os.iso" ]; then
    echo "Starting QEMU..."
    echo "Press Ctrl+Alt+G to release mouse, Ctrl+Alt+Q to quit"
    
    qemu-system-x86_64 \
        -cdrom rust_os.iso \
        -m 512M \
        -serial stdio \
        -display gtk
else
    echo "Build failed - no ISO file found"
    exit 1
fi