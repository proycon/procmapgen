extern crate rand;
extern crate clap;
extern crate num;
extern crate ansi_term;

use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use clap::{App,Arg};
use num::{Integer,Num,FromPrimitive,ToPrimitive,range};
use std::ops::Index;
use std::cmp::{min,PartialEq,Eq};
use ansi_term::Colour::{White,RGB};
use std::fmt;
use std::iter::Iterator;


///The basic grid type
pub struct Grid<ScaleType,ValueType> {
    ///A flattened vector
    data: Vec<ValueType>,

    ///The dimensions of the grid
    size: (ScaleType,ScaleType),
}

///An iterator over all points and (references to) values in the grid
pub struct GridIterator<'a, ScaleType, ValueType: 'a> {
    grid: &'a Grid<ScaleType, ValueType>,
    current: RectIterator<ScaleType>,
}


///A Point in an X,Y plane
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Point<ScaleType>(ScaleType,ScaleType);

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Rectangle<ScaleType> {
    topleft: Point<ScaleType>,
    bottomright: Point<ScaleType>,
}

///An iterator over all points in the rectangle
pub struct RectIterator<ScaleType> {
    rectangle: Rectangle<ScaleType>,
    current: Option<Point<ScaleType>>, //will be None at instantiation
}

#[derive(Debug,Clone,Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

trait Distance {
    fn distance(&self, other: &Self) -> f64;
}

trait Volume<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    fn area(&self) -> f64 {
        self.width().to_f64().expect("Conversion problem") * self.height().to_f64().expect("Conversion problem")
    }

    //Is the bounding box for this volume square?
    fn is_square(&self) -> bool {
        self.width() == self.height()
    }

    fn width(&self) -> ScaleType;
    fn height(&self) -> ScaleType;

    fn intersects(&self, other: &Self) -> bool;
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
    fn rendercell(&self, point: &Point<ScaleType>) -> String;

}

trait HeightGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: HeightGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, point: &Point<ScaleType>, min: ValueType, max: ValueType) -> String;
}

trait RoomGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: RoomGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, point: &Point<ScaleType>) -> String;
}






