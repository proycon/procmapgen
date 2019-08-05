extern crate rand;
extern crate clap;

use rand::{SeedableRng,Rng,random};
use rand::prelude::IteratorRandom;
use rand_pcg::Pcg32;
use clap::{App,Arg};

pub type AddressSize = u8;

#[derive(Debug,Default)]
pub struct WorldProperties {
    pub width: AddressSize,
    pub height: AddressSize,
    ///seed for the random number generator that creates the network
    pub networkseed: u64,
    ///initial backbone points
    pub backboneseeds: u16,
    pub regularseeds: Vec<u16> //multiple iterations
}

#[derive(Debug,Default)]
pub struct GenerationStatus {
    pub players: bool,
    pub nodes: bool,
}
pub fn getgridindex(properties: &WorldProperties, x: AddressSize,y: AddressSize) -> usize {
   y as usize * properties.width as usize + x as usize
}


///Generates the network (a planar graph), with a backbone
pub fn generate_grid(properties: &WorldProperties) -> Vec<u8> {
    let mut rng = Pcg32::seed_from_u64(properties.networkseed);
    let mut grid: Vec<u8> = Vec::new(); //flattened grid

    //create initial empty 2D grid
    for _ in 0..properties.height {
        for _ in 0..properties.width {
            grid.push(0);
        }
    }

    let mut backboneseeds: Vec<(AddressSize,AddressSize)> = Vec::new();
    //add initial backbone nodes
    for _ in 0..properties.backboneseeds {
        let x: AddressSize = rng.gen_range(0,properties.width);
        let y: AddressSize = rng.gen_range(0,properties.height);
        let index = getgridindex(properties, x,y);
        grid[index] = 1;
        backboneseeds.push((x,y));
    }

    let mut processed: Vec<usize> = Vec::new();
    //connect each backnode node to the nearest other
    for (i, (x,y)) in backboneseeds.iter().enumerate() {
        processed.push(i);

        //find the nearest unconnected other
        let mut mindistance: Option<f64> = None;
        let mut closest: Option<usize> = None;
        for (j, (x2,y2)) in backboneseeds.iter().enumerate() {
            if !processed.contains(&j) {
                let distx: f64 = (*x2 as f64 - *x as f64).abs();
                let disty: f64 = (*y2 as f64 - *y as f64).abs();
                let distance: f64 = (distx.powf(2.0) + disty.powf(2.0)).sqrt();
                if mindistance.is_none() || distance < mindistance.unwrap() {
                    mindistance = Some(distance);
                    closest = Some(j);
                }
            }
        }

        //draw a random path
        if let Some(closest) = closest {
            let (to_x,to_y) = backboneseeds[closest];
            let mut from_x = *x;
            let mut from_y = *y;
            let dx: i32 = if to_x > from_x { 1 } else { -1 };
            let dy: i32 = if to_y > from_y { 1 } else { -1 };
            while (from_x != to_x) || (from_y != to_y) {
                if (from_x != *x) || (from_y != *y) {
                    grid[getgridindex(properties, from_x,from_y)] = 2;
                }
                if (from_x != to_x) && ((from_y == to_y) || rng.gen()) {
                    from_x = ((from_x as i32) + dx) as AddressSize;
                } else if (from_y != to_y) && ((from_x == to_x) || rng.gen()) {
                    from_y = ((from_y as i32) + dy) as AddressSize;
                }
            }
        }
    }

    //Add regular nodes
    for (iternr, regularseedgoal) in properties.regularseeds.iter().enumerate() {
        let mut regularseeds = 0;
        let height: u8 = iternr as u8 + 3;
        while regularseeds < *regularseedgoal {
            let x: AddressSize = rng.gen_range(0,properties.width);
            let y: AddressSize = rng.gen_range(0,properties.height);
            let index = getgridindex(properties, x,y);
            if grid[index] == 0 {
                regularseeds += 1;
                grid[index] = height;
                //find the closest backbone
                let mut mindistance: Option<f64> = None;
                let mut closest: Option<(AddressSize,AddressSize)> = None;
                for y2 in 0..properties.height {
                    for x2 in 0..properties.width {
                        let index = getgridindex(properties, x2,y2);
                        if grid[index] > 0 && grid[index] < height {
                            let distx: f64 = (x2 as f64 - x as f64).abs();
                            let disty: f64 = (y2 as f64 - y as f64).abs();
                            let distance: f64 = (distx.powf(2.0) + disty.powf(2.0)).sqrt();
                            if mindistance.is_none() || distance < mindistance.unwrap() {
                                mindistance = Some(distance);
                                closest = Some((x2,y2));
                            }
                        }
                    }
                }
                //
                //draw a random path
                if let Some((to_x,to_y)) = closest {
                    let mut from_x = x;
                    let mut from_y = y;
                    let dx: i32 = if to_x > from_x { 1 } else { -1 };
                    let dy: i32 = if to_y > from_y { 1 } else { -1 };
                    while (from_x != to_x) || (from_y != to_y) {
                        if (from_x != x) || (from_y != y) {
                            grid[getgridindex(properties, from_x,from_y)] = height +1;
                        }
                        if (from_x != to_x) && ((from_y == to_y) || rng.gen()) {
                            from_x = ((from_x as i32) + dx) as AddressSize;
                        } else if (from_y != to_y) && ((from_x == to_x) || rng.gen()) {
                            from_y = ((from_y as i32) + dy) as AddressSize;
                        }
                    }
                }
            }
        }
    }

    grid
}


