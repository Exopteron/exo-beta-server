use crate::game::{Position, BlockPosition};

pub struct AABBPool {
    list: Vec<AABB>,
}
impl AABBPool {
    pub fn new() -> Self {
        Self {
            list: Vec::new(),
        }
    }
    pub fn add(&mut self, aabb: AABB) {
        self.list.push(aabb);
    }
    pub fn intersects(&self, other: &AABB) -> Vec<AABB> {
        let mut intersections = Vec::new();
        for aabb in self.list.iter() {
            if aabb.intersects(other) {
                intersections.push(aabb.clone());
            }
        }
        intersections
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    minx: f64,
    miny: f64,
    minz: f64,
    maxx: f64,
    maxy: f64,
    maxz: f64
}
pub struct AABBSize {
    pub minx: f64,
    pub miny: f64,
    pub minz: f64,
    pub maxx: f64,
    pub maxy: f64,
    pub maxz: f64
}
impl AABBSize {
    pub fn new(minx: f64, miny: f64, minz: f64, maxx: f64, maxy: f64, maxz: f64) -> Self {
        Self { minx, miny, minz, maxx, maxy, maxz }
    }
    pub fn get_from_block(&self, position: &BlockPosition) -> AABB {
        let position = Position::from_pos(position.x as f64, position.y as f64, position.z as f64);
        self.get(&position)
    }
    pub fn get(&self, position: &Position) -> AABB {
        AABB::new(position.x + self.minx, position.y + self.miny, position.z + self.minz, position.x + self.maxx, position.y + self.maxy, position.z + self.maxz)
    }
    pub fn set_bounds(&mut self, minx: f64, miny: f64, minz: f64, maxx: f64, maxy: f64, maxz: f64) {
        self.minx = minx;
        self.miny = miny;
        self.minz = minz;
        self.maxx = maxx;
        self.maxy = maxy;
        self.maxz = maxz;
    }
}
impl AABB {
    pub fn new(minx: f64, miny: f64, minz: f64, maxx: f64, maxy: f64, maxz: f64) -> Self {
        Self { minx, miny, minz, maxx, maxy, maxz }
    }
    pub fn set_bounds(&mut self, minx: f64, miny: f64, minz: f64, maxx: f64, maxy: f64, maxz: f64) {
        self.minx = minx;
        self.miny = miny;
        self.minz = minz;
        self.maxx = maxx;
        self.maxy = maxy;
        self.maxz = maxz;
    }
    pub fn intersects(&self, other: &AABB) -> bool {
        return (self.minx <= other.maxx && self.maxx >= other.minx) && (self.miny <= other.maxy && self.maxy >= other.miny) && (self.minz <= other.maxz && self.maxz >= other.minz)
    }
}
