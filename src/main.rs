extern crate rand;
extern crate clap;
extern crate num;
extern crate ansi_term;

use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use clap::{App,Arg};
use num::{Integer,Num,FromPrimitive,ToPrimitive,range};
use std::ops::Index;
use std::cmp::min;
use ansi_term::Colour::{White,RGB};


///The basic grid type
pub struct Grid<ScaleType,ValueType> {
    ///A flattened vector
    data: Vec<ValueType>,

    ///The dimensions of the grid
    size: (ScaleType,ScaleType),
}

#[derive(Debug,Default)]
pub struct PipeGridProperties {
    ///initial backbone points
    pub backboneseeds: u16,

    ///amount of regular seeds to place, each element corresponds to an iteration
    pub regularseeds: Vec<u16>,

    ///prune dead-ends to a large extent by interconnecting them
    pub interconnect: bool,
}

#[derive(Debug,Default)]
pub struct HeightGridProperties {
    ///number of iterations
    pub iterations: usize,
}

pub struct RoomGridProperties {
    pub rooms: usize,
}

trait PipeGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: PipeGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, x: ScaleType, y: ScaleType) -> String;

}

trait HeightGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: HeightGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, x: ScaleType, y: ScaleType, min: ValueType, max: ValueType) -> String;
}

trait RoomGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: RoomGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, x: ScaleType, y: ScaleType) -> String;
}


impl<ScaleType,ValueType> Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + PartialOrd + PartialEq + Copy ,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    pub fn new(width: ScaleType, height: ScaleType) -> Grid<ScaleType,ValueType> {
        //create initial empty 2D grid
        let mut grid: Vec<ValueType> = Vec::new(); //flattened grid
        for _ in range(ScaleType::zero(), height) {
            for _ in range(ScaleType::zero(), width) {
                grid.push(ValueType::zero());
            }
        }