///A point in a 2D euclidian plane
impl<ScaleType> Point<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    pub fn new(x: ScaleType, y: ScaleType) -> Point<ScaleType> {
        Point(x,y)
    }

    //conversion
    pub fn new64(x: u64, y: u64) -> Point<ScaleType> { Point(ScaleType::from_u64(x).expect("Out of bounds"), ScaleType::from_u64(y).expect("Out of bounds")) }
    pub fn new32(x: u32, y: u32) -> Point<ScaleType> { Point(ScaleType::from_u32(x).expect("Out of bounds"), ScaleType::from_u32(y).expect("Out of bounds")) }
    pub fn new16(x: u16, y: u16) -> Point<ScaleType> { Point(ScaleType::from_u16(x).expect("Out of bounds"), ScaleType::from_u16(y).expect("Out of bounds")) }
    pub fn new8(x: u8, y: u8) -> Point<ScaleType> { Point(ScaleType::from_u8(x).expect("Out of bounds"), ScaleType::from_u8(y).expect("Out of bounds")) }

    //Generate a random point within the specified rectangular bound
    pub fn random(rng: &mut Pcg32, bounds: &Rectangle<ScaleType>) -> Point<ScaleType> {
        Point::new64(
                rng.gen_range(bounds.topleft.xS(),bounds.bottomright.xS() + 1) as u64,
                rng.gen_range(bounds.topleft.yS(),bounds.bottomright.yS() + 1) as u64
        )
    }


    pub fn x(&self) -> ScaleType { self.0 }
    pub fn y(&self) -> ScaleType { self.1 }

    pub fn rectangle(&self, width: ScaleType, height: ScaleType) -> Rectangle<ScaleType> {
        Rectangle::new_dims(self.x(), self.y(), width, height)
    }

    pub fn square(&self, size: ScaleType) -> Rectangle<ScaleType> {
        Rectangle::new_dims(self.x(), self.y(), size, size)
    }

    pub fn square1(&self) -> Rectangle<ScaleType> {
        Rectangle::new_dims(self.x(), self.y(), ScaleType::one(), ScaleType::one())
    }

    //conversion
    pub fn xS(&self) -> usize { self.0.to_usize().expect("Out of bounds") }
    pub fn yS(&self) -> usize { self.1.to_usize().expect("Out of bounds") }
    pub fn x64(&self) -> u64 { self.0.to_u64().expect("Out of bounds") }
    pub fn y64(&self) -> u64 { self.1.to_u64().expect("Out of bounds") }
    pub fn x32(&self) -> u32 { self.0.to_u32().expect("Out of bounds") }
    pub fn y32(&self) -> u32 { self.1.to_u32().expect("Out of bounds") }
    pub fn x16(&self) -> u16 { self.0.to_u16().expect("Out of bounds") }
    pub fn y16(&self) -> u16 { self.1.to_u16().expect("Out of bounds") }
    pub fn x8(&self) -> u8 { self.0.to_u8().expect("Out of bounds") }
    pub fn y8(&self) -> u8 { self.1.to_u8().expect("Out of bounds") }


    pub fn neighbour(&self, direction: Direction, width: Option<ScaleType>, height: Option<ScaleType>) -> Option<Point<ScaleType>> {
        match direction {
            Direction::North => { if self.y() == ScaleType::zero() {
                                    None
                                  } else {
                                    Some(Point(self.x(), self.y() - ScaleType::one()))
                                  } },
            Direction::East => { if width.is_some() && self.x() == width.unwrap() - ScaleType::one() {
                                    None
                                } else {
                                    Some(Point(self.x() + ScaleType::one(), self.y() ))
                                } },
            Direction::South => { if height.is_some() && self.y() == height.unwrap() - ScaleType::one() {
                                    None
                                } else {
                                    Some(Point(self.x(), self.y() + ScaleType::one()))
                                } },
            Direction::West => { if self.x() == ScaleType::zero() {
                                    None
                                } else {
                                    Some(Point(self.x() - ScaleType::one(), self.y()))
                                } },
        }
    }

    //aliases for neighbour
    pub fn north(&self) -> Option<Point<ScaleType>> { self.neighbour(Direction::North, None, None) }
    pub fn west(&self) -> Option<Point<ScaleType>> { self.neighbour(Direction::West, None, None) }
    pub fn south(&self, height: Option<ScaleType>) -> Option<Point<ScaleType>> { self.neighbour(Direction::West, None, height) }
    pub fn east(&self, width: Option<ScaleType>) -> Option<Point<ScaleType>> { self.neighbour(Direction::East, width, None) }

    pub fn set(&self, x: ScaleType, y: ScaleType) {
        self.0 = x;
        self.1 = y;
    }


}

impl<ScaleType> Distance for Point<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    ///A simple euclidian distance function
    fn distance(&self, other: &Point<ScaleType>) -> f64 {
        let x = self.x64();
        let y = self.y64();
        let x2 = other.x64();
        let y2 = other.y64();
        let distx: f64 = (x2 as f64 - x as f64).abs();
        let disty: f64 = (y2 as f64 - y as f64).abs();
        let distance: f64 = (distx.powf(2.0) + disty.powf(2.0)).sqrt();
        distance
    }
}

impl<ScaleType> fmt::Display for Point<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x64(), self.y64())
    }
}



