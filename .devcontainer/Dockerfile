# syntax=docker/dockerfile:1
# Embedded Rust dev container for the STM32L562E-DK (Cortex-M33 + STLINK-V3E)
# Based on EmbeddedRustEssentials/Labs/00-Starters/LAB_3/rustDevL4/docker/Dockerfile
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Base tools, USB access for probe-rs, gdb for Cortex-Debug, serial terminal for the VCP,
# openocd for option bytes (TZEN/RDP) over the ST-LINK
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
      build-essential pkg-config ca-certificates curl git xz-utils \
      libudev-dev libudev1 libusb-1.0-0 usbutils \
      gdb-multiarch picocom openocd && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Rust: minimal profile plus the components the project needs
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
      sh -s -- -y --profile minimal --default-toolchain stable \
         -c rust-src -c rustfmt -c clippy -c llvm-tools
ENV PATH=/root/.cargo/bin:$PATH

# Cortex-M targets (several can be installed side by side)
#   thumbv8m.main-none-eabihf  Cortex-M33 with FPU -> STM32L562E-DK
#   thumbv7em-none-eabihf      Cortex-M4F          -> course labs (STM32L475)
#   thumbv7m-none-eabi         Cortex-M3           -> QEMU lm3s6965evb
RUN rustup target add thumbv8m.main-none-eabihf thumbv7em-none-eabihf thumbv7m-none-eabi

# probe-rs: probe-rs, cargo-flash, cargo-embed (flashing, RTT/defmt, VS Code debug adapter)
RUN curl --proto '=https' --tlsv1.2 -LsSf \
      https://github.com/probe-rs/probe-rs/releases/latest/download/probe-rs-tools-installer.sh | sh

# Prebuilt binaries via cargo-binstall:
#   cargo-binutils   cargo size / cargo objdump / cargo objcopy
#   workshop-runner  `wr`, checks the exercises in 100-exercises-to-learn-rust
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash && \
    cargo binstall -y cargo-binutils && \
    cargo binstall -y --locked workshop-runner

WORKDIR /workspaces
CMD ["/bin/bash"]