        Grid {
            data: grid,
            size: (width, height),
        }
    }

    pub fn width(&self) -> ScaleType {
        self.size.0
    }

    pub fn height(&self) -> ScaleType {
        self.size.1
    }

    pub fn width_as_usize(&self) -> usize {
        self.size.0.to_usize().unwrap()
    }

    pub fn height_as_usize(&self) -> usize {
        self.size.1.to_usize().unwrap()
    }


    pub fn max(&self) -> ValueType {
        let mut largest: Option<ValueType> = None;
        for v in self.data.iter() {
            if largest.is_none() ||  largest.unwrap() < *v {
                largest = Some(*v);
            }
        }
        largest.expect("Grid has no data")
    }

    pub fn min(&self) -> ValueType {
        let mut smallest: Option<ValueType> = None;
        for v in self.data.iter() {
            if smallest.is_none() ||  smallest.unwrap() > *v {
                smallest = Some(*v);
            }
        }
        smallest.expect("Grid has no data")
    }

    pub fn index(&self, x: ScaleType, y: ScaleType) -> usize {
       (y * self.width() + x).to_usize().expect("Unable to cast to usize")
    }

    pub fn get(&self, x: ScaleType, y: ScaleType) -> Option<&ValueType> {
        self.data.get(self.index(x,y))
    }

    pub fn get_mut(&mut self, x: ScaleType, y: ScaleType) -> Option<&mut ValueType> {
        let index = self.index(x,y);
        self.data.get_mut(index)
    }

    pub fn inc(&mut self, x: ScaleType, y: ScaleType, amount: ValueType) {
        let index = self.index(x,y);
        let mut v = self.data.get_mut(index).unwrap();
        //TODO: do overflow checking
        *v = *v + amount;
    }

    pub fn set(&mut self, x: ScaleType, y: ScaleType, value: ValueType) -> bool {
        if let Some(v) = self.get_mut(x,y) {
            *v = value;
            return true;
        }
        return false;
    }

    pub fn hasneighbours(&self,x: ScaleType, y: ScaleType) -> (bool, bool, bool, bool) {
       let hasnorth = y > ScaleType::zero() && self[(x,y-ScaleType::one())] > ValueType::zero();
       let haseast = x < self.width() - ScaleType::one() && self[(x+ScaleType::one(),y)] > ValueType::zero();
       let hassouth = y < self.height() - ScaleType::one() && self[(x,y+ScaleType::one())] > ValueType::zero();
       let haswest = x > ScaleType::zero() && self[(x-ScaleType::one(),y)] > ValueType::zero();
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

    ///A simple euclidian distance function
    pub fn distance(&self, x: ScaleType, y: ScaleType, x2: ScaleType, y2: ScaleType) -> f64 {
        //self is passed (though unused) just so we don't need type annotations
        let x = x.to_f64().unwrap();
        let y = y.to_f64().unwrap();
        let x2 = x2.to_f64().unwrap();
        let y2 = y2.to_f64().unwrap();
        let distx: f64 = (x2 - x).abs();
        let disty: f64 = (y2 - y).abs();
        let distance: f64 = (distx.powf(2.0) + disty.powf(2.0)).sqrt();
        distance
    }


    ///Computes the distance between two boxes, the distance is the shortest distance between a
    ///corner of box A and a corner of box B
    pub fn boxdistance(&self, left: ScaleType, top: ScaleType, width: ScaleType, height: ScaleType, left2: ScaleType, top2: ScaleType, width2: ScaleType, height2: ScaleType) -> f64 {
        let mut d: f64 = self.distance(left,top,left2+width2,top2);
        d = fmin(d, self.distance(left,top,left2+width2,top2+height2) );
        d = fmin(d, self.distance(left,top,left2,top2+height2) );

        d = fmin(d, self.distance(left+width,top,left2,top2) );
        d = fmin(d, self.distance(left+width,top,left2,top2+height2) );
        d = fmin(d, self.distance(left+width,top,left2+width2,top2+height2) );

        d = fmin(d, self.distance(left+width,top+height,left2,top2) );
        d = fmin(d, self.distance(left+width,top+height,left2,top2+height2) );
        d = fmin(d, self.distance(left+width,top+height,left2+width2,top2) );

        d = fmin(d, self.distance(left,top+height,left2,top2) );
        d = fmin(d, self.distance(left,top+height,left2+width,top2+height2) );
        d = fmin(d, self.distance(left,top+height,left2+width2,top2) );

        d
    }

    pub fn randompathto(&mut self, rng: &mut Pcg32, x: ScaleType, y: ScaleType, to_x: ScaleType, to_y: ScaleType, height: ValueType) {
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
                    if self[(from_x,from_y)] == ValueType::zero() {
                        self.set(from_x,from_y,height);
                    } else if iteration == 1 && retries < 5 {
                        //first step must be to a node that is still empty, restart:
                        retry = true;
                        retries += 1;
                        break;
                    }
                }
                if (from_x != to_x) && ((from_y == to_y) || rng.gen()) {
                    from_x = ScaleType::from_i32((from_x.to_i32().expect("conversion problem")) + dx).expect("conversion problem");
                } else if (from_y != to_y) && ((from_x == to_x) || rng.gen()) {
                    from_y = ScaleType::from_i32((from_y.to_i32().expect("conversion problem")) + dy).expect("conversion problem");
                }
                iteration += 1;
            }
        }
    }

}

///Implementing the index ([]) operator for Grid
impl<ScaleType,ValueType> Index<(ScaleType,ScaleType)> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

        type Output = ValueType;

        fn index(&self, index: (ScaleType, ScaleType)) -> &Self::Output {
            let (x,y) = index;
            self.get(x,y).expect("Out of bounds")
        }

}


