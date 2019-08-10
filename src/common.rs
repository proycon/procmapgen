use num::{Integer,FromPrimitive,ToPrimitive};

#[derive(Debug,Clone,Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}


pub trait Distance {
    fn distance(&self, other: &Self) -> f64;
}

pub trait Volume<ScaleType> where
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

///Implementing my own min() function because cmp::min() doesn't to floats
pub fn fmin(x: f64, y: f64) -> f64 {
    if x < y {
        x
    } else {
        y
    }
}
