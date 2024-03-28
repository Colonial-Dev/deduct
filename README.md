<p align="center">
<!-- <img src=".github/logo.png" width="512"> -->
</p>
<h3 align="center">A Fitch-style natural deduction proof checker, with support for modal logic.</h3>

<p align="center">
<img src="https://img.shields.io/github/actions/workflow/status/Colonial-Dev/deduct/rust.yml">
<img src="https://img.shields.io/github/license/Colonial-Dev/deduct">
<img src="https://img.shields.io/github/stars/Colonial-Dev/deduct">
</p>

## Features
- Support for multiple proof systems
  - TFL (basic and derived rulesets)
  - Modal logic (systems $K$, $T$, $S_4$, and $S_5$)
- Cross-platform thanks to `egui`; runs on all major operating systems and in the browser

## Installation

### Precompiled Binaries
Precompiled versions of Deduct are available for:
- Windows
- macOS
- Linux (compiled against `x86_64-unknown-linux-musl`, which should Just Work™ on most distributions.) 
- WebAssembly (runs in your browser.)

All binaries can be found in the [releases](https://github.com/Colonial-Dev/deduct/releases) section.

### From Source

Dependencies:
- The [Rust programming language](https://rustup.rs/).
- A C/C++ toolchain (such as `gcc`.)

Just use `cargo install`, and Deduct will be compiled and added to your `PATH`.
```sh
cargo install --locked --git https://github.com/Colonial-Dev/deduct --branch master
```

## Getting Started

## Design

## Acknowledgements