pub fn printgrid(properties: &WorldProperties, grid: &Vec<u8>, debug: bool) {
    for y in 0..properties.height {
        for x in 0..properties.width {
            let index = getgridindex(properties,x,y);
            let chr: char = match grid[index] {
                0 => ' ',
                _ => {
                   if debug {
                       grid[index] as char
                   } else {
                       let haswest = x > 0 && grid[getgridindex(properties, x-1, y)] > 0;
                       let hasnorth = y > 0 && grid[getgridindex(properties, x, y-1)] > 0;
                       let haseast = x < properties.width - 1 && grid[getgridindex(properties, x+1, y)] > 0 ;
                       let hassouth = y < properties.height - 1 && grid[getgridindex(properties,x,y+1)] > 0;
                       let isbackbone = grid[index] <= 2;
                       getnodeglyph(hasnorth, haseast, hassouth, haswest, isbackbone)

                   }
                }
            };
            print!("{}",chr);
        }
        println!("");
    }
}

pub fn getnodeglyph(hasnorth:bool, haseast:bool, hassouth:bool, haswest:bool, isbackbone:bool) -> char {
   match (hasnorth, haseast, hassouth, haswest, isbackbone) {
       (true,true,true,true, false) => '┼',
       (true,true,true,true, true) => '╋',
       (true,true,true,false, false) => '├',
       (true,true,true,false, true) => '┣',
       (false,true,true,true, false) => '┬',
       (false,true,true,true, true) => '┳',
       (true,false,true,true, false) => '┤',
       (true,false,true,true, true) => '┫',
       (true,true,false,true, false) => '┴',
       (true,true,false,true, true) => '┻',
       (true,true,false,false, false) => '└',
       (true,true,false,false, true) => '┗',
       (true,false,true,false, false) => '│',
       (true,false,true,false, true) => '┃',
       (true,false,false,true, false) => '┘',
       (true,false,false,true, true) => '┛',
       (false,true,true,false, false) => '┌',
       (false,true,true,false, true) => '┏',
       (false,true,false,true, false) => '─',
       (false,true,false,true, true) => '━',
       (false,false,true,true, false) => '┐',
       (false,false,true,true, true) => '┓',
       (true,false,false,false, false) => '╵',
       (true,false,false,false, true) => '╹',
       (false,true,false,false, false) => '╶',
       (false,true,false,false, true) => '╺',
       (false,false,true,false, false) => '╷',
       (false,false,true,false, true) => '╻',
       (false,false,false,true, false) => '╴',
       (false,false,false,true, true) => '╸',
       _ => '?',
   }
}

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
        .get_matches();

    let regularseeds: Option<Vec<&str>>= argmatches.value_of("regularseeds").map(|regularseeds: &str| {
                                    regularseeds.split_terminator(',').collect()
                                });
    let mut seed: u64 = argmatches.value_of("seed").unwrap().parse::<u64>().unwrap();
    if seed == 0 {
        seed = rand::random::<u64>();
    }
    let regularseeds: Vec<u16> = regularseeds.unwrap().iter().map(|x:&&str| { x.parse::<u16>().unwrap() } ).collect();
    let worldproperties = WorldProperties {
        width: argmatches.value_of("width").unwrap().parse::<AddressSize>().unwrap() as AddressSize,
        height: argmatches.value_of("height").unwrap().parse::<AddressSize>().unwrap() as AddressSize,
        networkseed: seed,
        backboneseeds: argmatches.value_of("backboneseeds").unwrap().parse::<u16>().unwrap() as u16,
        regularseeds: regularseeds,
    };
    let grid = generate_grid(&worldproperties);
    printgrid(&worldproperties, &grid, false);
}
