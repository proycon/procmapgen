use rand::{SeedableRng,Rng};
use rand_pcg::Pcg32;
use num::{Integer,Num,FromPrimitive,ToPrimitive,Bounded,range,CheckedAdd,CheckedSub};
use std::ops::{Index,Add,AddAssign};
use std::cmp::{min,max,PartialEq,Eq,Ord,PartialOrd,Ordering};
use std::fmt;
use std::iter::{Iterator,FromIterator};
use std::collections::BinaryHeap;

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

impl<ScaleType,ValueType> Grid<ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

    pub fn new(width: ScaleType, height: ScaleType) -> Grid<ScaleType,ValueType> {
        Self::new_init(width, height, ValueType::zero())
    }

    pub fn new_init(width: ScaleType, height: ScaleType, value: ValueType) -> Grid<ScaleType,ValueType> {
        //create initial empty 2D grid
        let size = width.to_usize().unwrap() * height.to_usize().unwrap();
        let mut grid: Vec<ValueType> = Vec::with_capacity(size); //flattened grid
        for _ in 0..size {
            grid.push(value);
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

    ///Clones the grid with a different ValueType and runs the map function
    pub fn map_into<ToValueType>(&self, f: impl Fn(Point<ScaleType>, ToValueType) -> ToValueType ) -> Grid<ScaleType,ToValueType> where
        ToValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

        let mut clone: Grid<ScaleType,ToValueType> = Grid::new(self.width(), self.height());
        for (i, value) in self.data.iter().enumerate() {
            let point = self.point(i);
            //MAYBE TODO: add fallback conversion options?
            let tovalue = ToValueType::from_usize(value.to_usize().expect("map_into(): Unable to convert to usize")).expect("map_into(): unable to convert from usize");
            clone.set_index(i, f(point, tovalue));
        }
        clone
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

    pub fn map(mut self, f: impl Fn(Point<ScaleType>, ValueType) -> ValueType ) -> Self {
        let width = self.width_as_usize();
        for (i, value) in self.data.iter_mut().enumerate() {
            let y = i / width;
            let x = i % width;
            *value = f(Point::new_usize(x,y), *value);
        }
        self
    }

    ///Point to Index
    pub fn index(&self, point: &Point<ScaleType>) -> usize {
       (point.y() * self.width() + point.x()).to_usize().expect("Unable to cast to usize")
    }

    ///Index to Point
    pub fn point(&self, index: usize) -> Point<ScaleType> {
        let y = index / self.width_as_usize();
        let x = index % self.width_as_usize();
        Point(ScaleType::from_usize(x).unwrap(), ScaleType::from_usize(y).unwrap())
    }

    pub fn get(&self, point: &Point<ScaleType>) -> Option<&ValueType> {
        self.data.get(self.index(point))
    }

    pub fn get_mut(&mut self, point: &Point<ScaleType>) -> Option<&mut ValueType> {
        let index = self.index(point);
        self.data.get_mut(index)
    }

    pub fn inc(&mut self, point: &Point<ScaleType>, amount: ValueType) -> bool {
        let index = self.index(point);
        let mut value = self.data.get_mut(index).unwrap();
        if let Some(result) = value.checked_add(&amount) {
            *value = result;
            true
        } else {
            //value is saturated
            *value = ValueType::max_value();
            false
        }
    }

    pub fn dec(&mut self, point: &Point<ScaleType>, amount: ValueType) -> bool {
        let index = self.index(point);
        let mut value = self.data.get_mut(index).unwrap();
        if let Some(result) = value.checked_sub(&amount) {
            *value = result;
            true
        } else {
            //value is saturated
            *value = ValueType::min_value();
            false
        }
    }

    pub fn set(&mut self, point: &Point<ScaleType>, value: ValueType) -> bool {
        if let Some(v) = self.get_mut(point) {
            *v = value;
            return true;
        }
        return false;
    }

    pub fn set_index(&mut self, index: usize, value: ValueType) -> bool {
        if let Some(mut v) = self.data.get_mut(index) {
            *v = value;
            true
        } else {
            false
        }
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

    pub fn getneighbour(&self, point: &Point<ScaleType>, direction: Direction) -> Option<Point<ScaleType>> {
        if let Some(neighbour) = point.neighbour(direction, Some(self.width()), Some(self.height())) {
            Some(neighbour)
        } else {
            None
        }
    }

    pub fn getneighbours(&self, point: &Point<ScaleType>) -> Vec<Point<ScaleType>> {
       let mut neighbours = Vec::new();
       if let Some(neighbour) = self.getneighbour(point, Direction::North) { neighbours.push(neighbour); }
       if let Some(neighbour) = self.getneighbour(point, Direction::East) { neighbours.push(neighbour); }
       if let Some(neighbour) = self.getneighbour(point, Direction::South) { neighbours.push(neighbour); }
       if let Some(neighbour) = self.getneighbour(point, Direction::West) { neighbours.push(neighbour); }
       neighbours
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

    ///Creates a rectangular path (only horizontal and vertical) between points A and B
    pub fn rectpathto(&mut self, rng: &mut Pcg32, from: &Point<ScaleType>, to: &Point<ScaleType>, value: ValueType) {
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
                if self[&point] == ValueType::zero() { self.set(&point,value); };
            }
            for y in  yrange {
                let point = Point(to.x(), y );
                if self[&point] == ValueType::zero() { self.set(&point,value); };
            }
        } else {
            for y in  yrange {
                let point = Point(from.x(), y );
                if self[&point] == ValueType::zero() { self.set(&point,value); };
            }
            for x in  xrange {
                let point = Point(x,to.y() );
                if self[&point] == ValueType::zero() { self.set(&point,value); };
            }
        }

    }

    fn add(&mut self, other: &Self) {
        let width = min(self.width(), other.width());
        let height = min(self.height(), other.height());

        for x in range(ScaleType::zero(),width) {
            for y in range(ScaleType::zero(),height) {
                let point = Point(x,y);
                self.inc(&point, other[&point]);
            }
        }
    }

    fn sub(&mut self, other: &Self) {
        let width = min(self.width(), other.width());
        let height = min(self.height(), other.height());

        for x in range(ScaleType::zero(),width) {
            for y in range(ScaleType::zero(),height) {
                let point = Point(x,y);
                self.dec(&point, other[&point]);
            }
        }
    }

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

}


#[derive(Eq,PartialEq)]
struct PathState<ScaleType> {
   point: Point<ScaleType>,
   cost: u32
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
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

        type Output = ValueType;

        fn index(&self, point: &Point<ScaleType>) -> &Self::Output {
            self.get(point).expect("Out of bounds")
        }

}



impl<'a, ScaleType, ValueType> Iterator for GridIterator<'a, ScaleType,ValueType> where
    ScaleType: Integer + FromPrimitive + ToPrimitive + Bounded + Copy,
    ValueType: Num + FromPrimitive + ToPrimitive + PartialOrd + PartialEq + Bounded + CheckedAdd + CheckedSub + Copy {

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


