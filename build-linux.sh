#!/bin/bash

# ========================================
# blkops Android Cross-compilation script
# ========================================

NDK_PATH="${ANDROID_NDK_HOME:-$HOME/Android/Sdk/ndk/25.2.9519653}"
TARGET_API="28"
TARGET_ARCH="aarch64-linux-android"
BUILD_TYPE="release"

# Color out
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check NDK path
check_ndk() {
    if [ ! -d "$NDK_PATH" ]; then
        print_error "NDK path not found: $NDK_PATH"
        print_info "Please set ANDROID_NDK_HOME environment variable or edit NDK_PATH in script"
        exit 1
    fi
    
    local clang_path="$NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/bin/${TARGET_ARCH}${TARGET_API}-clang++"
    if [ ! -f "$clang_path" ]; then
        print_error "Cannot found compiler: $clang_path"
        exit 1
    fi
    
    print_info "NDK path: $NDK_PATH"
    print_info "Compiler: $clang_path"
}

# Check Rust environment
check_rust() {
    if ! command -v rustc &> /dev/null; then
        print_error "Rust not install,Please install Rust"
        print_info "Install command: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    print_info "Rust version: $(rustc --version)"
}

# Add Rust target
add_rust_target() {
    print_info "Check Rust target..."
    if ! rustup target list | grep -q "installed.*$TARGET_ARCH"; then
        print_info "Add $TARGET_ARCH target..."
        rustup target add $TARGET_ARCH
    else
        print_info "$TARGET_ARCH target is installed"
    fi
}

# Set environment variable
setup_environment() {
    local clang_path="$NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/bin/${TARGET_ARCH}${TARGET_API}-clang++"
    
    export CC="$clang_path"
    export CXX="$clang_path"
    export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$clang_path"
    export ANDROID_NDK_HOME="$NDK_PATH"
    
    print_info "Environment variable is setted:"
    echo "  CC=$CC"
    echo "  CXX=$CXX"
    echo "  CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"
}

# Clean old compile
clean_build() {
    if [ -d "target" ]; then
        print_warning "Clean old compile..."
        rm -rf target
    fi
}

# Compile project
build_project() {
    print_info "Start compile..."
    
    if cargo build --$BUILD_TYPE --target $TARGET_ARCH; then
        local output_path="target/$TARGET_ARCH/$BUILD_TYPE/blkops"
        if [ -f "$output_path" ]; then
            print_success "Compile success!"
            print_info "Output file: $output_path"
            
            # Show file info
            ls -lh "$output_path"
            
            cp "$output_path" ./blkops-android-arm64
            print_info "Copied as: ./blkops-android-arm64"
            
            # Show file type
            file "$output_path"
        else
            print_error "Cannot found output file!"
            exit 1
        fi
    else
        print_error "Compile failed!"
        exit 1
    fi
}

# Test compilation result
test_binary() {
    print_info "In test binary file..."
    
    # 检查是否为静态链接
    if ldd "target/$TARGET_ARCH/$BUILD_TYPE/blkops" 2>&1 | grep -q "not a dynamic executable"; then
        print_success "The binary file is statically linked"
    else
        print_warning "The binary file may not be statically linked"
    fi
    
    # 检查架构
    if file "target/$TARGET_ARCH/$BUILD_TYPE/blkops" | grep -q "ARM aarch64"; then
        print_success "The binary file architecture is aarch64"
    else
        print_warning "The binary file architecture may not be aarch64"
    fi
}

# 主函数
main() {
    echo ""
    echo "========================================="
    echo " blkops Android Cross-compilation script"
    echo "========================================="
    echo ""
    
    check_ndk
    check_rust
    add_rust_target
    setup_environment
    clean_build
    build_project
    test_binary
    
    echo ""
    echo "========================================"
    echo "  Compile end"
    echo "========================================"
    echo ""
    print_info "Usage:"
    echo "  adb push blkops-android-arm64 /data/local/tmp/"
    echo "  adb shell chmod +x /data/local/tmp/blkops-android-arm64"
    echo "  adb shell su -c \"/data/local/tmp/blkops-android-arm64 -s boot\""
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --ndk-path)
            NDK_PATH="$2"
            shift 2
            ;;
        --target-api)
            TARGET_API="$2"
            shift 2
            ;;
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        *)
            print_warning "Unknow function: $1"
            shift
            ;;
    esac
done

main