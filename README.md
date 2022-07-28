# TIL -> VHDL
A prototype toolchain for demonstrating and exploring an intermediate representation for defining components using the Tydi interface specification.

This repository defines a grammar (simply called TIL, for Tydi Intermediate Language), and implements a parser, incremental query system, and VHDL backend.

# Demo

To demonstrate the complete prototype toolchain, this repository features a simple command-line application to parse a `.til` file and emit VHDL.

## Requirements

* The Rust compiler and Cargo (https://doc.rust-lang.org/cargo/getting-started/installation.html)

This application was verified in a Linux environment, using `rustc` version `1.61.0`.

## Instructions

1. Switch to the `demo-cmd` directory. (`cd demo-cmd`)
2. Build the application: `cargo build`
3. Run the application with arguments for the input file and the desired output directory. E.g., `cargo run ./til_samples/paper_example.til ./output`

# Note for Anonymous Repository

As anonymous.4open.science does not allow for cloning or otherwise downloading complete repositories, we have included a ZIP archive of the repository at `./til-vhdl-main.zip`