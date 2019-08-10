use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use std::cmp::{min,PartialEq,Eq};
use num::{Integer,Num,FromPrimitive,ToPrimitive,range};
use ansi_term::Colour::RGB;

use crate::common::{Distance,Direction,Volume};
use crate::point::Point;
use crate::rectangle::Rectangle;
use crate::grid::Grid;

#[derive(Debug,Default)]
pub struct HeightGridProperties {
    ///number of iterations
    pub iterations: usize,
}

pub trait HeightGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: HeightGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, point: &Point<ScaleType>, min: ValueType, max: ValueType) -> String;
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
