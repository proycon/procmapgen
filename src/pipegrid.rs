use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use std::cmp::{min,PartialEq,Eq};
use num::{Integer,Num,FromPrimitive,ToPrimitive,Bounded,range,CheckedAdd,CheckedSub};

use crate::common::{Distance,Direction,Volume};
use crate::point::Point;
use crate::rectangle::Rectangle;
use crate::grid::Grid;

#[derive(Debug,Default)]
pub struct PipeGridProperties {
    ///initial backbone points
    pub backboneseeds: u16,

    ///amount of regular seeds to place, each element corresponds to an iteration
    pub regularseeds: Vec<u16>,

    ///prune dead-ends to a large extent by interconnecting them
    pub interconnect: bool,
}

pub trait PipeGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded +  Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: PipeGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, point: &Point<ScaleType>) -> String;

}

impl<ScaleType,ValueType> PipeGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded +  Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

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
