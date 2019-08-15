use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use num::{Integer,Num,FromPrimitive,ToPrimitive,Bounded,range,CheckedAdd,CheckedSub};
use std::ops::{Index,Add,AddAssign};
use std::cmp::{min,max,PartialEq,Eq,Ord,PartialOrd,Ordering};
use std::fmt;
use std::iter::{Iterator,FromIterator};
use std::collections::BinaryHeap;
use ansi_term;

use crate::common::{Distance,Direction};
use crate::point::Point;
use crate::rectangle::{Rectangle,RectIterator};


///The basic grid type
#[derive(Clone,PartialEq)]
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

pub trait GenericGrid<ScaleType, ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: Default + PartialEq  + Clone {

    fn new(width: ScaleType, height: ScaleType) -> Grid<ScaleType,ValueType> {
        Self::new_init(width, height, ValueType::default())
    }

    fn new_init(width: ScaleType, height: ScaleType, value: ValueType) -> Grid<ScaleType,ValueType> {
        //create initial empty 2D grid
        let size = width.to_usize().unwrap() * height.to_usize().unwrap();
        let mut grid: Vec<ValueType> = Vec::with_capacity(size); //flattened grid
        for _ in 0..size {
            grid.push(value.clone());
        }

        Grid {
            data: grid,
            size: (width, height),
        }
    }



    fn rectangle(&self) -> Rectangle<ScaleType> {
        Rectangle::new_dims(ScaleType::zero(), ScaleType::zero(), self.width(), self.height())
    }

    fn width_as_usize(&self) -> usize {
        self.width().to_usize().expect("Conversion error")
    }

    fn height_as_usize(&self) -> usize {
        self.height().to_usize().expect("Conversion error")
    }


    ///Point to Index
    fn index(&self, point: &Point<ScaleType>) -> usize {
       (point.y() * self.width() + point.x()).to_usize().expect("Unable to cast to usize")
    }

    ///Index to Point
    fn point(&self, index: usize) -> Point<ScaleType> {
        let y = index / self.width_as_usize();
        let x = index % self.width_as_usize();
        Point(ScaleType::from_usize(x).unwrap(), ScaleType::from_usize(y).unwrap())
    }

    fn set(&mut self, point: &Point<ScaleType>, value: ValueType) -> bool {
        if let Some(v) = self.get_mut(point) {
            *v = value;
            return true;
        }
        return false;
    }

    fn is_set(&self, point: &Point<ScaleType>) -> bool {
        *self.get(point).expect("Unable to unpack") != ValueType::default()
    }

    fn set_index(&mut self, index: usize, value: ValueType) -> bool {
        if let Some(mut v) = self.get_mut_by_index(index) {
            *v = value;
            true
        } else {
            false
        }
    }

