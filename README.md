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

## Linked Implementations

The intermediate representation intentionally omits expressions for implementing or simulating arbitrary behavior of components. Instead, "behavioral implementations" in the IR exist only as *links* to directories, which contain the relevant code in languages more suited for expressing behavior.

In the case of this prototype toolchain, the backend expects a single VHDL file matching the name of the component that would be generated, containing an entity and architecture description. Multiple linked implementations can point to the same directory. If no such file exists, but the directory does, the backend will automatically generate an empty architecture for that entity at that location. (This is the recommended way of working - first generate the "template" files, then implement the component based on the template.)

Links must be relative paths, to ensure projects are not bound to a single environment.

When using a single TIL file through the `demo-cmd` tool, the relative paths defined in the TIL file are relative to the working directory (i.e., `.`). To get more control over how paths are interpreted, you should use a project instead.

## Projects

Projects are expressed in simple `*.toml` files, and may contain a list of multiple TIL files (as paths relative to the project file) and various configuration items.

This is a sample config file, as seen [here](/demo-cmd/til_samples/alt_link_example):
```toml
name = "alt_link"

files = [ "src/alt_link.til" ]

output_path = "../../../test_output"

[config]
link_relative_to_file = true
```

The main (required) configuration items are:
* **name**: The name of the project, used for the backend's output and for (currently unsupported by the parser) imports between projects.
* **files**: The TIL files that the project consists of, as an array of paths relative to the project file. They will be parsed in the order they're declared.
* **output_path**: The output directory of the backend, as a path relative to the project file.

Additional, optional configuration items are part of the `[config]` subsection. There is currently only one additional config item:
* **link_relative_to_file**: Defines how *links* should be interpreted. **By default, linked implementation paths are relative to the project file**, setting this config item to `true`  makes it so linked implementation paths are relative to the TIL file they're defined in.
