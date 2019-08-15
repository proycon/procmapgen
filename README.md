# Procedural Map Generator

A small toy project written in Rust: procedural generation of 2D grid-based maps with simple terminal-based visualisations

![Video](https://raw.githubusercontent.com/proycon/procmapgen/master/demo.gif)

There are three kinds of graphs, and different styles of visualisation:

* ``Pipe maps`` - an interconnected network of pipes/roads/subways/hallways or whatever you see in it.
    * No isolated subgraphs.
    * Two classes of pipes, a 'backbone' or set of core pipes (thicker) vs 'regular'
    * Simple visualisation (using unicode block drawing) to standard output
* ``Height maps`` - Each cell has a height, good for landscapes. Can also be visualised as a heat map, terrain map.
* ``Room maps`` - Rooms with corridors.


## Screenshots

![Screenshot](https://raw.githubusercontent.com/proycon/procmapgen/master/screenshot.png)

![Screenshot](https://raw.githubusercontent.com/proycon/procmapgen/master/screenshot2.png)

![Screenshot](https://raw.githubusercontent.com/proycon/procmapgen/master/screenshot3.png)

## Build instructions

Assumes you have Rust and Cargo installed:

```
$ git clone https://github.com/proycon/procmapgen
$ cd procmapgen
$ cargo build --release
```

Usage:

```
$ cargo run -- --help
```

It's fun to use this with ``--loop 250`` to see random ones continuously. The number is the amount of milliseconds to
wait between maps:

```
$ cargo run -- --loop 250 --type height
```
