use std::{path::PathBuf, sync::{atomic::AtomicBool, Arc}, time::Instant, mem};

use anyhow::bail;
use flume::{Sender, Receiver};
use nbt::CompoundTag;

use crate::{game::{ChunkCoords, BlockPosition}, ecs::systems::world::light::{LightPropagationManager, LightPropagationRequest}, configuration::CONFIGURATION};

use self::region::RegionWorker;

use super::{chunks::{Chunk, SECTION_HEIGHT, CHUNK_WIDTH}, chunk_lock::ChunkHandle, generation::WorldGenerator, light::LightPropagator};

mod region;
#[derive(Debug)]
pub struct LoadRequest {
    pub pos: ChunkCoords,
}
#[derive(Debug)]
pub struct LoadedChunk {
    pub pos: ChunkCoords,
    pub chunk: Chunk,
    pub tile_entity_data: Vec<CompoundTag>,
    pub entity_data: Vec<CompoundTag>
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ChunkLoadResult {
    /// The chunk does not exist in this source.
    Missing(ChunkCoords),
    /// An error occurred while loading the chunk.
    Error(anyhow::Error),
    /// Successfully loaded the chunk.
    Loaded(LoadedChunk),
}

#[derive(Debug)]
pub struct SaveRequest {
    pub pos: ChunkCoords,
    pub chunk: ChunkHandle,
    pub block_entities: Vec<CompoundTag>, 
    pub entities: Vec<CompoundTag>,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum WorkerRequest {
    Load(LoadRequest),
    Save(SaveRequest),
}
pub struct ChunkWorker {
    generator: Arc<dyn WorldGenerator>,
    send_req: Sender<WorkerRequest>,
    send_gen: Sender<LoadedChunk>,
    recv_gen: Receiver<LoadedChunk>, // Chunk generation should be infallible.
    recv_load: Receiver<ChunkLoadResult>,
    world_seed: u64,
}

impl ChunkWorker {
    /// Helper function for poll_loaded_chunk. Attemts to receive a freshly generated chunk.
    /// Function signature identical to that of poll_loaded_chunk for ease of use.
    fn try_recv_gen(&mut self, light: &mut LightPropagationManager) -> Result<Option<LoadedChunk>, anyhow::Error> {
        let mut chunk = match self.recv_gen.try_recv() {
            Ok(l) => l,
            Err(e) => match e {
                flume::TryRecvError::Empty => return Ok(None),
                flume::TryRecvError::Disconnected => bail!("chunkgen channel died"),
            },
        };
        light.push(LightPropagationRequest::ChunkSky { position: chunk.pos, world: chunk.pos.world });
        // for (idx, v) in chunk.chunk.data.iter().enumerate() {
        //     if let Some(v) = v {
        //         for (x, y, z) in v.lights() {
        //             light.push(LightPropagationRequest::BlockLight { position: BlockPosition::new(*x as i32 + (chunk.pos.x as i32 * CHUNK_WIDTH as i32), *y as i32 + (idx as i32 * SECTION_HEIGHT as i32), *z as i32 + (chunk.pos.z as i32 * CHUNK_WIDTH as i32), chunk.pos.world), was_source: 0 });
        //         }
        //     }
        // }
        // let v = mem::take(&mut chunk.light);
        // for v in v {
        //     light.push(v);
        // }
        Ok(Some(chunk))
    }
    pub fn drop_sender(&mut self) {
        let (send_req, _) = flume::unbounded();
        let (_, recv_load) = flume::unbounded();
        self.send_req = send_req;
        self.recv_load = recv_load;
    }
    pub fn new(world_dir: impl Into<PathBuf>, seed: u64, generator: Arc<dyn WorldGenerator>, shutdown: Arc<AtomicBool>) -> Self {
        let (send_req, recv_req) = flume::unbounded();
        let (send_gen, recv_gen) = flume::unbounded();
        let (region_worker, recv_load) = RegionWorker::new(world_dir.into(), recv_req, shutdown);
        region_worker.start();
        Self {
            send_req,
            send_gen,
            recv_gen,
            recv_load,
            generator,
            world_seed: seed
        }
    }
    pub fn queue_load(&mut self, request: LoadRequest) {
        self.send_req.send(WorkerRequest::Load(request)).unwrap()
    }
    pub fn poll_loaded_chunk(&mut self, light: &mut LightPropagationManager) -> Result<Option<LoadedChunk>, anyhow::Error> {
        match self.recv_load.try_recv() {
            Ok(answer) => {
                match answer {
                    // RegionWorker answered
                    ChunkLoadResult::Missing(pos) => {
                        // chunk does not exist, queue it for generation
                        let send_gen = self.send_gen.clone();
                        let gen = self.generator.clone();
                        let seed = self.world_seed;
                        rayon::spawn(move || {
                            // spawn task to generate chunk
                            if CONFIGURATION.logging.chunk_gen {
                                log::info!("Generating chunk at {}", pos);
                            }
                            let mut chunk = gen.generate_chunk(pos, seed);
                            chunk.heightmaps.recalculate(Chunk::block_at_fn(&chunk.data));
                            send_gen.send(LoadedChunk { pos, chunk, tile_entity_data: Vec::new(), entity_data: Vec::new() }).unwrap()
                        });
                        self.try_recv_gen(light) // check for generated chunks
                    }
                    ChunkLoadResult::Error(e) => Err(e),
                    ChunkLoadResult::Loaded(mut l) => {
                        //log::info!("Starting");
                        //let start = Instant::now();
                        //l.chunk.heightmaps.recalculate(Chunk::block_at_fn(&l.chunk.data));
                        //log::info!("Recalculation took {}ms", start.elapsed().as_millis());
                        Ok(Some(l))
                    },
                }
            }
            Err(e) => match e {
                flume::TryRecvError::Empty => self.try_recv_gen(light), // check for generated chunks
                flume::TryRecvError::Disconnected => panic!("RegionWorker died"),
            },
        }
    }

    pub fn queue_chunk_save(&mut self, req: SaveRequest) {
        self.send_req.send(WorkerRequest::Save(req)).unwrap()
    }
}