#![warn(missing_docs, clippy::pedantic, clippy::perf)]
#![doc = include_str!(r"../README.md")]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::ops::{Add, Mul};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[allow(missing_docs)]
/// A four-dimensional position of an object in a scene.
pub struct Position {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub t: i64
}

impl Add<i64> for Position {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        Position {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
            t: self.t + rhs,
        }
    }
}

impl Mul<f64> for Position {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Position {
            x: ((self.x as f64) * rhs) as i64,
            y: ((self.y as f64) * rhs) as i64,
            z: ((self.z as f64) * rhs) as i64,
            t: ((self.t as f64) * rhs) as i64,
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.z.cmp(&other.z)
            .then(self.y.cmp(&other.y))
            .then(self.x.cmp(&other.x))
            .then(self.t.cmp(&other.t))
    }
}

/// A trait marking something as a scene object.
pub trait Object {}

/// A whole scene.
pub struct Scene<O: Object> {
    map: ObjectMap<O>,
    flags: HashMap<String, String>
}

/// A sparse grid of objects in a scene.
pub struct ObjectMap<O: Object> {
    /// The width of the map.
    pub width: u64,
    /// The height of the map.
    pub height: u64,
    /// The time length of the map.
    pub length: u64,
    /// A map of positions to objects.
    pub objects: BTreeMap<Position, O>
}