impl<ScaleType,ValueType> PipeGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    ///Generates the network (a planar graph), with a backbone
    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: PipeGridProperties) -> Grid<ScaleType,ValueType> {
        let mut rng = Pcg32::seed_from_u64(seed);
        let mut grid: Grid<ScaleType,ValueType> = Grid::new(width, height);

        let mut backboneseeds: Vec<(ScaleType,ScaleType)> = Vec::new();
        //add initial backbone nodes
        for _ in 0..properties.backboneseeds {
            let x: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.width_as_usize())).unwrap();
            let y: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.height_as_usize())).unwrap();
            grid.set(x,y, ValueType::one());
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
                    let distance =  grid.distance(*x,*y,*x2,*y2);
                    if mindistance.is_none() || distance < mindistance.unwrap() {
                        mindistance = Some(distance);
                        closest = Some(j);
                    }
                }
            }

            //draw a random path
            if let Some(closest) = closest {
                let (to_x,to_y) = backboneseeds[closest];
                grid.randompathto(&mut rng, *x, *y, to_x, to_y, ValueType::from_u8(2).unwrap());
            }
        }

        //Add regular nodes (multiple iterations of a specific amount of seeds)
        for (iternr, regularseedgoal) in properties.regularseeds.iter().enumerate() {
            let mut regularseeds = 0;
            let height: ValueType = ValueType::from_usize(iternr).expect("Conversion error") + ValueType::from_u8(3).unwrap();
            while regularseeds < *regularseedgoal {
                let x: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.width_as_usize())).unwrap();
                let y: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.height_as_usize())).unwrap();
                if grid[(x,y)] == ValueType::zero() {
                    regularseeds += 1;
                    grid.set(x,y,height);
                    //find the closest backbone
                    let mut mindistance: Option<f64> = None;
                    let mut closest: Option<(ScaleType,ScaleType)> = None;
                    for y2 in range(ScaleType::zero(), grid.height()) {
                        for x2 in range(ScaleType::zero(), grid.width()) {
                            let v = grid[(x2,y2)];
                            if v > ValueType::zero() && v < height {
                                let distance: f64 = grid.distance(x,y,x2,y2);
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
                        grid.randompathto(&mut rng, x, y, to_x, to_y, height+ValueType::one());
                    }
                }
            }
        }

        if properties.interconnect {
            //prune dead ends by creating more interconnections
            let mut deadends: Vec<(ScaleType,ScaleType)> = Vec::new();
            let mut processed: Vec<usize> = Vec::new();
            //find all dead ends
            for y in range(ScaleType::zero(), grid.height()) {
                for x in range(ScaleType::zero(), grid.width()) {
                   //a dead end has only one neighbour
                   if grid[(x,y)] > ValueType::from_u8(2).unwrap() && grid.countneighbours(x, y) == 1 {
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
                        let distance: f64 = grid.distance(*x,*y,*x2,*y2);
                        if mindistance.is_none() || distance < mindistance.unwrap() {
                            mindistance = Some(distance);
                            closest = Some(j);
                        }
                    }
                }
                //draw a random path to the closest (former) dead end
                if let Some(closest) = closest {
                    let (to_x,to_y) = deadends[closest];
                    grid.randompathto(&mut rng, *x, *y, to_x, to_y, ValueType::from_u8(99).unwrap());
                    processed.push(closest);
                }
              }
            }
        }
        grid
    }

    fn render(&self) -> String {
        let mut output: String = String::new();
        for y in range(ScaleType::zero(), self.height()) {
            for x in range(ScaleType::zero(), self.width()) {
                output += PipeGrid::rendercell(self, x,y).as_str();
            }
            output.push('\n');
        }
        output
    }

    fn rendercell(&self, x: ScaleType, y: ScaleType) -> String {
        let v = self[(x,y)];
        let chr: char = if v == ValueType::zero() {
            ' '
        } else {
           let (hasnorth, haseast, hassouth, haswest) = self.hasneighbours(x, y);
           let isbackbone = self[(x,y)] <= ValueType::from_u8(2).unwrap();
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
        };
        chr.to_string()
    }

}


