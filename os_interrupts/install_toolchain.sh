#!/bin/bash

set -e  # Exit on any error

echo "Installing Rust OS Development Toolchain"
echo "========================================"

# Detect OS
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [ -f /etc/debian_version ]; then
        OS="debian"
    elif [ -f /etc/arch-release ]; then
        OS="arch"
    elif [ -f /etc/fedora-release ]; then
        OS="fedora"
    else
        OS="linux"
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
else
    echo "❌ Unsupported OS: $OSTYPE"
    exit 1
fi

echo "Detected OS: $OS"
echo ""

# 1. Install Rust
echo "1️⃣  Installing Rust toolchain..."
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "✓ Rust installed"
else
    echo "✓ Rust already installed"
fi

# 2. Install Rust nightly and components
echo ""
echo "2️⃣  Setting up Rust nightly..."
rustup install nightly
rustup default nightly
rustup component add rust-src
rustup component add llvm-tools-preview
echo "✓ Rust nightly configured"

# 3. Install build tools based on OS
echo ""
echo "3️⃣  Installing build tools for $OS..."

case $OS in
    "debian")
        sudo apt update
        sudo apt install -y build-essential nasm qemu-system-x86 grub-pc-bin grub-common xorriso mtools
        echo "✓ Debian/Ubuntu packages installed"
        ;;
    "arch")
        sudo pacman -Syu --needed base-devel nasm qemu grub libisoburn mtools
        echo "✓ Arch Linux packages installed"
        ;;
    "fedora")
        sudo dnf groupinstall -y "Development Tools"
        sudo dnf install -y nasm qemu grub2-tools xorriso mtools
        echo "✓ Fedora packages installed"
        ;;
    "macos")
        if ! command -v brew &> /dev/null; then
            echo "Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        brew install nasm qemu i386-elf-grub xorriso
        echo "✓ macOS packages installed"
        ;;
    *)
        echo "⚠️  Unknown Linux distribution. Please install manually:"
        echo "   - build-essential/base-devel"
        echo "   - nasm"
        echo "   - qemu-system-x86_64"
        echo "   - grub tools"
        echo "   - xorriso"
        ;;
esac

# 4. Verify installations
echo ""
echo "4️⃣  Verifying installations..."

check_command() {
    if command -v $1 &> /dev/null; then
        echo "✓ $1 found"
    else
        echo "❌ $1 not found"
        return 1
    fi
}

MISSING=0

check_command rustc || MISSING=1
check_command cargo || MISSING=1
check_command nasm || MISSING=1
check_command qemu-system-x86_64 || MISSING=1

if command -v grub-mkrescue &> /dev/null; then
    echo "✓ grub-mkrescue found"
elif command -v grub2-mkrescue &> /dev/null; then
    echo "✓ grub2-mkrescue found"
else
    echo "❌ grub-mkrescue/grub2-mkrescue not found"
    MISSING=1
fi

check_command xorriso || MISSING=1

# Check Rust target
echo ""
echo "Checking Rust configuration..."
if rustc --print target-list | grep -q "x86_64-unknown-none"; then
    echo "✓ x86_64-unknown-none target available"
else
    echo "❌ x86_64-unknown-none target not available"
    MISSING=1
fi

# Check nightly features
if rustc +nightly --version | grep -q "nightly"; then
    echo "✓ Rust nightly active"
else
    echo "❌ Rust nightly not active"
    MISSING=1
fi

echo ""
if [ $MISSING -eq 0 ]; then
    echo "🎉 All tools installed successfully!"
    echo ""
    echo "You can now build your Rust OS with:"
    echo "  ./build.sh"
    echo "  ./run.sh"
else
    echo "⚠️  Some tools are missing. Please install them manually."
    exit 1
fi

# Add the target
rustup target add x86_64-unknown-none

# Make sure you have rust-src component (needed for build-std)
rustup component add rust-src