    fn hasneighbour(&self, point: &Point<ScaleType>, direction: Direction) -> bool {
        if let Some(neighbour) = point.neighbour(direction, Some(self.width()), Some(self.height())) {
            match self.get(&neighbour) {
                Some(value) if *value != ValueType::default() => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn hasneighbours(&self,point: &Point<ScaleType>) -> (bool, bool, bool, bool) {
       (
           self.hasneighbour(point, Direction::North),
           self.hasneighbour(point, Direction::East),
           self.hasneighbour(point, Direction::South),
           self.hasneighbour(point, Direction::West),
       )
    }

    fn countneighbours(&self, point: &Point<ScaleType>) -> usize {
       let mut count = 0;
       if self.hasneighbour(point, Direction::North) { count += 1 };
       if self.hasneighbour(point, Direction::East) { count += 1 };
       if self.hasneighbour(point, Direction::South) { count += 1 };
       if self.hasneighbour(point, Direction::West) { count += 1 };
       count
    }

    fn getneighbour(&self, point: &Point<ScaleType>, direction: Direction) -> Option<Point<ScaleType>> {
        if let Some(neighbour) = point.neighbour(direction, Some(self.width()), Some(self.height())) {
            Some(neighbour)
        } else {
            None
        }
    }

    fn getneighbours(&self, point: &Point<ScaleType>) -> Vec<Point<ScaleType>> {
       let mut neighbours = Vec::new();
       if let Some(neighbour) = self.getneighbour(point, Direction::North) { neighbours.push(neighbour); }
       if let Some(neighbour) = self.getneighbour(point, Direction::East) { neighbours.push(neighbour); }
       if let Some(neighbour) = self.getneighbour(point, Direction::South) { neighbours.push(neighbour); }
       if let Some(neighbour) = self.getneighbour(point, Direction::West) { neighbours.push(neighbour); }
       neighbours
    }


    fn get(&self, point: &Point<ScaleType>) -> Option<&ValueType> {
        self.get_data_vec().get(self.index(point))
    }

    fn get_mut(&mut self, point: &Point<ScaleType>) -> Option<&mut ValueType> {
        let index = self.index(point);
        self.get_mut_data_vec().get_mut(index)
    }

    fn get_by_index(&self, index: usize) -> Option<&ValueType> {
        self.get_data_vec().get(index)
    }

    fn get_mut_by_index(&mut self, index: usize) -> Option<&mut ValueType> {
        self.get_mut_data_vec().get_mut(index)
    }

    //methods that need to be implemented:
    fn width(&self) -> ScaleType;
    fn height(&self) -> ScaleType;
    fn iter(&self) -> GridIterator<ScaleType, ValueType>;
    fn get_data_vec(&self) -> &Vec<ValueType>;
    fn get_mut_data_vec(&mut self) -> &mut Vec<ValueType>;

}

impl<ScaleType,ValueType> GenericGrid<ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: Default + PartialEq + Clone {


    fn width(&self) -> ScaleType {
        self.size.0
    }

    fn height(&self) -> ScaleType {
        self.size.1
    }

    fn iter(&self) -> GridIterator<ScaleType, ValueType> {
        GridIterator { grid: &self, current: self.rectangle().iter() }
    }


    fn get_data_vec(&self) -> &Vec<ValueType> {
        &self.data
    }

    fn get_mut_data_vec(&mut self) -> &mut Vec<ValueType> {
        &mut self.data
    }
}

pub trait NumericGrid<'a,ScaleType, ValueType>: GenericGrid<ScaleType, ValueType> + Index<&'a Point<ScaleType>> where
    ScaleType: 'a + Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: 'a + Num + Default + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

    fn max(&self) -> ValueType {
        let mut largest: Option<ValueType> = None;
        for (_, v) in self.iter() {
            if largest.is_none() ||  largest.unwrap() < *v {
                largest = Some(*v);
            }
        }
        largest.expect("Grid has no data")
    }

    fn min(&self) -> ValueType {
        let mut smallest: Option<ValueType> = None;
        for (_,v) in self.iter() {
            if smallest.is_none() ||  smallest.unwrap() > *v {
                smallest = Some(*v);
            }
        }
        smallest.expect("Grid has no data")
    }

    ///Clones the grid with a different ValueType and runs the map function
    fn map_into<ToValueType>(&self, f: impl Fn(Point<ScaleType>, ToValueType) -> ToValueType ) -> Grid<ScaleType,ToValueType> where
        ToValueType: Num + Default + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

        let mut clone: Grid<ScaleType,ToValueType> = Grid::new(self.width(), self.height());
        for (i, (point, value)) in self.iter().enumerate() {
            //MAYBE TODO: add fallback conversion options?
            let tovalue = ToValueType::from_usize(value.to_usize().expect("map_into(): Unable to convert to usize")).expect("map_into(): unable to convert from usize");
            clone.set_index(i, f(point, tovalue));
        }
        clone
    }

    fn inc(&mut self, point: &Point<ScaleType>, amount: ValueType) -> bool {
        let mut value = self.get_mut(point).expect("Point not found in grid!");
        if let Some(result) = value.checked_add(&amount) {
            *value = result;
            true
        } else {
            //value is saturated
            *value = ValueType::max_value();
            false
        }
    }

    fn dec(&mut self, point: &Point<ScaleType>, amount: ValueType) -> bool {
        let mut value = self.get_mut(point).expect("Point not found in grid!");
        if let Some(result) = value.checked_sub(&amount) {
            *value = result;
            true
        } else {
            //value is saturated
            *value = ValueType::min_value();
            false
        }
    }

    fn randompathto(&mut self, rng: &mut Pcg32, from: &Point<ScaleType>, to: &Point<ScaleType>, value: ValueType) {
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
                    if !self.is_set(&walk) {
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

    ///Creates a rectangular path (only horizontal and vertical) between points A and B
    fn rectpathto(&mut self, rng: &mut Pcg32, from: &Point<ScaleType>, to: &Point<ScaleType>, value: ValueType) {
        if from == to {
            return;
        }
        let dx = if to.x() > from.x() { Direction::East } else { Direction::West };
        let dy = if to.y() > from.y() { Direction::South } else { Direction::North };
        let horizontal_first: bool = rng.gen();
        let xrange = range(min(from.x(),to.x()), max(from.x(),to.x()) + ScaleType::one());
        let yrange = range(min(from.y(),to.y()), max(from.y(),to.y()) + ScaleType::one());
        if horizontal_first {
            for x in  xrange {
                let point = Point(x,from.y() );
                if self.get(&point) == Some(&ValueType::zero()) { self.set(&point,value); };
            }
            for y in  yrange {
                let point = Point(to.x(), y );
                if self.get(&point) == Some(&ValueType::zero()) { self.set(&point,value); };
            }
        } else {
            for y in  yrange {
                let point = Point(from.x(), y );
                if self.get(&point) == Some(&ValueType::zero()) { self.set(&point,value); };
            }
            for x in  xrange {
                let point = Point(x,to.y() );
                if self.get(&point) == Some(&ValueType::zero()) { self.set(&point,value); };
            }
        }

    }

    fn add(&mut self, other: &Self) {
        let width = min(self.width(), other.width());
        let height = min(self.height(), other.height());

        for x in range(ScaleType::zero(),width) {
            for y in range(ScaleType::zero(),height) {
                let point = Point(x,y);
                if let Some(value) = other.get(&point) {
                    self.inc(&point, *value);
                }
            }
        }
    }

    fn sub(&mut self, other: &Self) {
        let width = min(self.width(), other.width());
        let height = min(self.height(), other.height());

        for x in range(ScaleType::zero(),width) {
            for y in range(ScaleType::zero(),height) {
                let point = Point(x,y);
                if let Some(value) = other.get(&point) {
                    self.dec(&point, *value);
                }
            }
        }
    }

}

impl<'a, ScaleType,ValueType> NumericGrid<'a, ScaleType,ValueType> for Grid<ScaleType,ValueType> where
    ScaleType: 'a + Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: 'a + Num + Default + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

}



impl<ScaleType,ValueType> fmt::Display for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: fmt::Display + Default + PartialEq + Clone {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (point, value)) in self.iter().enumerate() {
            if point.x() == ScaleType::zero() && i > 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", value)?;
        }
        Ok(())
    }
}




    /*
    ///Dijkstra pathfinding algorithm
    fn findpath(&self, from: &Point<ScaleType>, to: &Point<ScaleType>, costgrid: Option<Grid<ScaleType,u32>>) -> Vec<Point<ScaleType>> {

        let mut fringe: BinaryHeap<PathState<ScaleType>> = BinaryHeap::new();

        //Maintains current distance from "from" to each node, initialise to the highest possible
        //value
        let mut dist: Grid<ScaleType,u32> = Grid::new_init(self.width(), self.height(), u32::max_value());

        let costgrid = costgrid.unwrap_or(self.map_into(
                |_point,value| {
                    if value == 0 { 0 } else { 1 } //0 means inaccessible
                }
        ));

        //push the start
        dist.set(from, 0);
        fringe.push(PathState { point: *from, cost: 0 });

        while let Some(PathState { point, cost }) = fringe.pop() {
            if point == *to {

            }

            if cost > dist[&point] {
                continue;
            }


            //Expand the neighbour nodes,
            for neighbour in self.getneighbours(&point).into_iter() {
                let nextstate = PathState { point: neighbour, cost: cost + costgrid[&neighbour] };

                if nextstate.cost < dist[&neighbour] {

                }


            }


        }

        vec![]

    }
    */



