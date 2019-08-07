extern crate rand;
extern crate clap;
extern crate num;

use rand::{SeedableRng,Rng,random};
use rand::prelude::IteratorRandom;
use rand_pcg::Pcg32;
use clap::{App,Arg};

pub type AddressSize = u8;

#[derive(Debug,Default)]
pub struct Properties {
    pub width: AddressSize,
    pub height: AddressSize,
    ///seed for the random number generator that creates the network
    pub networkseed: u64,
    ///initial backbone points
    pub backboneseeds: u16,
    pub regularseeds: Vec<u16>, //multiple iterations
    pub interconnect: bool,
}

#[derive(Debug,Default)]
pub struct GenerationStatus {
    pub players: bool,
    pub nodes: bool,
}


pub struct Grid<ScaleType,ValueType> {
    pub data: Vec<ValueType>,
    pub width: ScaleType,
    pub height: ScaleType,
}

impl<ScaleType,ValueType> Grid<ScaleType,ValueType> where
    ScaleType: num::Integer, ValueType: num::Num {
    pub fn new(width: ScaleType, height: ScaleType) -> Grid<ScaleType,ValueType> {
        //create initial empty 2D grid
        let mut grid: Vec<u8> = Vec::new(); //flattened grid
        for _ in 0..height {
            for _ in 0..width {
                grid.push(0);
            }
        }

        Grid {
            data: grid,
            width: width,
            height: height,
        }
    }

    pub fn index(&self, x: ScaleType, y: ScaleType) -> usize {
       y as usize * self.width as usize + x as usize
    }

    pub fn get(&self, x: ScaleType, y: ScaleType) -> Option<&ValueType> {
        self.data.get(self.index(x,y))
    }

    pub fn get_mut(&self, x: ScaleType, y: ScaleType) -> Option<&mut ValueType> {
        self.data.get_mut(self.index(x,y))
    }

    pub fn set(&self, x: ScaleType, y: ScaleType, value: ValueType) -> bool {
        if let Some(mut v) = self.data.get_mut(x,y) {
            *v = value;
            return true;
        }
        return false;
    }

    pub fn hasneighbours(&self,x: ScaleType, y: ScaleType) -> (bool, bool, bool, bool) {
       let hasnorth = y > 0 && self.get(x,y-1) > 0;
       let haseast = x < self.width - 1 && self.get(x+1,y) > 0 ;
       let hassouth = y < self.height - 1 && self.get(x,y+1) > 0;
       let haswest = x > 0 && self.get(x-1,y) > 0;
       (hasnorth, haseast, hassouth, haswest)
    }

    pub fn countneighbours(&self, x: ScaleType, y: ScaleType) -> usize {
       let (hasnorth, haseast, hassouth, haswest) = self.hasneighbours(x, y);
       let mut count = 0;
       if hasnorth { count += 1 }
       if haseast { count += 1 }
       if hassouth { count += 1 }
       if haswest { count += 1 }
       count
    }

}




pub fn getgridindex(properties: &Properties, x: AddressSize,y: AddressSize) -> usize {
   y as usize * properties.width as usize + x as usize
}


///Generates the network (a planar graph), with a backbone
pub fn generate_grid(properties: &Properties) -> Vec<u8> {
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
            randompathto(properties, &mut grid, *x, *y, to_x, to_y, 2, &mut rng);
        }
    }

    //Add regular nodes (multiple iterations of a specific amount of seeds)
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
                //draw a random path to the closest backbone
                if let Some((to_x,to_y)) = closest {
                    randompathto(properties, &mut grid, x, y, to_x, to_y, height+1, &mut rng);
                }
            }
        }
    }

    if properties.interconnect {
        //prune dead ends by creating more interconnections
        let mut deadends: Vec<(AddressSize,AddressSize)> = Vec::new();
        let mut processed: Vec<usize> = Vec::new();
        //find all dead ends
        for y in 0..properties.height {
            for x in 0..properties.width {
               //a dead end has only one neighbour
               if grid[getgridindex(properties,x,y)] > 2 && countneighbours(properties, &grid, x, y) == 1 {
                   deadends.push((x,y));
               }
            }
        }

        for (i, (x,y)) in deadends.iter().enumerate() {
          if !processed.contains(&i) {
            //we find the closest other dead end (or former dead end)
            let mut mindistance: Option<f64> = None;
            let mut closest: Option<usize> = None;
            for (j, (x2,y2)) in deadends.iter().enumerate() {
                if i != j {
                    let distx: f64 = (*x2 as f64 - *x as f64).abs();
                    let disty: f64 = (*y2 as f64 - *y as f64).abs();
                    let distance: f64 = (distx.powf(2.0) + disty.powf(2.0)).sqrt();
                    if mindistance.is_none() || distance < mindistance.unwrap() {
                        mindistance = Some(distance);
                        closest = Some(j);
                    }
                }
            }
            //draw a random path to the closest (former) dead end
            if let Some(closest) = closest {
                let (to_x,to_y) = deadends[closest];
                randompathto(properties, &mut grid, *x, *y, to_x, to_y, 99, &mut rng);
                processed.push(closest);
            }
          }
        }
    }
    grid
}

