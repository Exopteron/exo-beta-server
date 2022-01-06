use glam::Vec3;

use crate::{
    game::{BlockPosition, Position},
    protocol::packets::Face,
};

#[derive(Default)]
pub struct AABBPool {
    list: Vec<AABB>,
}
impl AABBPool {
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
#[derive(Clone, Copy)]
pub struct AABBSize {
    pub minx: f64,
    pub miny: f64,
    pub minz: f64,
    pub maxx: f64,
    pub maxy: f64,
    pub maxz: f64,
}
impl AABBSize {
    pub fn new(minx: f64, miny: f64, minz: f64, maxx: f64, maxy: f64, maxz: f64) -> Self {
        Self {
            minx,
            miny,
            minz,
            maxx,
            maxy,
            maxz,
        }
    }
    pub fn get_from_block(&self, position: &BlockPosition) -> AABB {
        let position = Position::from_pos(
            position.x as f64,
            position.y as f64,
            position.z as f64,
            position.world,
        );
        self.get(&position)
    }
    pub fn get(&self, position: &Position) -> AABB {
        AABB::new(
            position.x + self.minx,
            position.y + self.miny,
            position.z + self.minz,
            position.x + self.maxx,
            position.y + self.maxy,
            position.z + self.maxz,
        )
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
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub minx: f64,
    pub miny: f64,
    pub minz: f64,
    pub maxx: f64,
    pub maxy: f64,
    pub maxz: f64,
}
pub type SweeptestOutput = (f64, (f64, f64, f64));
impl AABB {
    pub fn get_position(&self, size: &AABBSize, world: i32) -> Position {
        let x = self.minx - size.minx;
        let y = self.miny - size.miny;
        let z = self.minz - size.minz;
        Position::from_pos(x, y, z, world)
    }
    pub fn get_size(&self, position: Position) -> AABBSize {
        AABBSize::new(self.minx - position.x, self.miny - position.y, self.minz - position.z, self.maxx - position.x, self.maxy - position.y, self.maxz - position.z)
    }
    pub fn new(minx: f64, miny: f64, minz: f64, maxx: f64, maxy: f64, maxz: f64) -> Self {
        Self {
            minx,
            miny,
            minz,
            maxx,
            maxy,
            maxz,
        }
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
        (self.minx <= other.maxx && self.maxx >= other.minx)
            && (self.miny <= other.maxy && self.maxy >= other.miny)
            && (self.minz <= other.maxz && self.maxz >= other.minz)
    }
    pub fn collisions(&self, other: &AABB) -> Vec<Face> {
        let mut faces = Vec::new();
        if !self.intersects(other) {
            return faces;
        }
        if self.minx <= other.maxx && self.maxx >= other.minx {
            // x axis collision
            if self.maxx >= other.maxx {
                faces.push(Face::NegativeX);
            } else {
                faces.push(Face::PositiveX);
            }
        }

        if self.miny <= other.maxy && self.maxy >= other.miny {
            // y axis collision
            if self.maxy >= other.maxy {
                faces.push(Face::NegativeY);
            } else {
                faces.push(Face::PositiveY);
            }
        }

        if self.minz <= other.maxz && self.maxz >= other.minz {
            // z axis collision
            if self.maxz >= other.maxz {
                faces.push(Face::NegativeZ);
            } else {
                faces.push(Face::PositiveZ);
            }
        }
        faces
    }
    pub fn swept_aabb(b1: AABB, b2: AABB, velocity: Vec3) -> SweeptestOutput {
        let mut normalx = 0.;
        let mut normaly = 0.;
        let mut normalz = 0.;

        let mut xinventry = 0.;
        let mut yinventry = 0.;
        let mut zinventry = 0.;

        let mut xinvexit = 0.;
        let mut yinvexit = 0.;
        let mut zinvexit = 0.;

        if velocity.x > 0. {
            xinventry = b2.minx - (b1.maxx);
            xinvexit = (b2.maxx) - b1.minx;
        } else {
            xinventry = (b2.maxx) - b1.minx;
            xinvexit = b2.minx - (b1.maxx);
        }

        if velocity.y > 0. {
            yinventry = b2.miny - (b1.maxy);
            yinvexit = (b2.maxy) - b1.miny;
        } else {
            yinventry = (b2.maxy) - b1.miny;
            yinvexit = b2.miny - (b1.maxy);
        }

        if velocity.z > 0. {
            zinventry = b2.minz - (b1.maxz);
            zinvexit = (b2.maxz) - b1.minz;
        } else {
            zinventry = (b2.maxz) - b1.minz;
            zinvexit = b2.minz - (b1.maxz);
        }

        let mut xentry = 0.;
        let mut yentry = 0.;
        let mut zentry = 0.;

        let mut xexit = 0.;
        let mut yexit = 0.;
        let mut zexit = 0.;

        if velocity.x == 0. {
            xentry = f64::NEG_INFINITY;
            xexit = f64::INFINITY;
        } else {
            xentry = xinventry / velocity.x as f64;
            xexit = xinvexit / velocity.x as f64;
        }

        if velocity.y == 0. {
            yentry = f64::NEG_INFINITY;
            yexit = f64::INFINITY;
        } else {
            yentry = yinventry / velocity.y as f64;
            yexit = yinvexit / velocity.y as f64;
        }

        if velocity.z == 0. {
            zentry = f64::NEG_INFINITY;
            zexit = f64::INFINITY;
        } else {
            zentry = zinventry / velocity.z as f64;
            zexit = zinvexit / velocity.z as f64;
        }
        let mut entrytime = (xentry.max(yentry)).max(zentry);
        let mut exittime = (xexit.min(yexit)).min(zexit);

        if entrytime > exittime
            || (xentry < 0. && yentry < 0. && zentry < 0.)
            || xentry > 1.
            || yentry > 1.
            || zentry > 1.
        {
            //normalx = 0.;
            //normaly = 0.;
            //normalz = 0.;
            return (1., (normalx, normaly, normalz));
        } else if ((xentry.max(yentry)).max(zentry) - xentry).abs() < f64::EPSILON {
            if xinventry < 0. {
                normalx = 1.0;
                //normaly = 0.0;
                //normalz = 0.0;
            } else {
                normalx = -1.0;
            }
            //normaly = 0.0;
            //normalz = 0.0;
        } else if ((xentry.max(yentry)).max(zentry) - yentry).abs() < f64::EPSILON {
            //normalx = 0.0;
            if yinventry < 0.0 {
                normaly = 1.0;
                //normalz = 0.0;
            } else {
                //normalx = 0.0;
                normaly = -1.0;
            }
            //normalz = 0.0;
        } else if zinventry < 0.0 {
            //normalx = 0.0;
            //normaly = 0.0;
            //normalz = 1.0;
        } else {
            //normalx = 0.0;
            //normaly = 0.0;
            normalz = -1.0;
        }

        (entrytime, (normalx, normaly, normalz))
    }
}
#[test]
fn test() {
    let mut aabb1 = AABB::new(1., 1., 1., 2., 2., 2.);
    let mut aabb2 = AABB::new(0., 0., 0., 0.5, 1.5, 0.5);
    let collisions = aabb1.collisions(&aabb2);
    panic!("Collisions: {:?}", collisions);
}
