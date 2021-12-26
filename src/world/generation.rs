use std::hash::Hasher;

use rand::{SeedableRng, Rng, RngCore};
use rand_xorshift::XorShiftRng;
use siphasher::sip128::SipHasher13;
use worldgen::{noise::perlin::PerlinNoise, noisemap::{NoiseMap, NoiseMapGenerator, Size, Seed, Step, NoiseMapGeneratorBase}};

use crate::{game::ChunkCoords, configuration::CONFIGURATION};

use super::chunks::{Chunk, BlockState};

pub trait WorldGenerator: Send + Sync {
    /// Generates the chunk at the given position.
    fn generate_chunk(&self, position: ChunkCoords) -> Chunk;
}

pub struct EmptyWorldGenerator {}

impl WorldGenerator for EmptyWorldGenerator {
    fn generate_chunk(&self, position: ChunkCoords) -> Chunk {
        Chunk::new(position)
    }
}

const GROUND_LEVEL: usize = 65;
pub struct FlatWorldGenerator {}

impl WorldGenerator for FlatWorldGenerator {
    fn generate_chunk(&self, position: ChunkCoords) -> Chunk {
        let mut chunk = Chunk::new(position);
        for x in 0..16 {
            for z in 0..16 {
                chunk.set_block_at(x, GROUND_LEVEL, z, BlockState::from_id(7));
                chunk.set_block_at(x, GROUND_LEVEL + 1, z, BlockState::from_id(3));
                chunk.set_block_at(x, GROUND_LEVEL + 2, z, BlockState::from_id(2));
            }
        }
        chunk
    }
}

const WATER_HEIGHT: i32 = 25;
pub struct TerrainWorldGenerator {}

impl WorldGenerator for TerrainWorldGenerator {
    fn generate_chunk(&self, position: ChunkCoords) -> Chunk {
        let seed = CONFIGURATION.world_seed.unwrap();
        let mut chunk = Chunk::new(position);
        let mut hash = SipHasher13::new_with_keys(seed, seed);
        hash.write_i32(position.x);
        hash.write_i32(position.z);
        let hash = hash.finish();
        let mut rng = XorShiftRng::seed_from_u64(hash);
        let noise = PerlinNoise::new();
        let nm = NoiseMap::new(noise)
            .set(Size::of(16, 16))
            .set(Seed::of_value(seed))
            .set(Step::of(-0.01, 0.01));
        let nm2 = NoiseMap::new(noise)
            .set(Size::of(16, 16))
            .set(Seed::of_value(seed))
            .set(Step::of(0.05, 0.05));
        let nm = ((nm * 5)  + nm2) * 25;
        let noise = nm.generate_chunk(-(position.z as i64), -(position.x as i64));
        let mut noisevec = Vec::with_capacity(noise.len() * 2);
        for row in noise {
            for value in row.into_iter() {
                noisevec.push(value);
            }
        }
        for x in 0..16 {
            for z in 0..16 {
                if noisevec.len() <= 0 {
                    break;
                }
                let mut num = noisevec.pop().unwrap() as i32;
                num += 40;
                if num < WATER_HEIGHT {
                    chunk.set_block_at(x, num as usize, z, BlockState::from_id(13));
                } else {
                    chunk.set_block_at(x, num as usize, z, BlockState::from_id(2));
                }
                for y in 0..num - 3 {
                    chunk.set_block_at(x, y as usize, z, BlockState::from_id(1));
                }
                for y in num - 3..num {
                    chunk.set_block_at(x, y as usize, z, BlockState::from_id(3));
                }
            }
        }
        for y in 0..WATER_HEIGHT {
            chunk.fill_layer_air(y as usize, BlockState::from_id(9));
        }
        chunk.fill_layer(0, BlockState::from_id(7));
        chunk
    }
}

pub struct CustomWorldGenerator {
    pub x_step: f64,
    pub y_step: f64,
    pub multiplication_factor: i64,
}