impl<ScaleType,ValueType> HeightGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    ///Generates the network (a planar graph), with a backbone
    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: HeightGridProperties) -> Grid<ScaleType,ValueType> {
        let mut rng = Pcg32::seed_from_u64(seed);
        let mut grid: Grid<ScaleType,ValueType> = Grid::new(width,height);
        for i in 0..properties.iterations {
            let width: ScaleType = ScaleType::from_usize(rng.gen_range(1,grid.width_as_usize() / 5)).expect("Unable to compute width");
            let height: ScaleType = ScaleType::from_usize(rng.gen_range(1,grid.height_as_usize() / 5)).expect("Unable to compute height");
            let left: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.width_as_usize())).expect("Unable to compute left");
            let top: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.height_as_usize())).expect("Unable to compute top");
            for y in range(top, min(top + height, grid.height())) {
                for x in range(left, min(left + width, grid.width())) {
                    let cornercase: bool =  (width >= ScaleType::from_u8(3).unwrap()  && (x == left || x == left + width - ScaleType::one()))
                       && (height >= ScaleType::from_u8(3).unwrap() && (y == top || y == top + height - ScaleType::one()));
                    if !cornercase {
                        grid.inc(x,y, ValueType::one());
                    }
                }
            }
        }
        grid
    }

    fn render(&self) -> String {
        let mut output: String = String::new();
        let min = self.min();
        let max = self.max();
        for y in range(ScaleType::zero(), self.height()) {
            for x in range(ScaleType::zero(), self.width()) {
                output += HeightGrid::rendercell(self, x,y, min, max).as_str();
            }
            output.push('\n');
        }
        output
    }

    fn rendercell(&self, x: ScaleType, y: ScaleType, min: ValueType, max: ValueType) -> String {
        let v = self[(x,y)].to_usize().unwrap();
        let min  = min.to_usize().unwrap();
        let max  = max.to_usize().unwrap();
        let colour: usize = (v - min) * (255/(max-min));
        let colour: u8 = colour as u8;
        RGB(colour,colour,colour).paint("█").to_string()
    }
}


impl<ScaleType,ValueType> RoomGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: RoomGridProperties) -> Grid<ScaleType,ValueType> {
        let mut rng = Pcg32::seed_from_u64(seed);
        let mut grid: Grid<ScaleType,ValueType> = Grid::new(width,height);
        let mut rooms: Vec<(ScaleType,ScaleType,ScaleType,ScaleType)> = Vec::new(); //left,top,width,height
        let mut tries = 0;
        while rooms.len() < properties.rooms && tries < 100 { //we give adding rooms when we fail after 100 tries
            let width: ScaleType = ScaleType::from_usize(rng.gen_range(4,grid.width_as_usize() / 4)).expect("Unable to compute width");
            let height: ScaleType = ScaleType::from_usize(rng.gen_range(4,grid.height_as_usize() / 4)).expect("Unable to compute height");
            let left: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.width_as_usize())).expect("Unable to compute left");
            let top: ScaleType = ScaleType::from_usize(rng.gen_range(0,grid.height_as_usize())).expect("Unable to compute top");

            //the room may not overlap with others
            let mut overlaps = false;
            for (left2,top2,width2,height2) in rooms.iter() {
                if left + width >= *left2 && left <= *left2 + *width2 &&
                    top + height >= *top2 && top <= *top2 + *height2 {
                        overlaps = true;
                        break;
                }
            }
            if !overlaps {
                rooms.push((left,top,width,height));
                for y in range(top, min(top + height, grid.height())) {
                    for x in range(left, min(left + width, grid.width())) {
                        grid.inc(x,y, ValueType::one());
                    }
                }
                tries = 0;
            } else {
                tries += 1;
            }
        }

        //create corridors
        let mut isolatedrooms = rooms.clone();
        while !isolatedrooms.is_empty() {
            if let Some((left,top,width,height)) = isolatedrooms.pop() {
                //find the closest other room
                let mut mindistance: Option<f64> = None;
                let mut closest: Option<usize> = None;
                for (i, (left2, top2, width2, height2)) in isolatedrooms.iter().enumerate() {
                    let distance: f64 = grid.boxdistance(left,top,width,height,*left2,*top2,*width2,*height2);
                    if mindistance.is_none() || distance < mindistance.unwrap() {
                        mindistance = Some(distance);
                        closest = Some(i);
                    }
                }
            }
        }
        grid
    }

    fn render(&self) -> String {
        let mut output: String = String::new();
        for y in range(ScaleType::zero(), self.height()) {
            for x in range(ScaleType::zero(), self.width()) {
                output += RoomGrid::rendercell(self, x,y).as_str();
            }
            output.push('\n');
        }
        output
    }

    fn rendercell(&self, x: ScaleType, y: ScaleType) -> String {
        if self[(x,y)] != ValueType::zero() {
            "█".to_string()
        } else {
            " ".to_string()
        }
    }
}

///Implementing my own min() function because cmp::min() doesn't to floats
fn fmin(x: f64, y: f64) -> f64 {
    if x < y {
        x
    } else {
        y
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