///A rectangle in a 2D euclidian plane
impl<ScaleType> Rectangle<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    pub fn new(topleft: &Point<ScaleType>, bottomright: &Point<ScaleType>) -> Rectangle<ScaleType> {
        Rectangle {
            topleft: topleft.clone(),
            bottomright: bottomright.clone()
        }
    }

    pub fn topright(&self) -> Point<ScaleType> {
        Point(self.bottomright.x(), self.topleft.y())
    }

    pub fn bottomleft(&self) -> Point<ScaleType> {
        Point(self.topleft.x(), self.bottomright.y())
    }

    pub fn new_dims(x: ScaleType, y: ScaleType, width: ScaleType, height: ScaleType) -> Rectangle<ScaleType> {
        Rectangle {
            topleft: Point(x,y),
            bottomright: Point(x+width-ScaleType::one(),y+height-ScaleType::one())
        }
    }
    //
    ///Generate a random rectangle within the specified rectangular bound
    pub fn random(rng: &mut Pcg32, bounds: &Rectangle<ScaleType>, minwidth: Option<ScaleType>, maxwidth: Option<ScaleType>, minheight: Option<ScaleType>, maxheight: Option<ScaleType>) -> Rectangle<ScaleType> {
        let minwidth = minwidth.unwrap_or(ScaleType::one()).to_usize().unwrap();
        let maxwidth = maxwidth.unwrap_or(bounds.width()).to_usize().unwrap();
        let minheight = minheight.unwrap_or(ScaleType::one()).to_usize().unwrap();
        let maxheight = maxheight.unwrap_or(bounds.height()).to_usize().unwrap();
        let topleft = Point::new64(
                rng.gen_range(bounds.topleft.xS(),bounds.bottomright.xS() + 1 - minwidth) as u64,
                rng.gen_range(bounds.topleft.yS(),bounds.bottomright.yS() + 1 - minheight) as u64
        );
        let bottomright = Point::new64(
                rng.gen_range(topleft.xS() + minwidth, min(topleft.xS() + maxwidth, bounds.bottomright.xS() + 1)) as u64,
                rng.gen_range(topleft.yS() + minheight, min(topleft.yS() + minwidth,  bounds.bottomright.yS() + 1) )as u64
        );
        Rectangle {
            topleft: topleft,
            bottomright: bottomright,
        }
    }

    ///Iterate over all points in the rectangle
    pub fn iter(&self) -> RectIterator<ScaleType> {
        RectIterator {
            rectangle: self.clone(),
            current: None,
        }
    }
}

impl<ScaleType> Volume<ScaleType> for Rectangle<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    fn width(&self) -> ScaleType {
        self.bottomright.x() - self.topleft.x() + ScaleType::one()
    }

    fn height(&self) -> ScaleType {
        self.bottomright.y() - self.topleft.y() + ScaleType::one()
    }

    fn intersects(&self, other: &Self) -> bool {
        false
    }
}

