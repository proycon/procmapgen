extern crate rand;
extern crate clap;
extern crate num;
extern crate ansi_term;

use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use clap::{App,Arg};
use num::{Integer,Num,FromPrimitive,ToPrimitive,range};
use std::ops::Index;
use std::cmp::{min,max};
use ansi_term::Colour::{White,RGB};



#[derive(Debug,Default)]
pub struct PipeGridProperties {
    pub width: usize,
    pub height: usize,
    ///seed for the random number generator that creates the network
    pub seed: u64,
    ///initial backbone points
    pub backboneseeds: u16,
    pub regularseeds: Vec<u16>, //multiple iterations
    pub interconnect: bool,
}

#[derive(Debug,Default)]
pub struct HeightGridProperties {
    pub width: usize,
    pub height: usize,
    ///seed for the random number generator that creates the network
    pub seed: u64,
    pub iterations: usize, //number of iterations
}

trait RenderGrid<ScaleType> {
    fn render(&self) -> String;
    fn rendercell(&self, x: ScaleType, y: ScaleType) -> String;
}

pub struct Grid<ScaleType,ValueType> {
    data: Vec<ValueType>,
    size: (ScaleType,ScaleType),
}

pub struct PipeGrid<ScaleType> {
    properties: PipeGridProperties,
    grid: Grid<ScaleType,u8>,
}

pub struct HeightGrid<ScaleType> {
    properties: HeightGridProperties,
    grid: Grid<ScaleType,u8>,
}


impl<ScaleType,ValueType> Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy, ValueType: Num + PartialOrd + PartialEq + Copy {
    pub fn new(width: &ScaleType, height: &ScaleType) -> Grid<ScaleType,ValueType> {
        //create initial empty 2D grid
        let mut grid: Vec<ValueType> = Vec::new(); //flattened grid
        for _ in range(ScaleType::zero(), height.clone()) {
            for _ in range(ScaleType::zero(), width.clone()) {
                grid.push(ValueType::zero());
            }
        }

        Grid {
            data: grid,
            size: (width.clone(), height.clone()),
        }
    }

    pub fn width(&self) -> ScaleType {
        self.size.0.clone()
    }

