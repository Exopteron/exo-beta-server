// feather license in FEATHER_LICENSE.md
use std::{path::PathBuf, time::{Instant, Duration}, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use ahash::AHashMap;
use anvil_region::{provider::FolderRegionProvider, position::RegionPosition};
use flume::{Receiver, Sender};

use crate::{game::{ChunkCoords, Game}, configuration::CONFIGURATION};

use crate::world::{chunks::Chunk, chunk_lock::ChunkHandle, mcregion::MCRegionLoader};

use super::{WorkerRequest, ChunkLoadResult, SaveRequest, LoadRequest, LoadedChunk};

const CACHE_TIME: Duration = Duration::from_secs(60);
pub struct RegionWorker {
    request_receiver: Receiver<WorkerRequest>,
    result_sender: Sender<ChunkLoadResult>,
    region_provider: MCRegionLoader,
    shutdown: Arc<AtomicBool>,
}

impl RegionWorker {
    pub fn new(
        world_dir: PathBuf,
        request_receiver: Receiver<WorkerRequest>,
        shutdown: Arc<AtomicBool>,
    ) -> (Self, Receiver<ChunkLoadResult>) {
        let (result_sender, result_receiver) = flume::bounded(256);
        (
            Self {
                request_receiver,
                result_sender,
                region_provider: MCRegionLoader::new(world_dir.as_path().to_str().unwrap()).unwrap(),
                shutdown,
            },
            result_receiver,
        )
    }

    pub fn start(self) {
        std::thread::Builder::new()
            .name("chunk_worker".to_owned())
            .spawn(move || self.run())
            .expect("failed to create chunk worker thread");
    }

    fn run(mut self) {
        log::info!("Chunk worker started");
        loop {
            let x = self.request_receiver.recv_timeout(CACHE_TIME);
            //log::info!("Got request {:?}", x);
            match x {
                Ok(req) => match req {
                    WorkerRequest::Load(load) => self.load_chunk(load),
                    WorkerRequest::Save(save) => self.save_chunk(save).unwrap(),
                },
                Err(flume::RecvTimeoutError::Timeout) => (),
                Err(flume::RecvTimeoutError::Disconnected) => {
                    log::info!("Chunk worker shutting down");
                    self.shutdown.store(true, Ordering::Relaxed);
                    return;
                }
            }
        }
    }
    fn save_chunk(&mut self, req: SaveRequest) -> anyhow::Result<()> {
        let reg_pos = RegionPosition::from_chunk_position(req.pos.x, req.pos.z);
        self.region_provider.save_chunk(req.chunk, req.block_entities, req.entities)?;
        Ok(())
    }

    fn load_chunk(&mut self, req: LoadRequest) {
        if CONFIGURATION.logging.chunk_load {
            log::info!("Loading chunk at {}", req.pos);
        }
        let result = self.get_chunk_load_result(req);
        let _ = self.result_sender.send(result);
    }

    fn get_chunk_load_result(&mut self, req: LoadRequest) -> ChunkLoadResult {
        let pos = req.pos;
        let region = RegionPosition::from_chunk_position(pos.x, pos.z);
        self.region_provider.get_chunk(&pos)
    }
}
