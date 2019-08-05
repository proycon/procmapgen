# Pipe Map Generator

A small toy project written in Rust: procedural generation of an interconnected network of pipes/roads/subways/hallways or whatever you see in it.

Properties:
* No isolated subgraphs.
* Two classes of pipes, a 'backbone' or set of core pipes (thicker) vs 'regular'
* Simple visualisation (using unicode block drawing) to standard output

![Screenshot](https://raw.githubusercontent.com/proycon/pipemapgen/master/screenshot.png)

## Build instructions

Assumes you have Rust and Cargo installed:

```
$ git clone https://github.com/proycon/pipemapgen
$ cd pipemapgen
$ cargo build --release
```

Usage:

```
$ ./target/release/pipemapgen --help
```
