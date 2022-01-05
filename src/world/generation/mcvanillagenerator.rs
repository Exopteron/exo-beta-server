use std::{convert::TryFrom, path::PathBuf, sync::Arc, thread};

use ahash::AHashMap;
use flume::{Receiver, Sender};
use j4rs::{ClasspathEntry, Instance, InvocationArg, Jvm, JvmBuilder};
use parking_lot::Mutex;
use serde::Deserialize;

use crate::{
    configuration::CONFIGURATION,
    game::ChunkCoords,
    world::{
        chunks::Chunk,
        heightmap::HeightmapStore,
        mcregion::{vec_i8_into_u8, Region},
    }, jvm::JVMSetup,
};

use super::WorldGenerator;

pub enum VWGCommand {
    LoadChunk(i32, i32),
}
pub struct VLoadedChunk {
    chunk: RustableChunk,
    coords: ChunkCoords,
}
pub struct VanillaWorldGenerator {
    sender: Sender<VWGCommand>,
    receiver: Receiver<VLoadedChunk>,
    cache: Mutex<AHashMap<ChunkCoords, RustableChunk>>,
    world_type: i32,
}
impl VanillaWorldGenerator {
    pub fn new(seed: u64, world_type: i8) -> anyhow::Result<Self> {
        let (send_cmd, recv_command) = flume::unbounded();
        let (send_chunk, recv_chunk) = flume::unbounded();
        thread::spawn(move || {
            let task = VanillaWorldGeneratorTask::new(seed, world_type as i32, send_chunk, recv_command).unwrap();
            if let Err(e) = task.run() {
                log::error!("ERROR: {:?}", e);
            }
        });
        Ok(Self {
            sender: send_cmd,
            receiver: recv_chunk,
            cache: Mutex::new(AHashMap::new()),
            world_type: world_type as i32,
        })
    }
}
impl WorldGenerator for VanillaWorldGenerator {
    fn generate_chunk(&self, position: ChunkCoords, seed: u64) -> crate::world::chunks::Chunk {
        let mut cache = self.cache.lock();
        let rustable_chunk = if let Some(c) = cache.remove(&position) {
            c
        } else {
            self.sender
                .send(VWGCommand::LoadChunk(position.x, position.z))
                .unwrap();
            let mut chunk: Option<VLoadedChunk> = None;
            loop {
                match self.receiver.recv() {
                    Ok(c) => {
                        //log::info!("Got chunk {} looking fo", c.coords);
                        if c.coords == position {
                            chunk = Some(c);
                            break;
                        } else {
                            cache.insert(c.coords, c.chunk);
                        }
                    }
                    Err(e) => {
                        log::error!("RECVERROR: {:?}", e);
                        break;
                    }
                }
            }
            if chunk.is_none() {
                panic!("No chunk");
            }
            chunk.unwrap().chunk
        };
        rustable_chunk.to_chunk(self.world_type).unwrap()
    }
}
pub struct VanillaWorldGeneratorTask {
    interface: Instance,
    sender: Sender<VLoadedChunk>,
    receiver: Receiver<VWGCommand>,
    world_type: i32,
}
impl VanillaWorldGeneratorTask {
    pub fn new(
        seed: u64,
        world_type: i32,
        sender: Sender<VLoadedChunk>,
        receiver: Receiver<VWGCommand>,
    ) -> anyhow::Result<Self> {
        if let Ok(jvm) = Jvm::attach_thread() {
            let interface = jvm.create_instance(
                "com.exopteron.ChunkInterface",
                &[
                    InvocationArg::try_from(seed as i64)?,
                    InvocationArg::try_from(world_type)?,
                ],
            )?;
            Ok(Self {
                interface,
                sender,
                receiver,
                world_type
            })
        } else {
            Err(anyhow::anyhow!("No JVM"))
        }
    }
    pub fn run(self) -> anyhow::Result<()> {
        let jvm = Jvm::attach_thread().unwrap();
        while let Ok(cmd) = self.receiver.recv() {
            match cmd {
                VWGCommand::LoadChunk(x, z) => {
                    //log::info!("Got cmd for {}, {}", x, z);
                    let instance = jvm.invoke(
                        &self.interface,
                        "genChunk",
                        &[InvocationArg::try_from(x)?, InvocationArg::try_from(z)?],
                    )?;
                    let obj = instance.java_object();
                    if obj.is_null() {
                        log::info!("ALERT: NULL");
                    }
                    let instance = Instance::from_jobject(obj)?;
                    //log::info!("Successful invocation");
                    let chunk: RustableChunk = match jvm.to_rust(instance) {
                        Ok(c) => c,
                        Err(e) => {
                            log::error!("CONVERSION ERROR: {:?}", e);
                            return Err(anyhow::anyhow!("ERROR: {:?}", e));
                        }
                    };
                    //log::info!("Successful conversion");
                    self.sender.send(VLoadedChunk {
                        chunk,
                        coords: ChunkCoords::new(x, z, self.world_type),
                    })?;
                    //log::info!("Sent chunk");
                }
            }
        }
        log::info!("Returning");
        Ok(())
    }
}
#[derive(Deserialize)]
pub struct RustableChunk {
    blocks: Vec<i8>,
    data: Vec<i8>,
    x: i32,
    z: i32,
}

impl RustableChunk {
    pub fn to_chunk(&self, world_type: i32) -> anyhow::Result<Chunk> {
        let block_ids = vec_i8_into_u8(self.blocks.clone());
        if block_ids.len() == 0 {
            return Err(anyhow::anyhow!("0 length blocks"));
        }
        let block_metadata = vec_i8_into_u8(self.data.clone());

        //log::info!("Got to here!");
        use crate::world::chunks::*;
        let metadata = crate::world::chunks::decompress_vec(block_metadata);
        let mut blocks = Vec::new();
        let mut i = 0;
        for block in block_ids {
            let meta = if let Some(ref meta) = metadata {
                meta[i]
            } else {
                0
            };
            blocks.push(BlockState {
                b_type: block,
                b_metadata: meta,
                b_light: 0,
                b_skylight: 15,
            });
            i += 1;
        }
        //log::info!("Compression type: {}", comp_type);
        //log::info!("Pos: {} {}", x_pos, z_pos);
        let mut chunksections = Vec::new();
        for i in 0..8 {
            chunksections.push(ChunkSection::new(i));
        }
        for section in 0..8 {
            for x in 0..16 {
                for z in 0..16 {
                    for y in 0..16 {
                        let y = y + (section * 16);
                        //log::info!("Doing section {}, {} {} {}", section, x, y, z);
                        let section = chunksections.get_mut(section).unwrap();
                        section
                            .get_data()
                            .push(blocks[Region::pos_to_idx(x, y as i32, z)]);
                    }
                }
            }
        }
        let mut chunk = Chunk {
            pos: ChunkCoords {
                x: self.x,
                z: self.z,
                world: world_type
            },
            data: [
                Some(chunksections[0].clone()),
                Some(chunksections[1].clone()),
                Some(chunksections[2].clone()),
                Some(chunksections[3].clone()),
                Some(chunksections[4].clone()),
                Some(chunksections[5].clone()),
                Some(chunksections[6].clone()),
                Some(chunksections[7].clone()),
            ],
            heightmaps: HeightmapStore::new(),
        };
        chunk
            .heightmaps
            .recalculate(Chunk::block_at_fn(&chunk.data));
        //chunk.calculate_full_skylight();
        return Ok(chunk);
    }
}