pub fn randompathto(properties: &Properties, grid: &mut Vec<u8>, x: AddressSize, y: AddressSize, to_x: AddressSize, to_y: AddressSize, height: u8, rng: &mut Pcg32) {
    let mut retry = true;
    let mut retries = 0;
    while retry {
        let mut from_x = x;
        let mut from_y = y;
        let dx: i32 = if to_x > from_x { 1 } else { -1 };
        let dy: i32 = if to_y > from_y { 1 } else { -1 };
        let mut iteration = 0;
        retry = false;
        while (from_x != to_x) || (from_y != to_y) {
            if (from_x != x) || (from_y != y) {
                if grid[getgridindex(properties, from_x,from_y)] == 0 {
                    grid[getgridindex(properties, from_x,from_y)] = height;
                } else if iteration == 1 && retries < 5 {
                    //first step must be to a node that is still empty, restart:
                    retry = true;
                    retries += 1;
                    break;
                }
            }
            if (from_x != to_x) && ((from_y == to_y) || rng.gen()) {
                from_x = ((from_x as i32) + dx) as AddressSize;
            } else if (from_y != to_y) && ((from_x == to_x) || rng.gen()) {
                from_y = ((from_y as i32) + dy) as AddressSize;
            }
            iteration += 1;
        }
    }
}


pub fn hasneighbours(properties: &Properties, grid: &Vec<u8>, x: AddressSize, y: AddressSize) -> (bool, bool, bool, bool) {
   let hasnorth = y > 0 && grid[getgridindex(properties, x, y-1)] > 0;
   let haseast = x < properties.width - 1 && grid[getgridindex(properties, x+1, y)] > 0 ;
   let hassouth = y < properties.height - 1 && grid[getgridindex(properties,x,y+1)] > 0;
   let haswest = x > 0 && grid[getgridindex(properties, x-1, y)] > 0;
   (hasnorth, haseast, hassouth, haswest)
}

pub fn countneighbours(properties: &Properties, grid: &Vec<u8>, x: AddressSize, y: AddressSize) -> usize {
   let (hasnorth, haseast, hassouth, haswest) = hasneighbours(properties, grid, x, y);
   let mut count = 0;
   if hasnorth { count += 1 }
   if haseast { count += 1 }
   if hassouth { count += 1 }
   if haswest { count += 1 }
   count
}


pub fn printgrid(properties: &Properties, grid: &Vec<u8>, debug: bool) {
    for y in 0..properties.height {
        for x in 0..properties.width {
            let index = getgridindex(properties,x,y);
            let chr: char = match grid[index] {
                0 => ' ',
                _ => {
                   if debug {
                       grid[index] as char
                   } else {
                       let (hasnorth, haseast, hassouth, haswest) = hasneighbours(properties, grid, x, y);
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
        .arg(Arg::with_name("interconnect")
             .help("Generate more interconnections between branches, resulting in fewer dead ends")
             .long("interconnect")
             .short("i")
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
    let properties = Properties {
        width: argmatches.value_of("width").unwrap().parse::<AddressSize>().unwrap() as AddressSize,
        height: argmatches.value_of("height").unwrap().parse::<AddressSize>().unwrap() as AddressSize,
        networkseed: seed,
        backboneseeds: argmatches.value_of("backboneseeds").unwrap().parse::<u16>().unwrap() as u16,
        regularseeds: regularseeds,
        interconnect: argmatches.is_present("interconnect"),
    };
    let grid = generate_grid(&properties);
    printgrid(&properties, &grid, false);
}
