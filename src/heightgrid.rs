use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use std::cmp::{min,PartialEq,Eq};
use num::{Integer,Num,FromPrimitive,ToPrimitive,Bounded,range,CheckedAdd,CheckedSub};
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

#[derive(Debug,Clone,Copy)]
pub enum HeightRenderStyle {
    Simple,
    HeatMap,
    Terrain
}

pub trait HeightGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded +  Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: HeightGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self, renderstyle: HeightRenderStyle) -> String;
    fn rendercell(&self, point: &Point<ScaleType>, min: ValueType, max: ValueType,renderstyle: HeightRenderStyle) -> String;
}

impl<ScaleType,ValueType> HeightGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded +  Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

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

    fn render(&self,renderstyle: HeightRenderStyle) -> String {
        let mut output: String = String::new();
        let min = self.min();
        let max = self.max();
        for (i, point) in self.rectangle().iter().enumerate() {
            if point.x() == ScaleType::zero() && i > 0 {
                output.push('\n');
            }
            output += HeightGrid::rendercell(self, &point, min, max, renderstyle).as_str();
        }
        output
    }

    fn rendercell(&self, point: &Point<ScaleType>, min: ValueType, max: ValueType,renderstyle: HeightRenderStyle) -> String {
        let v = self[point].to_usize().unwrap();
        let min  = min.to_usize().unwrap();
        let max  = max.to_usize().unwrap();
        let (r,g,b): (u8,u8,u8) = match renderstyle {
            HeightRenderStyle::Simple | HeightRenderStyle::Terrain => {
                let colour: usize = (v - min) * (255/(max-min));
                let colour: u8 = colour as u8;
                (colour,colour, colour)
            },
            HeightRenderStyle::HeatMap => {
                //convert HSV (hue, saturation, value) to RGB, assuming saturation and value are
                //always max (1)
                let hue: f64 = 360.0 - ((v as f64 - min as f64) * (360.0/(max as f64 - min as f64)));
                let x: f64 = 1.0 - ((hue / 60.0) % 2.0 - 1.0).abs();
                let x: u8 = (x * 255.0) as u8;
                match hue {
                    _ if hue < 60.0 => (255, x , 0),
                    _ if hue < 120.0 => (x, 255, 0),
                    _ if hue < 180.0 => (0, 255, x),
                    _ if hue < 240.0 => (0, x, 255),
                    _ if hue < 300.0 => (x, 0, 255),
                    _ => (x, 0, 255)
                }
            }

        };
        println!("{},{},{}",r,g,b);
        RGB(r,g,b).paint("█").to_string()
    }
}