impl<ScaleType> Distance for Rectangle<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    ///Computes the distance between two boxes, the distance is the shortest distance between a
    ///corner of box A and a corner of box B
    fn distance(&self, other: &Self) -> f64 {
        let mut d: f64 = self.topleft.distance(&other.topright());
        d = fmin(d, self.topleft.distance(&other.bottomright));
        d = fmin(d, self.topleft.distance(&other.bottomleft() ));

        d = fmin(d, self.topright().distance(&other.topleft ));
        d = fmin(d, self.topright().distance(&other.bottomright ) );
        d = fmin(d, self.topright().distance(&other.bottomleft() ) );

        d = fmin(d, self.bottomright.distance(&other.topleft) );
        d = fmin(d, self.bottomright.distance(&other.topright() ));
        d = fmin(d, self.bottomright.distance(&other.bottomleft() ));

        d = fmin(d, self.bottomleft().distance(&other.topleft) );
        d = fmin(d, self.bottomleft().distance(&other.bottomright ));
        d = fmin(d, self.bottomleft().distance(&other.topright() ) );

        d
    }
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

    pub fn rectangle(&self) -> Rectangle<ScaleType> {
        Rectangle::new_dims(ScaleType::zero(), ScaleType::zero(), self.width(), self.height())
    }

    pub fn width_as_usize(&self) -> usize {
        self.size.0.to_usize().unwrap()
    }

    pub fn height_as_usize(&self) -> usize {
        self.size.1.to_usize().unwrap()
    }

    pub fn iter(&self) -> GridIterator<ScaleType, ValueType> {
        GridIterator { grid: &self, current: self.rectangle().iter() }
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

    pub fn index(&self, point: &Point<ScaleType>) -> usize {
       (point.y() * self.width() + point.x()).to_usize().expect("Unable to cast to usize")
    }

    pub fn get(&self, point: &Point<ScaleType>) -> Option<&ValueType> {
        self.data.get(self.index(point))
    }

    pub fn get_mut(&mut self, point: &Point<ScaleType>) -> Option<&mut ValueType> {
        let index = self.index(point);
        self.data.get_mut(index)
    }

    pub fn inc(&mut self, point: &Point<ScaleType>, amount: ValueType) {
        let index = self.index(point);
        let mut v = self.data.get_mut(index).unwrap();
        //TODO: do overflow checking
        *v = *v + amount;
    }

    pub fn set(&mut self, point: &Point<ScaleType>, value: ValueType) -> bool {
        if let Some(v) = self.get_mut(point) {
            *v = value;
            return true;
        }
        return false;
    }

    pub fn hasneighbour(&self, point: &Point<ScaleType>, direction: Direction) -> bool {
        if let Some(neighbour) = point.neighbour(direction, Some(self.width()), Some(self.height())) {
            self[&neighbour] > ValueType::zero()
        } else {
            false
        }
    }

    pub fn hasneighbours(&self,point: &Point<ScaleType>) -> (bool, bool, bool, bool) {
       (
           self.hasneighbour(point, Direction::North),
           self.hasneighbour(point, Direction::East),
           self.hasneighbour(point, Direction::South),
           self.hasneighbour(point, Direction::West),
       )
    }

    pub fn countneighbours(&self, point: &Point<ScaleType>) -> usize {
       let mut count = 0;
       if self.hasneighbour(point, Direction::North) { count += 1 };
       if self.hasneighbour(point, Direction::East) { count += 1 };
       if self.hasneighbour(point, Direction::South) { count += 1 };
       if self.hasneighbour(point, Direction::West) { count += 1 };
       count
    }




    pub fn randompathto(&mut self, rng: &mut Pcg32, from: &Point<ScaleType>, to: &Point<ScaleType>, value: ValueType) {
        let mut retry = true;
        let mut retries = 0;
        let mut walk = *from; //copy
        while retry {
            let dx = if to.x() > walk.x() { Direction::East } else { Direction::West };
            let dy = if to.y() > walk.y() { Direction::South } else { Direction::North };
            let mut iteration = 0;
            retry = false;
            while walk != *to {
                if walk != *to {
                    if self[&walk] == ValueType::zero() {
                        self.set(&walk,value);
                    } else if iteration == 1 && retries < 5 {
                        //first step must be to a node that is still empty, restart:
                        retry = true;
                        retries += 1;
                        break;
                    }
                }
                if (walk.x() != to.x()) && ((walk.y() == to.y()) || rng.gen()) {
                    walk = walk.neighbour(dx, Some(self.width()), Some(self.height())).expect("Bumped into boundary, shouldn't happen");
                } else if (walk.y() != to.y()) && ((walk.x() == to.x()) || rng.gen()) {
                    walk = walk.neighbour(dy, Some(self.width()), Some(self.height())).expect("Bumped into boundary, shouldn't happen");
                }
                iteration += 1;
            }
        }
    }

}


///Implementing the index ([]) operator for Grid
impl<ScaleType,ValueType> Index<&Point<ScaleType>> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

        type Output = ValueType;

        fn index(&self, point: &Point<ScaleType>) -> &Self::Output {
            self.get(point).expect("Out of bounds")
        }

}

impl<ScaleType> Iterator for RectIterator<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    type Item = Point<ScaleType>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current {
            current = current.east(None).expect("Iterator out of bounds");
            if current.x() > self.rectangle.bottomright.x() {
                //next row
                current.set( self.rectangle.topleft.x(), current.y() + ScaleType::one());
                if current.y() > self.rectangle.bottomright.y() {
                    //out of bounds, stop condition
                    self.current = None
                } else {
                    self.current = Some(current);
                }
            } else {
                self.current = Some(current);
            }
        } else {
            self.current = Some(self.rectangle.topleft.clone());
        };
        self.current
    }
}


impl<'a, ScaleType, ValueType> Iterator for GridIterator<'a, ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    type Item = (Point<ScaleType>,&'a ValueType);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(point) = self.current.next() {
            if let Some(value) = self.grid.get(&point) {
                Some( (point, value) )
            } else {
                None
            }
        } else {
            None
        }
    }
}


