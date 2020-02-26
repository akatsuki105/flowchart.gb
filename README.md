# flowchart.gb

flowchart.gb is a flowchart generator for GameBoy binary.

Given a GameBoy ROM file or RGBASM file, a DSL file for [flowchart.js](https://flowchart.js.org/) will be generated.

<img src="https://imgur.com/BHgJtfR.png" width="600px">

## requirements

- Rust
- Python3 (To disassemble a .gb file)

## usage

When you execute cargo run, a file dialog will appear, so select the target file there.  
The output DSL file is for [flowchart.js](https://flowchart.js.org/). Please see [flowchart.js](https://flowchart.js.org/) for details.

```sh
# for gb file
cargo run

# for asm file
cargo run INIT_LABEL
```

## usage for examples

#### examples/hello

This is rgbasm source file, so you need to input init label "start".

```sh
cargo run start # start: is ok.
```

#### examples/picture

This is gameboy ROM file, so you don't need to input init label.

```sh
cargo run
```

## Warning

1. The flowchart may be interrupted due to bank switch. In that case, execute the command again with the label at the break point as the init label.
2. [mgbdis](https://github.com/mattcurrie/mgbdis) is used for disassemble of gb file. Thank you!