    pub fn height(&self) -> ScaleType {
        self.size.1.clone()
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
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy, ValueType: Num + PartialOrd + PartialEq + Copy {
        type Output = ValueType;

        fn index(&self, index: (ScaleType, ScaleType)) -> &Self::Output {
            let (x,y) = index;
            self.get(x,y).expect("Out of bounds")
        }

}


impl<ScaleType> RenderGrid<ScaleType> for PipeGrid<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    fn render(&self) -> String {
        let mut output: String = String::new();
        for y in range(ScaleType::zero(), self.grid.height()) {
            for x in range(ScaleType::zero(), self.grid.width()) {
                output += self.rendercell(x,y).as_str();
            }
            output.push('\n');
        }
        output
    }

    fn rendercell(&self, x: ScaleType, y: ScaleType) -> String {
        let v = self.grid[(x,y)];
        let chr: char = if v == 0 {
            ' '
        } else {
           let (hasnorth, haseast, hassouth, haswest) = self.grid.hasneighbours(x, y);
           let isbackbone = self.grid[(x,y)] <= 2;
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

impl<ScaleType> PipeGrid<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    ///Generates the network (a planar graph), with a backbone
    pub fn generate(properties: PipeGridProperties) -> PipeGrid<ScaleType> {
        let mut rng = Pcg32::seed_from_u64(properties.seed);
        let mut grid: Grid<ScaleType,u8> = Grid::new(&ScaleType::from_usize(properties.width).unwrap(), &ScaleType::from_usize(properties.height).unwrap());

        let mut backboneseeds: Vec<(ScaleType,ScaleType)> = Vec::new();
        //add initial backbone nodes
        for _ in 0..properties.backboneseeds {
            let x: ScaleType = ScaleType::from_usize(rng.gen_range(0,properties.width)).unwrap();
            let y: ScaleType = ScaleType::from_usize(rng.gen_range(0,properties.height)).unwrap();
            grid.set(x,y, 1);
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
                grid.randompathto(&mut rng, *x, *y, to_x, to_y, 2);
            }
        }

        //Add regular nodes (multiple iterations of a specific amount of seeds)
        for (iternr, regularseedgoal) in properties.regularseeds.iter().enumerate() {
            let mut regularseeds = 0;
            let height: u8 = iternr as u8 + 3;
            while regularseeds < *regularseedgoal {
                let x: ScaleType = ScaleType::from_usize(rng.gen_range(0,properties.width)).unwrap();
                let y: ScaleType = ScaleType::from_usize(rng.gen_range(0,properties.height)).unwrap();
                if grid[(x,y)] == 0 {
                    regularseeds += 1;
                    grid.set(x,y,height);
                    //find the closest backbone
                    let mut mindistance: Option<f64> = None;
                    let mut closest: Option<(ScaleType,ScaleType)> = None;
                    for y2 in range(ScaleType::zero(), grid.height()) {
                        for x2 in range(ScaleType::zero(), grid.width()) {
                            let v = grid[(x2,y2)];
                            if v > 0 && v < height {
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
                        grid.randompathto(&mut rng, x, y, to_x, to_y, height+1);
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
                   if grid[(x,y)] > 2 && grid.countneighbours(x, y) == 1 {
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
                    grid.randompathto(&mut rng, *x, *y, to_x, to_y, 99);
                    processed.push(closest);
                }
              }
            }
        }
        PipeGrid {
            properties: properties,
            grid: grid,
        }
    }

}

impl<ScaleType> HeightGrid<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    pub fn generate(properties: HeightGridProperties) -> HeightGrid<ScaleType> {
        let mut rng = Pcg32::seed_from_u64(properties.seed);
        let mut grid: Grid<ScaleType,u8> = Grid::new(&ScaleType::from_usize(properties.width).unwrap(), &ScaleType::from_usize(properties.height).unwrap());
        for i in 0..properties.iterations {
            let width: ScaleType = ScaleType::from_usize(rng.gen_range(1,properties.width / 5)).expect("Unable to compute width");
            let height: ScaleType = ScaleType::from_usize(rng.gen_range(1,properties.width / 5)).expect("Unable to compute height");
            let left: ScaleType = ScaleType::from_usize(rng.gen_range(0,properties.width)).expect("Unable to compute left");
            let top: ScaleType = ScaleType::from_usize(rng.gen_range(0,properties.height)).expect("Unable to compute top");
            //TODO: implement edge smoothing
            for y in range(top, min(top + height, grid.height())) {
                for x in range(left, min(left + width, grid.width())) {
                    grid.inc(x,y, 1);
                }
            }
        }
        HeightGrid {
            properties: properties,
            grid: grid,
        }
    }
}

impl<ScaleType> RenderGrid<ScaleType> for HeightGrid<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    fn render(&self) -> String {
        let mut output: String = String::new();
        for y in range(ScaleType::zero(), self.grid.height()) {
            for x in range(ScaleType::zero(), self.grid.width()) {
                output += self.rendercell(x,y).as_str();
            }
            output.push('\n');
        }
        output
    }

    fn rendercell(&self, x: ScaleType, y: ScaleType) -> String {
        let max = self.grid.max();
        let min = self.grid.min();
        let v = self.grid[(x,y)];
        let colour = (v - min) * (255/(max-min));
        RGB(colour,colour,colour).paint("█").to_string()
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
    match argmatches.value_of("type").unwrap() {
        "pipes" => {
            let regularseeds: Option<Vec<&str>>= argmatches.value_of("regularseeds").map(|regularseeds: &str| {
                                    regularseeds.split_terminator(',').collect()
                                });
            let regularseeds: Vec<u16> = regularseeds.unwrap().iter().map(|x:&&str| { x.parse::<u16>().unwrap() } ).collect();
            let properties = PipeGridProperties {
                width: argmatches.value_of("width").unwrap().parse::<usize>().unwrap() as usize,
                height: argmatches.value_of("height").unwrap().parse::<usize>().unwrap() as usize,
                seed: seed,
                backboneseeds: argmatches.value_of("backboneseeds").unwrap().parse::<u16>().unwrap() as u16,
                regularseeds: regularseeds,
                interconnect: argmatches.is_present("interconnect"),
            };
            let grid: PipeGrid<u16> = PipeGrid::generate(properties);
            println!("{}",grid.render());
        },
        "height" => {
            let properties = HeightGridProperties {
                width: argmatches.value_of("width").unwrap().parse::<usize>().unwrap() as usize,
                height: argmatches.value_of("height").unwrap().parse::<usize>().unwrap() as usize,
                seed: seed,
                iterations: argmatches.value_of("iterations").unwrap().parse::<usize>().unwrap() as usize,
            };
            let grid: HeightGrid<u16> = HeightGrid::generate(properties);
            println!("{}",grid.render());
        },
        _ => {
            eprintln!("No such type");
        }
    }
}