impl WorldGenerator for CustomWorldGenerator {
    fn generate_chunk(&self, position: ChunkCoords) -> Chunk {
        let seed = CONFIGURATION.world_seed.unwrap();
        let mut chunk = Chunk::new(position);
        let mut hash = SipHasher13::new_with_keys(seed, seed);
        hash.write_i32(position.x);
        hash.write_i32(position.z);
        let hash = hash.finish();
        let rng = XorShiftRng::seed_from_u64(hash);
        let noise = PerlinNoise::new();
        let nm = NoiseMap::new(noise)
            .set(Size::of(16, 16))
            .set(Seed::of_value(seed))
            .set(Step::of(self.x_step, self.y_step));
        let nm = nm * self.multiplication_factor;
        let noise = nm.generate_chunk(-(position.z as i64), -(position.x as i64));
        let mut noisevec = Vec::with_capacity(noise.len() * 2);
        for row in noise {
            for value in row.into_iter() {
                noisevec.push(value);
            }
        }
        for x in 0..16 {
            for z in 0..16 {
                if noisevec.len() <= 0 {
                    break;
                }
                let mut num = noisevec.pop().unwrap() as i32;
                num += 40;
                if num < WATER_HEIGHT {
                    chunk.set_block_at(x, num as usize, z, BlockState::from_id(13));
                } else {
                    chunk.set_block_at(x, num as usize, z, BlockState::from_id(2));
                }
                for y in 0..num - 3 {
                    chunk.set_block_at(x, y as usize, z, BlockState::from_id(1));
                }
                for y in num - 3..num {
                    chunk.set_block_at(x, y as usize, z, BlockState::from_id(3));
                }
            }
        }
        for y in 0..WATER_HEIGHT {
            chunk.fill_layer_air(y as usize, BlockState::from_id(9));
        }
        chunk.fill_layer(0, BlockState::from_id(7));
        chunk
    }
}
pub struct MountainWorldGenerator;

impl WorldGenerator for MountainWorldGenerator {
    fn generate_chunk(&self, position: ChunkCoords) -> Chunk {
        let seed = CONFIGURATION.world_seed.unwrap();
        let mut chunk = Chunk::new(position);
        let mut hash = SipHasher13::new_with_keys(seed, seed);
        hash.write_i32(position.x);
        hash.write_i32(position.z);
        let hash = hash.finish();
        let mut rng = XorShiftRng::seed_from_u64(hash);
        let noise = PerlinNoise::new();
        let nm = NoiseMap::new(noise)
            .set(Size::of(16, 16))
            .set(Seed::of_value(seed))
            .set(Step::of(-0.01, 0.01));
        let nm = nm * 50;
        let noise = nm.generate_chunk(-(position.z as i64), -(position.x as i64));
        let mut noisevec = Vec::with_capacity(noise.len() * 2);
        for row in noise {
            for value in row.into_iter() {
                noisevec.push(value);
            }
        }
        for x in 0..16 {
            for z in 0..16 {
                if noisevec.len() <= 0 {
                    break;
                }
                let mut num = noisevec.pop().unwrap() as i32;
                num += 40;
                if num < WATER_HEIGHT {
                    chunk.set_block_at(x, num as usize, z, BlockState::from_id(13));
                } else {
                    chunk.set_block_at(x, num as usize, z, BlockState::from_id(2));
                }
                for y in 0..num - 3 {
                    chunk.set_block_at(x, y as usize, z, BlockState::from_id(1));
                }
                for y in num - 3..num {
                    chunk.set_block_at(x, y as usize, z, BlockState::from_id(3));
                }
            }
        }
        for y in 0..WATER_HEIGHT {
            chunk.fill_layer_air(y as usize, BlockState::from_id(9));
        }
        chunk.fill_layer(0, BlockState::from_id(7));
        chunk
    }
}