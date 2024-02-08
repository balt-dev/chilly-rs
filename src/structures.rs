//! Holds a few data structures for general use.

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::ops::{Add, Mul};
use num_traits::Num;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[allow(missing_docs)]
/// A four-dimensional position of an object in a scene.
pub struct Position<N: Num> {
    pub x: N,
    pub y: N,
    pub z: N,
    pub t: N
}

impl<N> Position<N> where N: Num {
    /// Converts this position into another numeric representation.
    ///
    /// # Notes
    /// This cannot be a basic From implementation, as From can't blanket all types.
    fn into<O: From<N> + Num>(self) -> Position<O> {
        Position {
            x: self.x.into(),
            y: self.y.into(),
            z: self.z.into(),
            t: self.t.into()
        }
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

impl<N: Ord + Num> PartialOrd for Position<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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

/// A whole scene.
#[derive(Debug, Default)]
pub struct Scene<O: Object, N: Num> {
    /// A tilemap of the objects in the scene.
    pub map: ObjectMap<O, N>,
    /// The attached flags of the scene.
    pub flags: HashMap<String, Option<String>>
}

/// A sparse grid of objects in a scene.
#[derive(Debug, Default)]
pub struct ObjectMap<O: Object, N: Num> {
    /// The width of the map.
    pub width: usize,
    /// The height of the map.
    pub height: usize,
    /// The time length of the map.
    pub length: usize,
    /// A map of positions to objects.
    pub objects: BTreeMap<Position<N>, O>
}