#[derive(Eq,PartialEq)]
struct PathState<ScaleType> {
   point: Point<ScaleType>,
   cost: u32
}


#[derive(Eq,PartialEq,Default,Clone)]
pub struct RenderedTextCell {
    ///The background colour (R,G,B)
    pub background_colour: Option<(u8,u8,u8)>,
    ///The foreground colour (R,G,B)
    pub foreground_colour: Option<(u8,u8,u8)>,
    ///the glyph to render, the text should span only one cell (or at least consistently the same number for  the entire grid)
    pub text: Option<String>,
}

//TODO later: implement Add/AssignAdd to combine RenderedTextCells so we can combine different
//layers

impl fmt::Display for RenderedTextCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: &str = if let Some(s) = self.text.as_ref() {
            s
        } else {
            " "
        };
        if self.background_colour.is_some() || self.foreground_colour.is_some() {
            let mut style = ansi_term::Style::new();
            if let Some((r,g,b)) = self.background_colour {
                style = style.on(ansi_term::Colour::RGB(r,g,b));
            }
            if let Some((r,g,b)) = self.foreground_colour {
                style = style.fg(ansi_term::Colour::RGB(r,g,b));
            }
            write!(f, "{}",style.paint(text))
        } else {
            write!(f, "{}",text)
        }
    }
}



// The priority queue depends on `Ord`. (from:
// https://doc.rust-lang.org/std/collections/binary_heap/index.html)
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl<ScaleType> Ord for PathState<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy {

    fn cmp(&self, other: &PathState<ScaleType>) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.cost.cmp(&self.cost)
            .then_with(|| self.point.cmp(&other.point))
    }
}

// `PartialOrd` needs to be implemented as well.
impl<ScaleType> PartialOrd for PathState<ScaleType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy {

    fn partial_cmp(&self, other: &PathState<ScaleType>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


///Implementing the index ([]) operator for Grid
impl<ScaleType,ValueType> Index<&Point<ScaleType>> for Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: Default + PartialEq + Clone {

        type Output = ValueType;

        fn index(&self, point: &Point<ScaleType>) -> &Self::Output {
            <Self as GenericGrid<ScaleType,ValueType>>::get(self,point).expect("Out of bounds")
        }
}



impl<'a, ScaleType, ValueType> Iterator for GridIterator<'a, ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: Default + PartialEq + Clone {

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


