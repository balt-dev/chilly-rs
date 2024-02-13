//! Holds a few data structures for general use.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Mul};
use num_traits::Num;

#[derive(Copy, Clone, PartialEq, Eq, Default, Hash)]
#[allow(missing_docs)]
/// A four-dimensional position of an object in a scene.
pub struct Position<N: Num> {
    pub x: N,
    pub y: N,
    pub z: N,
    pub t: N
}

impl<N: Debug + Num> Debug for Position<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "Position {{ x: {:?}, y: {:?}, z: {:?}, t: {:?} }}",
            self.x, self.y, self.z, self.t
        )
    }
}

impl<N: Display + Num> Display for Position<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{{ {}, {}, {}, {} }}",
            self.x, self.y, self.z, self.t
        )
    }
}

impl<N: Num + Into<A>, A: Num + Copy> Add<A> for Position<N> {
    type Output = Position<A>;

    fn add(self, rhs: A) -> Self::Output {
        Position {
            x: self.x.into() + rhs,
            y: self.y.into() + rhs,
            z: self.z.into() + rhs,
            t: self.t.into() + rhs,
        }
    }
}

impl<N: Num + Into<A>, A: Num + Copy> Mul<A> for Position<N> {
    type Output = Position<A>;

    fn mul(self, rhs: A) -> Self::Output {
        Position {
            x: self.x.into() * rhs,
            y: self.y.into() * rhs,
            z: self.z.into() * rhs,
            t: self.t.into() * rhs,
        }
    }
}

impl<N: PartialOrd + Num> PartialOrd for Position<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.z.partial_cmp(&other.z)
            .and_then(|ord| Some(ord.then(self.y.partial_cmp(&other.y)?)))
            .and_then(|ord| Some(ord.then(self.x.partial_cmp(&other.x)?)))
            .and_then(|ord| Some(ord.then(self.t.partial_cmp(&other.t)?)))
        }
}

impl<N: Ord + Num> Ord for Position<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.z.cmp(&other.z)
            .then(self.y.cmp(&other.y))
            .then(self.x.cmp(&other.x))
            .then(self.t.cmp(&other.t))
    }
}

/// A trait marking something as a scene object.
pub trait Object {}

/// A sparse grid of objects in a scene.
#[derive(Debug, Clone)]
pub struct ObjectMap<O: Object, N: Num> {
    /// The width of the map.
    pub width: usize,
    /// The height of the map.
    pub height: usize,
    /// The time length of the map.
    pub length: usize,
    /// A map of positions to objects.
    pub objects: HashMap<Position<N>, O>
}

impl<N: Num, O: Object> Default for ObjectMap<O, N> {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            length: 0,
            objects: HashMap::new()
        }
    }
}
