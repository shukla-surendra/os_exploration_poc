#!/bin/bash

# Usage: ./build.sh <assembly_file.asm>
# Example: ./build.sh hello.asm
# Will produce: target/hello.o and target/hello
# Then run: target/hello

# Check if a filename was provided
if [ -z "$1" ]; then
    echo "Usage: $0 <assembly_file.asm>"
    exit 1
fi

# Setup paths
filename=$(basename -- "$1")
name="${filename%.*}"
target_dir="target"

# Ensure target directory exists
mkdir -p "$target_dir"

# Assemble
nasm -f elf32 "$1" -o "$target_dir/$name.o"
if [ $? -ne 0 ]; then
    echo "❌ Assembly failed!"
    exit 1
fi

# Link
ld -m elf_i386 "$target_dir/$name.o" -o "$target_dir/$name"
if [ $? -ne 0 ]; then
    echo "❌ Linking failed!"
    exit 1
fi

# Run
echo "🚀 Running $target_dir/$name ..."
"$target_dir/$name"
