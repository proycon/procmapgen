use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use num::{Integer,Num,FromPrimitive,ToPrimitive,range};
use std::ops::Index;
use std::cmp::{min,PartialEq,Eq};
use std::fmt;

use crate::common::{Distance,Direction};
use crate::rectangle::Rectangle;

///A Point in an X,Y plane
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Point<ScaleType>(pub ScaleType,pub ScaleType);

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
                rng.gen_range(bounds.topleft.xs(),bounds.bottomright.xs() + 1) as u64,
                rng.gen_range(bounds.topleft.ys(),bounds.bottomright.ys() + 1) as u64
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
    pub fn xs(&self) -> usize { self.0.to_usize().expect("Out of bounds") }
    pub fn ys(&self) -> usize { self.1.to_usize().expect("Out of bounds") }
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

    pub fn set(&mut self, x: ScaleType, y: ScaleType) {
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

