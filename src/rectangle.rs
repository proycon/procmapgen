use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use clap::{App,Arg};
use num::{Integer,Num,FromPrimitive,ToPrimitive,range};
use std::ops::Index;
use std::cmp::{min,PartialEq,Eq};
use std::fmt;
use std::iter::Iterator;

use crate::common::{Distance,Direction,Volume,fmin};
use crate::point::Point;


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Rectangle<ScaleType> {
    pub topleft: Point<ScaleType>,
    pub bottomright: Point<ScaleType>,
}

///An iterator over all points in the rectangle
pub struct RectIterator<ScaleType> {
    rectangle: Rectangle<ScaleType>,
    current: Option<Point<ScaleType>>, //will be None at instantiation
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

    pub fn left(&self) -> ScaleType {
        self.topleft.x()
    }

    pub fn right(&self) -> ScaleType {
        self.bottomright.x()
    }

    pub fn top(&self) -> ScaleType {
        self.topleft.y()
    }

    pub fn bottom(&self) -> ScaleType {
        self.bottomright.y()
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
                rng.gen_range(bounds.topleft.xs(),bounds.bottomright.xs() + 1 - minwidth) as u64,
                rng.gen_range(bounds.topleft.ys(),bounds.bottomright.ys() + 1 - minheight) as u64
        );
        let bottomright = Point::new64(
                rng.gen_range(topleft.xs() + minwidth, min(topleft.xs() + maxwidth, bounds.bottomright.xs() + 1)) as u64,
                rng.gen_range(topleft.ys() + minheight, min(topleft.ys() + minwidth,  bounds.bottomright.ys() + 1) )as u64
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
        self.bottomright.x() >= other.topleft.x() && self.topleft.x() <= other.topright().x() &&
        self.bottomright.y() >= other.topleft.y() && self.topleft.y() <= other.bottomright.y()
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

impl<ScaleType> Iterator for RectIterator<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy {

    type Item = Point<ScaleType>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut current) = self.current {
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

