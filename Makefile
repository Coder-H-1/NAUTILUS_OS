# New Makefile for Rust bare-metal OS
KERNEL := kernel8.img
TARGET := aarch64-unknown-none
BUILD_DIR := target/$(TARGET)/release
CRATE_NAME := rpi_os

.PHONY: all clean FORCE

all: $(KERNEL)

$(KERNEL): FORCE
	cargo build --release
	# Requires llvm-tools-preview and cargo-binutils (cargo install cargo-binutils)
	cargo objcopy --release -- -O binary $(KERNEL)

clean:
	cargo clean
	cmd /c del /f /q $(KERNEL) || rm -f $(KERNEL)
	exit