impl<ScaleType,ValueType> PipeGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    ///Generates the network (a planar graph), with a backbone
    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: PipeGridProperties) -> Grid<ScaleType,ValueType> {
        let mut rng = Pcg32::seed_from_u64(seed);
        let mut grid: Grid<ScaleType,ValueType> = Grid::new(width, height);

        let mut backboneseeds: Vec<Point<ScaleType>> = Vec::new();
        //add initial backbone nodes
        for _ in 0..properties.backboneseeds {
            let point = Point::random(&mut rng, &grid.rectangle());
            grid.set(&point, ValueType::one());
            backboneseeds.push(point);
        }

        let mut processed: Vec<usize> = Vec::new();
        //connect each backnode node to the nearest other
        for (i, point) in backboneseeds.iter().enumerate() {
            processed.push(i);

            //find the nearest unconnected other
            let mut mindistance: Option<f64> = None;
            let mut closest: Option<usize> = None;
            for (j, point2) in backboneseeds.iter().enumerate() {
                if !processed.contains(&j) {
                    let distance =  point.distance(point2);
                    if mindistance.is_none() || distance < mindistance.unwrap() {
                        mindistance = Some(distance);
                        closest = Some(j);
                    }
                }
            }

            //draw a random path
            if let Some(closest) = closest {
                let point2 = backboneseeds[closest];
                grid.randompathto(&mut rng, point, &point2, ValueType::from_u8(2).unwrap());
            }
        }

        //Add regular nodes (multiple iterations of a specific amount of seeds)
        for (iternr, regularseedgoal) in properties.regularseeds.iter().enumerate() {
            let mut regularseeds = 0;
            let height: ValueType = ValueType::from_usize(iternr).expect("Conversion error") + ValueType::from_u8(3).unwrap();
            while regularseeds < *regularseedgoal {
                let point = Point::random(&mut rng, &grid.rectangle());
                if grid[&point] == ValueType::zero() {
                    regularseeds += 1;
                    grid.set(&point,height);
                    //find the closest backbone
                    let mut mindistance: Option<f64> = None;
                    let mut closest: Option<Point<ScaleType>> = None;
                    for (point2, v) in grid.iter() {
                        if *v > ValueType::zero() && *v < height {
                            let distance: f64 = point.distance(&point2);
                            if mindistance.is_none() || distance < mindistance.unwrap() {
                                mindistance = Some(distance);
                                closest = Some(point2);
                            }
                        }
                    }
                    //
                    //draw a random path to the closest backbone
                    if let Some(point2) = closest {
                        grid.randompathto(&mut rng, &point, &point2, height+ValueType::one());
                    }
                }
            }
        }

        if properties.interconnect {
            //prune dead ends by creating more interconnections
            let mut deadends: Vec<Point<ScaleType>> = Vec::new();
            let mut processed: Vec<Point<ScaleType>> = Vec::new();
            //find all dead ends
            for (point,value) in grid.iter() {
               if *value > ValueType::from_u8(2).unwrap() && grid.countneighbours(&point) == 1 {
                   deadends.push(point);
               }
            }

            for point in deadends.iter() {
              if !processed.contains(point) {
                //we find the closest other dead end (or former dead end)
                let mut mindistance: Option<f64> = None;
                let mut closest: Option<Point<ScaleType>> = None;
                for point2 in deadends.iter() {
                    if point != point2 {
                        let distance: f64 = point.distance(&point2);
                        if mindistance.is_none() || distance < mindistance.unwrap() {
                            mindistance = Some(distance);
                            closest = Some(point2.clone());
                        }
                    }
                }
                //draw a random path to the closest (former) dead end
                if let Some(closest) = closest {
                    grid.randompathto(&mut rng, point, &closest, ValueType::from_u8(99).unwrap());
                    processed.push(closest);
                }
              }
            }
        }
        grid
    }

    fn render(&self) -> String {
        let mut output: String = String::new();
        for (i, point) in self.rectangle().iter().enumerate() {
            if point.x() == ScaleType::zero() && i > 0 {
                output.push('\n');
            }
            output += PipeGrid::rendercell(self, &point).as_str();
        }
        output
    }

    fn rendercell(&self, point: &Point<ScaleType>) -> String {
        let v = self[point];
        let chr: char = if v == ValueType::zero() {
            ' '
        } else {
           let (hasnorth, haseast, hassouth, haswest) = self.hasneighbours(point);
           let isbackbone = v <= ValueType::from_u8(2).unwrap();
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

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: HeightGridProperties) -> Grid<ScaleType,ValueType> {
        let mut rng = Pcg32::seed_from_u64(seed);
        let mut grid: Grid<ScaleType,ValueType> = Grid::new(width,height);
        for i in 0..properties.iterations {
            let rect: Rectangle<ScaleType> = Rectangle::random(&mut rng, &grid.rectangle(),
                               Some(ScaleType::one()),  //minwidth
                               Some(ScaleType::from_usize(grid.width_as_usize() / 5).expect("conversion error")), //maxwidth
                               Some(ScaleType::one()),  //minheight
                               Some(ScaleType::from_usize(grid.height_as_usize() / 5).expect("conversion error")), //maxheight
            );
            for point in rect.iter() {
                let cornercase: bool =  (rect.width() >= ScaleType::from_u8(3).unwrap()  && (point.x() == rect.topleft.x() || point.x() == rect.topright().x()))
                   && (point.y() >= ScaleType::from_u8(3).unwrap() && (point.y() == rect.topleft.y() || point.y() == rect.bottomright.y()));
                if !cornercase {
                    grid.inc(&point, ValueType::one());
                }
            }
        }
        grid
    }

    fn render(&self) -> String {
        let mut output: String = String::new();
        let min = self.min();
        let max = self.max();
        for (i, point) in self.rectangle().iter().enumerate() {
            if point.x() == ScaleType::zero() && i > 0 {
                output.push('\n');
            }
            output += HeightGrid::rendercell(self, &point, min, max).as_str();
        }
        output
    }

    fn rendercell(&self, point: &Point<ScaleType>, min: ValueType, max: ValueType) -> String {
        let v = self[point].to_usize().unwrap();
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

                if let Some(index) = closest {
                    let (left2, top2, width2, height2) = isolatedrooms.remove(index);
                    let mut corridor_h: Option<ScaleType> = None;
                    let mut corridor_v: Option<ScaleType> = None;
                    //can we do a horizontal corridor?
                    if top <= top2 + height2 && top + height >= top2 {
                        //horizontal corridor
                        corridor_h = Some(ScaleType::from_usize(rng.gen_range( top2.to_usize().unwrap() , top2.to_usize().unwrap() + height2.to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    } else if top2 <= top + height && top2 + height2 >= top {
                        //horizontal corridor
                        corridor_h = Some(ScaleType::from_usize(rng.gen_range( top.to_usize().unwrap() , top.to_usize().unwrap() + height.to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    } else if left <= left2 + width2 && left + width >= left2 {
                        corridor_v = Some(ScaleType::from_usize(rng.gen_range( left2.to_usize().unwrap() , left2.to_usize().unwrap() + width2.to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    } else if left2 <= left + width && left2 + width2 >= left {
                        corridor_v = Some(ScaleType::from_usize(rng.gen_range( left.to_usize().unwrap() , left.to_usize().unwrap() + width.to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    }
                    if let Some(corridor_h) = corridor_h {
                        let (begin_x, end_x) = if left < left2 {
                            (left + width, left2)
                        } else {
                            (left2 + width2, left)
                        };
                        for x in range(begin_x, end_x) {
                            grid.set(x,corridor_h, ValueType::one());
                        }
                    } else if let Some(corridor_v) = corridor_v {
                        let (begin_y, end_y) = if top < top2 {
                            (top + height, top2)
                        } else {
                            (top2 + height2, top)
                        };
                        for y in range(begin_y, end_y) {
                            grid.set(corridor_v,y, ValueType::one());
                        }
                    } else {
                        //TODO: cornered corridors
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
