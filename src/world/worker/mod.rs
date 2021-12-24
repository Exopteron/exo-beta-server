use std::{path::PathBuf, sync::{atomic::AtomicBool, Arc}, time::Instant};

use anyhow::bail;
use flume::{Sender, Receiver};
use nbt::CompoundTag;

use crate::{game::{ChunkCoords, BlockPosition}, ecs::systems::world::light::{LightPropagationManager, LightPropagationRequest}};

use self::region::RegionWorker;

use super::{chunks::Chunk, chunk_lock::ChunkHandle, generation::WorldGenerator};

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
/*     pub entities: Vec<EntityData>,
    pub block_entities: Vec<BlockEntityData>, 
    TODO: entities later */
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
}

impl ChunkWorker {
    /// Helper function for poll_loaded_chunk. Attemts to receive a freshly generated chunk.
    /// Function signature identical to that of poll_loaded_chunk for ease of use.
    fn try_recv_gen(&mut self, light: &mut LightPropagationManager) -> Result<Option<LoadedChunk>, anyhow::Error> {
        let chunk = match self.recv_gen.try_recv() {
            Ok(l) => l,
            Err(e) => match e {
                flume::TryRecvError::Empty => return Ok(None),
                flume::TryRecvError::Disconnected => bail!("chunkgen channel died"),
            },
        };
        for x in 0..16 {
            for z in 0..16 {
                if let Some(y) = chunk.chunk.heightmaps.light_blocking.height(x, z) {
                    let pos = BlockPosition::new((x + (chunk.pos.x as usize) * 16) as i32, (y + 0) as i32, (z + (chunk.pos.z as usize) * 16) as i32);
                    //light.push(LightPropagationRequest { position: pos, world: 0, level: 15, skylight: true }); //TODO: unhardcodeworld
                }
            }
        }
        Ok(Some(chunk))
    }
    pub fn drop_sender(&mut self) {
        let (send_req, _) = flume::unbounded();
        let (_, recv_load) = flume::unbounded();
        self.send_req = send_req;
        self.recv_load = recv_load;
    }
    pub fn new(world_dir: impl Into<PathBuf>, generator: Arc<dyn WorldGenerator>, shutdown: Arc<AtomicBool>) -> Self {
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
                        rayon::spawn(move || {
                            // spawn task to generate chunk
                            let mut chunk = gen.generate_chunk(pos);
                            chunk.heightmaps.recalculate(Chunk::block_at_fn(&chunk.data));
                            //chunk.calculate_full_skylight();
                            send_gen.send(LoadedChunk { pos, chunk, tile_entity_data: Vec::new() }).unwrap()
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