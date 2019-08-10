extern crate rand;
extern crate clap;
extern crate num;
extern crate ansi_term;

pub mod grid;
pub mod point;
pub mod rectangle;
pub mod common;
pub mod pipegrid;
pub mod heightgrid;
pub mod roomgrid;

use clap::{App,Arg};
use std::iter::Iterator;

use grid::Grid;
use pipegrid::{PipeGrid,PipeGridProperties};
use heightgrid::{HeightGrid,HeightGridProperties};
use roomgrid::{RoomGrid,RoomGridProperties};


fn main() {
    let argmatches = App::new("mapgen")
        .version("0.1")
        .author("Maarten van Gompel (proycon) <proycon@anaproy.nl>")
        .about("Procedural map generation prototype")
        .arg(Arg::with_name("width")
             .help("width")
             .long("width")
             .short("w")
             .takes_value(true)
             .default_value("80")
        )
        .arg(Arg::with_name("height")
             .help("height")
             .long("height")
             .short("h")
             .takes_value(true)
             .default_value("30")
        )
        .arg(Arg::with_name("seed")
             .help("seed (0 = random seed)")
             .long("seed")
             .short("s")
             .takes_value(true)
             .default_value("0")
        )
        .arg(Arg::with_name("backboneseeds")
             .help("backboneseeds")
             .long("backboneseeds")
             .short("b")
             .takes_value(true)
             .default_value("20")
        )
        .arg(Arg::with_name("regularseeds")
             .help("regularseeds")
             .long("regularseeds")
             .short("r")
             .takes_value(true)
             .default_value("40,40,60")
        )
        .arg(Arg::with_name("interconnect")
             .help("Generate more interconnections between branches, resulting in fewer dead ends")
             .long("interconnect")
             .short("x")
        )
        .arg(Arg::with_name("iterations")
             .help("Iterations in generation (for height map)")
             .long("iterations")
             .short("i")
             .default_value("90")
        )
        .arg(Arg::with_name("rooms")
             .help("Rooms (for room map)")
             .long("rooms")
             .short("R")
             .default_value("6")
        )
        .arg(Arg::with_name("type")
             .help("type")
             .long("type")
             .short("t")
             .takes_value(true)
             .required(true)
             .default_value("pipes")
        )
        .get_matches();

    let mut seed: u64 = argmatches.value_of("seed").unwrap().parse::<u64>().unwrap();
    if seed == 0 {
        seed = rand::random::<u64>();
    }
    let width =  argmatches.value_of("width").unwrap().parse::<usize>().unwrap() as usize;
    let height = argmatches.value_of("height").unwrap().parse::<usize>().unwrap() as usize;
    match argmatches.value_of("type").unwrap() {
        "pipes" => {
            let regularseeds: Option<Vec<&str>>= argmatches.value_of("regularseeds").map(|regularseeds: &str| {
                                    regularseeds.split_terminator(',').collect()
                                });
            let regularseeds: Vec<u16> = regularseeds.unwrap().iter().map(|x:&&str| { x.parse::<u16>().unwrap() } ).collect();
            //using a <Type as Trait> construction: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html
            // to construct the grid
            let grid: Grid<u16,u8> = <Grid<u16,u8> as PipeGrid<u16,u8>>::generate(width as u16,height as u16, seed, PipeGridProperties {
                backboneseeds: argmatches.value_of("backboneseeds").unwrap().parse::<u16>().unwrap() as u16,
                regularseeds: regularseeds,
                interconnect: argmatches.is_present("interconnect"),
            });
            println!("{}", PipeGrid::render(&grid));
        },
        "height" => {
            let grid: Grid<u16,u8> = <Grid<u16,u8> as HeightGrid<u16,u8>>::generate(width as u16, height as u16, seed, HeightGridProperties {
                iterations: argmatches.value_of("iterations").unwrap().parse::<usize>().unwrap() as usize,
            });
            println!("{}", HeightGrid::render(&grid));
        },
        "rooms" => {
            let grid: Grid<u16,u8> = <Grid<u16,u8> as RoomGrid<u16,u8>>::generate(width as u16, height as u16, seed, RoomGridProperties {
                rooms: argmatches.value_of("rooms").unwrap().parse::<usize>().unwrap() as usize,
            });
            println!("{}", RoomGrid::render(&grid));
        },
        _ => {
            eprintln!("No such type");
        }
    }
}
