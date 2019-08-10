use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use std::cmp::{min,PartialEq,Eq};
use num::{Integer,Num,FromPrimitive,ToPrimitive,range};

use crate::common::{Distance,Direction,Volume};
use crate::point::Point;
use crate::rectangle::Rectangle;
use crate::grid::Grid;

pub struct RoomGridProperties {
    pub rooms: usize,
}

pub trait RoomGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: RoomGridProperties) -> Grid<ScaleType,ValueType>;
    fn render(&self) -> String;
    fn rendercell(&self, point: &Point<ScaleType>) -> String;
}

impl<ScaleType,ValueType> RoomGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Copy {

    fn generate(width: ScaleType, height: ScaleType, seed: u64, properties: RoomGridProperties) -> Grid<ScaleType,ValueType> {
        let mut rng = Pcg32::seed_from_u64(seed);
        let mut grid: Grid<ScaleType,ValueType> = Grid::new(width,height);
        let mut rooms: Vec<Rectangle<ScaleType>> = Vec::new(); //left,top,width,height
        let mut tries = 0;
        while rooms.len() < properties.rooms && tries < 100 { //we give adding rooms when we fail after 100 tries
            let room: Rectangle<ScaleType> = Rectangle::random(&mut rng, &grid.rectangle(),
                               Some(ScaleType::from_u8(3).expect("conversion error")),  //minwidth
                               Some(ScaleType::from_usize(grid.width_as_usize() / 4).expect("conversion error")), //maxwidth
                               Some(ScaleType::from_u8(3).expect("conversion error")),  //minheight
                               Some(ScaleType::from_usize(grid.height_as_usize() / 4).expect("conversion error")), //maxheight
            );

            //the room may not overlap with others
            let mut overlaps = false;
            for room2 in rooms.iter() {
                if room.intersects(room2) {
                    overlaps = true;
                    break;
                }
            }
            if !overlaps {
                rooms.push(room);
                for point in room.iter() {
                    grid.inc(&point, ValueType::one());
                }
                tries = 0;
            } else {
                tries += 1;
            }
        }

        //create corridors
        let mut isolatedrooms = rooms.clone();
        while !isolatedrooms.is_empty() {
            if let Some(room) = isolatedrooms.pop() {
                //find the closest other room
                let mut mindistance: Option<f64> = None;
                let mut closest: Option<usize> = None;
                for (i, room2) in isolatedrooms.iter().enumerate() {
                    let distance: f64 = room.distance(&room2);
                    if mindistance.is_none() || distance < mindistance.unwrap() {
                        mindistance = Some(distance);
                        closest = Some(i);
                    }
                }

                if let Some(index) = closest {
                    let room2 = isolatedrooms.remove(index);
                    let mut corridor_h: Option<ScaleType> = None;
                    let mut corridor_v: Option<ScaleType> = None;
                    //can we do a horizontal corridor?
                    if room.top() <= room2.bottom() && room.bottom() >= room2.top() {
                        //horizontal corridor
                        corridor_h = Some(ScaleType::from_usize(rng.gen_range( room2.top().to_usize().unwrap() , room2.bottom().to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    } else if room2.top() <= room.bottom() && room2.bottom() >= room.top() {
                        //horizontal corridor
                        corridor_h = Some(ScaleType::from_usize(rng.gen_range( room.top().to_usize().unwrap() , room.bottom().to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    } else if room.left() <= room2.right() && room.right() >= room2.left() {
                        corridor_v = Some(ScaleType::from_usize(rng.gen_range( room2.left().to_usize().unwrap() , room2.right().to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    } else if room2.left() <= room.right() && room2.right() >= room.left() {
                        corridor_v = Some(ScaleType::from_usize(rng.gen_range( room.left().to_usize().unwrap() , room.right().to_usize().unwrap()  )).expect("Unable to compute corridor H"));
                    }
                    if let Some(corridor_h) = corridor_h {
                        let (begin_x, end_x) = if room.left() < room2.left() {
                            (room.right(), room2.left())
                        } else {
                            (room2.right(), room.left())
                        };
                        for x in range(begin_x, end_x) {
                            grid.set(&Point(x,corridor_h), ValueType::one());
                        }
                    } else if let Some(corridor_v) = corridor_v {
                        let (begin_y, end_y) = if room.top() < room2.top() {
                            (room.bottom(), room2.top())
                        } else {
                            (room2.bottom(), room.top())
                        };
                        for y in range(begin_y, end_y) {
                            grid.set(&Point(corridor_v,y), ValueType::one());
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
        for (i, point) in self.rectangle().iter().enumerate() {
            if point.x() == ScaleType::zero() && i > 0 {
                output.push('\n');
            }
            output += RoomGrid::rendercell(self, &point).as_str();
        }
        output
    }

    fn rendercell(&self, point: &Point<ScaleType>) -> String {
        if self[&point] != ValueType::zero() {
            "█".to_string()
        } else {
            " ".to_string()
        }
    }
}
