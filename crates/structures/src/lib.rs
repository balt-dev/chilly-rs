#![warn(missing_docs, clippy::pedantic, clippy::perf)]
#![doc = include_str!(r"../README.md")]

use std::collections::{BTreeMap, HashMap};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
/// A four-dimensional position of an object in a scene.
pub struct Position {
    pub x: u64,
    pub y: u64,
    pub z: u64,
    pub t: u64
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