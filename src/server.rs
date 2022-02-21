use crate::block_entity;
use crate::block_entity::BlockEntity;
use crate::block_entity::BlockEntityLoader;
use crate::block_entity::SignData;
use crate::configuration::CONFIGURATION;
use crate::ecs::entities::living::Health;
use crate::ecs::entities::living::Hunger;
use crate::ecs::entities::player::ChatMessage;
use crate::ecs::entities::player::Gamemode;
use crate::ecs::entities::player::HotbarSlot;
use crate::ecs::entities::player::Username;
use crate::ecs::entities::player::SLOT_HOTBAR_OFFSET;
use crate::ecs::systems;
use crate::ecs::systems::world::view::WaitingChunks;
use crate::ecs::systems::SysResult;
use crate::ecs::systems::SystemExecutor;
use crate::ecs::EntityRef;
use crate::entities;
use crate::entities::metadata::EntityMetadata;
use crate::entities::PreviousPosition;
use crate::game::BlockPosition;
use crate::game::ChunkCoords;
use crate::game::Game;
use crate::game::Position;
use crate::item::inventory::reference::BackingWindow;
use crate::item::inventory_slot::InventorySlot;
use crate::item::stack::ItemStackType;
use crate::item::window::Window;
use crate::network::ids::NetworkID;
use crate::network::metadata::Metadata;
use crate::network::Listener;
use crate::player_count::PlayerCount;
use crate::protocol::io::Slot;
use crate::protocol::io::String16;
use crate::protocol::packets::EntityEffectKind;
use crate::protocol::packets::EnumMobType;
use crate::protocol::packets::ObjectVehicleKind;
use crate::protocol::packets::WindowKind;
use crate::protocol::packets::server::AddObjectVehicle;
use crate::protocol::packets::server::BlockAction;
use crate::protocol::packets::server::BlockChange;
use crate::protocol::packets::server::ChunkData;
use crate::protocol::packets::server::ChunkDataKind;
use crate::protocol::packets::server::CloseWindow;
use crate::protocol::packets::server::CollectItem;
use crate::protocol::packets::server::DestroyEntity;
use crate::protocol::packets::server::EntityEffect;
use crate::protocol::packets::server::EntityEquipment;
use crate::protocol::packets::server::EntityLook;
use crate::protocol::packets::server::EntityLookAndRelativeMove;
use crate::protocol::packets::server::EntityRelativeMove;
use crate::protocol::packets::server::EntityStatus;
use crate::protocol::packets::server::EntityTeleport;
use crate::protocol::packets::server::EntityVelocity;
use crate::protocol::packets::server::KeepAlive;
use crate::protocol::packets::server::Kick;
use crate::protocol::packets::server::MobSpawn;
use crate::protocol::packets::server::NamedEntitySpawn;
use crate::protocol::packets::server::NewState;
use crate::protocol::packets::server::OpenWindow;
use crate::protocol::packets::server::PickupSpawn;
use crate::protocol::packets::server::PlayerListItem;
use crate::protocol::packets::server::PlayerPositionAndLook;
use crate::protocol::packets::server::PreChunk;
use crate::protocol::packets::server::RemoveEntityEffect;
use crate::protocol::packets::server::Respawn;
use crate::protocol::packets::server::SendEntityAnimation;
use crate::protocol::packets::server::SendEntityMetadata;
use crate::protocol::packets::server::SetSlot;
use crate::protocol::packets::server::SoundEffect;
use crate::protocol::packets::server::TimeUpdate;
use crate::protocol::packets::server::Transaction;
use crate::protocol::packets::server::UpdateHealth;
use crate::protocol::packets::server::UpdateSign;
use crate::protocol::packets::server::WindowItems;
use crate::protocol::packets::EntityAnimationType;
use crate::protocol::packets::EntityStatusKind;
use crate::protocol::packets::SoundEffectKind;
use crate::protocol::ClientPlayPacket;
use crate::protocol::ServerPlayPacket;
use crate::world::chunk_lock::ChunkHandle;
use crate::world::chunk_subscriptions::ChunkSubscriptions;
use crate::world::chunks::BlockState;
use ahash::AHashMap;
use flume::{Receiver, Sender};
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::net::*;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
pub struct NewPlayer {
    pub username: String,
    pub recv_packets_recv: Receiver<ClientPlayPacket>,
    pub packet_send_sender: Sender<ServerPlayPacket>,
    pub id: NetworkID,
    pub addr: SocketAddr,
}
impl NewPlayer {
    pub async fn write(&mut self, packet: ServerPlayPacket) -> anyhow::Result<()> {
        self.packet_send_sender.send_async(packet).await?;
        Ok(())
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPlayPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
}
pub struct Client {
    pub recv_packets_recv: Receiver<ClientPlayPacket>,
    pub packet_send_sender: Sender<ServerPlayPacket>,
    username: String,
    pub id: NetworkID,
    sent_entities: RefCell<HashSet<NetworkID>>,
    pub addr: SocketAddr,
    pub disconnected: Cell<bool>,
    pub chunk_send_queue: RefCell<VecDeque<ChunkData>>,
    known_chunks: RefCell<HashSet<ChunkCoords>>,
    knows_position: Cell<bool>,
    /// The previous own position sent by the client.
    /// Used to detect when we need to teleport the client.
    client_known_position: Cell<Option<Position>>,
}
impl Client {
    pub fn spawn_mob(&self, id: NetworkID, mob_type: EnumMobType, position: Position, metadata: Metadata) {
        self.send_packet(MobSpawn {
            eid: id.0,
            mobtype: mob_type,
            x: position.x.into(),
            y: position.y.into(),
            z: position.z.into(),
            yaw: position.yaw.into(),
            pitch: position.pitch.into(),
            meta: metadata,
        });
    }
    pub fn close_window(&self, window_id: i8) {
        self.send_packet(CloseWindow {
            wid: window_id,
        });
    }
    pub fn open_window(&self, window_id: i8, inventory_type: WindowKind, title: String, num_slots: i8) {
        self.send_packet(OpenWindow {
            window_id,
            inventory_type,
            window_title: title.into(),
            num_slots,
        });
    }
    pub fn spawn_object_vehicle(&self, id: NetworkID, object_type: ObjectVehicleKind, position: Position) {
        self.register_entity(id);
        self.send_packet(AddObjectVehicle {
            eid: id.0,
            object_type,
            x: position.x.into(),
            y: position.y.into(),
            z: position.z.into(),
            fbeid: 0,
        });
    }
    pub fn send_entity_velocity(&self, id: NetworkID, x: f64, y: f64, z: f64) {
        self.send_packet(EntityVelocity {
            eid: id.0,
            velocity_x: (x * 8000.) as i16,
            velocity_y: (y * 8000.) as i16,
            velocity_z: (z * 8000.) as i16,
        });
    } 
    pub fn send_collect_item(&self, collected: NetworkID, collector: NetworkID) {
        self.send_packet(CollectItem {
            collected_eid: collected.0,
            collector_eid: collector.0,
        });
    }
    pub fn spawn_dropped_item(&self, eid: NetworkID, pos: Position, item: Slot) {
        if let InventorySlot::Filled(_) = &item {
            self.send_packet(PickupSpawn {
                eid: eid.0,
                item,
                x: pos.x.into(),
                y: pos.y.into(),
                z: pos.z.into(),
                rotation: 0,
                pitch: 0,
                roll: 0,
            });
            self.register_entity(eid);
        }
    }
    pub fn remove_entity_effect(&self, id: NetworkID, kind: EntityEffectKind) {
        self.send_packet(RemoveEntityEffect {
            eid: id.0,
            effect_id: kind
        });
    }
    pub fn send_entity_effect(&self, id: NetworkID, kind: EntityEffectKind, amplifier: i8, duration: i16) {
        self.send_packet(EntityEffect {
            eid: id.0,
            effect_id: kind,
            amplifier,
            duration
        });
    }
    pub fn send_block_action(&self, position: BlockPosition, byte1: i8, byte2: i8) {
        self.send_packet(BlockAction {
            x: position.x,
            y: position.y as i16,
            z: position.z,
            byte1,
            byte2,
        });
    }
    pub fn update_sign(&self, position: BlockPosition, data: SignData) {
        log::info!("Updating sign at {:?} with {:?}", position, data);
        self.send_packet(UpdateSign {
            x: position.x,
            y: position.y as i16,
            z: position.z,
            text1: data.0[0].clone().into(),
            text2: data.0[1].clone().into(),
            text3: data.0[2].clone().into(),
            text4: data.0[3].clone().into(),
        });
    }
    pub fn notify_time(&self, time: i64) {
        self.send_packet(TimeUpdate { time });
    }
    pub fn notify_respawn(&self, us: &EntityRef, world_seed: u64) -> SysResult {
        let current_world = us.get::<Position>()?.world as i8;
        let gamemode = us.get::<Gamemode>()?.id();
        self.send_packet(Respawn {
            world: current_world,
            difficulty: 0,
            gamemode,
            world_height: 128,
            map_seed: world_seed as i64,
        });
        Ok(())
    }
    pub fn set_health(&self, health: &Health, hunger: &Hunger) {
        self.send_packet(UpdateHealth {
            health: health.0,
            food: hunger.0,
            saturation: hunger.1,
        });
    }
    pub fn set_gamemode(&self, gamemode: Gamemode) {
        self.send_packet(NewState {
            reason: 3,
            gamemode: gamemode as i8,
        });
    }
    pub fn send_effect(&self, position: BlockPosition, effect: SoundEffectKind, data: i32) {
        self.send_packet(SoundEffect {
            effect,
            pos: position,
            data,
        });
    }
    pub fn send_block_change(&self, position: BlockPosition, new_block: BlockState) {
        self.send_packet(BlockChange {
            pos: position,
            state: new_block,
        });
    }
    pub fn send_entity_status(&self, id: NetworkID, status: EntityStatusKind) {
        self.send_packet(EntityStatus { eid: id.0, status });
    }
    pub fn send_entity_equipment(&self, entity: &EntityRef) -> SysResult {
        let id = *entity.get::<NetworkID>()?;
        if self.id == id {
            return Ok(());
        }
        let hotbar_slot = entity.get::<HotbarSlot>()?.get();
        let inventory = entity.get::<Window>()?.inner().clone();
        let slot = inventory.item(SLOT_HOTBAR_OFFSET + hotbar_slot)?;
        self.entity_equipment(id, 0, &slot);
        for i in 1..5 {
            let slot = inventory.item(i + 4)?;
            self.entity_equipment(id, i as i16, &slot);
        }
        Ok(())
    }
    fn entity_equipment(&self, id: NetworkID, slot: i16, item: &InventorySlot) {
        match item.item_kind() {
            Some(i) => match i {
                ItemStackType::Item(i) => {
                    self.send_packet(EntityEquipment {
                        eid: id.0,
                        slot,
                        item_id: i.id(),
                        damage: item.damage(),
                    });
                }
                ItemStackType::Block(i) => {
                    self.send_packet(EntityEquipment {
                        eid: id.0,
                        slot,
                        item_id: i.id() as i16,
                        damage: item.damage(),
                    });
                }
            },
            None => {
                self.send_packet(EntityEquipment {
                    eid: id.0,
                    slot,
                    item_id: -1,
                    damage: 0,
                });
            }
        };
    }
    pub fn set_cursor_slot(&self, item: &InventorySlot) {
        log::trace!("Setting cursor slot of {} to {:?}", self.username, item);
        self.set_slot(-1, -1, item);
    }
    pub fn set_slot(&self, window_id: i8, slot: i16, item: &InventorySlot) {
        log::trace!("Setting slot {} of {} to {:?}", slot, self.username, item);
        self.send_packet(SetSlot {
            window_id,
            slot,
            item: item.clone(),
        });
    }
    pub fn confirm_window_action(&self, window_id: i8, action_number: i16, is_accepted: bool) {
        self.send_packet(Transaction {
            window_id,
            action_number,
            accepted: is_accepted,
        });
    }
    pub fn send_window_items(&self, window: &Window) {
        log::trace!("Updating window for {}", self.username);
        let mut id = 1;
        if let BackingWindow::Player { .. } = window.inner() {
            id = 0;
        }
        let packet = WindowItems {
            window_id: id,
            items: window.inner().to_vec(),
        };
        self.send_packet(packet);
    }
    pub fn add_tablist_player(&self, name: String, latency: i16) {
        log::trace!("Sending PlayerListItem({}) to {}", name, self.username);
        self.send_packet(PlayerListItem {
            name: String16(name),
            online: true,
            ping: latency,
        });
    }

    pub fn remove_tablist_player(&self, name: String) {
        log::trace!("Sending RemovePlayer({}) to {}", name, self.username);
        self.send_packet(PlayerListItem {
            name: String16(name),
            online: false,
            ping: 0,
        });
    }
    pub fn send_entity_animation(&self, network_id: NetworkID, animation: EntityAnimationType) {
        if self.id == network_id {
            return;
        }
        self.send_packet(SendEntityAnimation {
            eid: network_id.0,
            animation,
        })
    }
    pub fn send_entity_metadata(&self, s2s: bool, network_id: NetworkID, metadata: Metadata) {
        if (self.id == network_id) && !s2s {
            return;
        }
        self.send_packet(SendEntityMetadata {
            eid: network_id.0,
            metadata,
        });
    }
    pub fn send_exact_entity_position(&self, network_id: NetworkID, position: Position) {
        self.send_packet(EntityTeleport {
            eid: network_id.0,
            x: position.x.into(),
            y: position.y.into(),
            z: position.z.into(),
            yaw: position.yaw.into(),
            pitch: position.pitch.into(),
        });
    }
    pub fn update_entity_position(
        &self,
        network_id: NetworkID,
        position: Position,
        prev_position: PreviousPosition,
    ) {
        if !position.update {
            return;
        }
        if self.id == network_id {
            // This entity is the client. Only update
            // the position if it has changed from the client's
            // known position.
            if Some(position) != self.client_known_position.get() {
                self.update_own_position(position);
            }
            return;
        }

        let no_change_yaw = (position.yaw - prev_position.0.yaw).abs() < 0.001;
        let no_change_pitch = (position.pitch - prev_position.0.pitch).abs() < 0.001;

        // If the entity jumps or falls we should send a teleport packet instead to keep relative movement in sync.
        //if position.on_ground != prev_position.0.on_ground && true {
        //log::info!("Sending teleport packet to {}", self.username());
        self.send_packet(EntityTeleport {
            eid: network_id.0,
            x: position.x.into(),
            y: position.y.into(),
            z: position.z.into(),
            yaw: position.yaw.into(),
            pitch: position.pitch.into(),
        });
        // Needed for head orientation
        self.send_packet(EntityLook {
            eid: network_id.0,
            yaw: position.yaw.into(),
            pitch: position.pitch.into(),
        });
        return;
        //}

        if no_change_yaw && no_change_pitch {
            self.send_packet(EntityRelativeMove {
                eid: network_id.0,
                delta_x: (position.x * 32.0 - prev_position.0.x * 32.0) as i8,
                delta_y: (position.y * 32.0 - prev_position.0.y * 32.0) as i8,
                delta_z: (position.z * 32.0 - prev_position.0.z * 32.0) as i8,
            });
        } else {
            self.send_packet(EntityLookAndRelativeMove {
                eid: network_id.0,
                delta_x: (position.x * 32.0 - prev_position.0.x * 32.0) as i8,
                delta_y: (position.y * 32.0 - prev_position.0.y * 32.0) as i8,
                delta_z: (position.z * 32.0 - prev_position.0.z * 32.0) as i8,
                yaw: position.yaw.into(),
                pitch: position.pitch.into(),
            });
        }
    }

    pub fn unload_entity(&self, id: NetworkID) {
        log::trace!("Unloading {:?} on {}", id, self.username);
        self.sent_entities.borrow_mut().remove(&id);
        self.send_packet(DestroyEntity { eid: id.0 });
    }
    pub fn register_entity(&self, network_id: NetworkID) {
        self.sent_entities.borrow_mut().insert(network_id);
    }
    pub fn send_player(&self, network_id: NetworkID, username: &Username, pos: Position) {
        if username.0 == self.username {
            return;
        }
        log::info!("Sending {:?} to {}", username.0, self.username);
        if self.sent_entities.borrow().contains(&network_id) {
            return;
        }
        self.send_packet(NamedEntitySpawn {
            eid: network_id.0,
            x: pos.x.into(),
            y: pos.y.into(),
            z: pos.z.into(),
            rotation: pos.yaw as i8,
            pitch: pos.pitch as i8,
            player_name: username.0.clone().into(),
            current_item: 0,
        });
        self.send_entity_status(network_id, EntityStatusKind::None);
        self.register_entity(network_id);
    }
    pub fn set_disconnected(&self, val: bool) {
        self.disconnected.set(val);
    }
    pub fn set_client_known_position(&self, pos: Position) {
        self.client_known_position.set(Some(pos));
    }
    pub fn client_known_position(&self) -> Option<Position> {
        self.client_known_position.get()
    }
    pub fn send_keepalive(&self) {
        log::trace!("Sending keepalive to {}", self.username);
        self.send_packet(KeepAlive { id: 0 });
    }
    pub fn update_own_position(&self, new_position: Position) {
        log::trace!(
            "Updating position of {} to {:?}",
            self.username,
            new_position
        );
        self.send_packet(PlayerPositionAndLook {
            x: new_position.x,
            y: new_position.y + 1.620000004768372,
            z: new_position.z,
            yaw: new_position.yaw,
            pitch: new_position.pitch,
            stance: 71.62,
            on_ground: new_position.on_ground,
        });
        self.knows_position.set(true);
        self.client_known_position.set(Some(new_position));
    }
    pub fn knows_own_position(&self) -> bool {
        self.knows_position.get()
    }
    pub fn known_chunks(&self) -> usize {
        self.known_chunks.borrow().len()
    }

    pub fn unload_chunk(&self, pos: ChunkCoords) {
        log::trace!("Unloading chunk at {:?} on {}", pos, self.username);
        self.send_packet(PreChunk {
            chunk_x: pos.x,
            chunk_z: pos.z,
            mode: false,
        });
        self.known_chunks.borrow_mut().remove(&pos);
    }
    pub fn send_chunk(&self, chunk: &ChunkHandle) {
        //log::info!("Sending chunk");
        self.chunk_send_queue.borrow_mut().push_back(ChunkData {
            chunk: Arc::clone(chunk),
            kind: ChunkDataKind::LoadChunk,
        });
        self.known_chunks
            .borrow_mut()
            .insert(chunk.read().position());
    }
    pub fn tick(&self) {
        //let num_to_send = MAX_CHUNKS_PER_TICK.min(self.chunk_send_queue.borrow().len());
        for packet in self.chunk_send_queue.borrow_mut().drain(0..) {
            log::trace!(
                "Sending chunk at {:?} to {}",
                packet.chunk.read().position(),
                self.username
            );
            //let chunk = Arc::clone(&packet.chunk);
            //self.send_packet(UpdateLight { chunk });
            let pos = packet.chunk.clone().read().pos;
            self.send_packet(PreChunk {
                chunk_x: pos.x,
                chunk_z: pos.z,
                mode: true,
            });
            self.send_packet(packet);
        }
    }
    pub fn send_chat_message(&self, message: ChatMessage) {
        let packet = crate::protocol::packets::server::ChatMessage {
            message: String16(message.0.to_string()),
        };
        self.send_packet(packet);
    }
    fn send_packet(&self, packet: impl Into<ServerPlayPacket>) {
        let _ = self.packet_send_sender.try_send(packet.into());
    }

    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn new(player: NewPlayer, id: NetworkID) -> Self {
        Self {
            recv_packets_recv: player.recv_packets_recv,
            packet_send_sender: player.packet_send_sender,
            username: player.username,
            id,
            addr: player.addr,
            disconnected: Cell::from(false),
            chunk_send_queue: RefCell::new(VecDeque::new()),
            known_chunks: RefCell::new(HashSet::new()),
            knows_position: Cell::new(false),
            client_known_position: Cell::new(None),
            sent_entities: RefCell::new(HashSet::new()),
        }
    }
    pub fn disconnect(&self, reason: &str) {
        self.disconnected.set(true);
        self.send_packet(Kick {
            reason: String16(reason.to_owned()),
        });
    }
    pub fn is_disconnected(&self) -> bool {
        self.recv_packets_recv.is_disconnected() || self.disconnected.get()
    }
    pub fn write(&mut self, packet: ServerPlayPacket) -> anyhow::Result<()> {
        self.packet_send_sender.send(packet)?;
        Ok(())
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPlayPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
    pub fn recieved_packets(&self) -> impl Iterator<Item = ClientPlayPacket> + '_ {
        self.recv_packets_recv.try_iter()
    }
}
pub struct Server {
    new_players: Receiver<NewPlayer>,
    pub clients: AHashMap<NetworkID, Client>,
    pub last_keepalive_time: Instant,
    pub chunk_subscriptions: ChunkSubscriptions,
    pub waiting_chunks: WaitingChunks,
    pub player_count: PlayerCount,
}
impl Server {
    pub fn sync_block_entity(
        &self,
        position: Position,
        world: i32,
        block_entity_loader: BlockEntityLoader,
        entity: &EntityRef,
    ) {
        self.broadcast_nearby_with(position, |cl| {
            log::info!("Loading BE to {}", cl.username);
            if let Err(e) = block_entity_loader.load(cl, entity) {
                log::error!("Error syncing block entity: {:?}", e);
            }
        });
    }
    pub fn broadcast_entity_status(
        &self,
        position: Position,
        world: i32,
        id: NetworkID,
        effect: EntityStatusKind,
    ) {
        self.broadcast_nearby_with(position, |cl| {
            cl.send_entity_status(id, effect.clone());
        });
    }
    pub fn broadcast_effect_from_entity(
        &self,
        id: NetworkID,
        effect: SoundEffectKind,
        position: BlockPosition,
        world: i32,
        data: i32,
    ) {
        self.broadcast_nearby_with(position.into(), |c| {
            if c.id != id {
                c.send_effect(position, SoundEffectKind::DoorToggle, 0);
            }
        });
    }
    pub fn broadcast_effect(
        &self,
        effect: SoundEffectKind,
        position: BlockPosition,
        world: i32,
        data: i32,
    ) {
        self.broadcast_nearby_with(position.into(), |c| {
            c.send_effect(position, effect.clone(), data);
        });
    }
    pub fn broadcast_equipment_change(&self, player: &EntityRef, world: i32) -> SysResult {
        let id = player.get::<NetworkID>()?.deref().clone();
        self.broadcast_nearby_with(*player.get::<Position>()?, |cl| {
            if cl.id != id {
                if let Err(e) = cl.send_entity_equipment(&player) {
                    log::error!("Error sending entity equipment: {:?}", e);
                }
            }
        });
        Ok(())
    }
    /// Sends a packet to all clients currently subscribed
    /// to the given position. This function should be
    /// used for entity updates, block updates, etc—
    /// any packets that need to be sent only to nearby players.
    pub fn broadcast_nearby_with(&self, position: Position, mut callback: impl FnMut(&Client)) {
        for &client_id in self
            .chunk_subscriptions
            .subscriptions_for(position.to_chunk_coords())
        {
            if let Some(client) = self.clients.get(&client_id) {
                callback(client);
            }
        }
    }
    pub fn broadcast_keepalive(&mut self) {
        self.broadcast_with(|client| client.send_keepalive());
        self.last_keepalive_time = Instant::now();
    }
    pub async fn bind() -> anyhow::Result<Self> {
        let player_count = PlayerCount::new(CONFIGURATION.max_players);
        let (new_players_send, new_players) = flume::bounded(4);
        Listener::start_listening(new_players_send, player_count.clone()).await?;
        Ok(Self {
            new_players,
            clients: AHashMap::new(),
            last_keepalive_time: Instant::now(),
            chunk_subscriptions: ChunkSubscriptions::default(),
            waiting_chunks: WaitingChunks::default(),
            player_count,
        })
    }
    pub fn register(self, game: &mut Game) {
        game.insert_object(self);
        game.add_entity_spawn_callback(entities::add_entity_components);
    }
    pub fn accept_clients(&mut self) -> Vec<NetworkID> {
        let mut clients = Vec::new();
        for player in self.new_players.clone().try_iter() {
            if let Some(old_client) = self
                .clients
                .iter()
                .find(|x| x.1.username == player.username)
            {
                old_client.1.disconnect("Logged in from another location!");
            }
            let id = self.create_client(player);
            clients.push(id);
        }
        clients
    }
    fn create_client(&mut self, player: NewPlayer) -> NetworkID {
        let id = player.id;
        let client = Client::new(player, id.clone());
        self.clients.insert(id.clone(), client);
        id
    }
    pub fn get_id(&mut self) -> NetworkID {
        NetworkID::new()
    }
    /// Invokes a callback on all clients.
    pub fn broadcast_with(&self, mut callback: impl FnMut(&Client)) {
        for client in self.clients.iter() {
            callback(client.1);
        }
    }
    pub fn broadcast_mut(
        &mut self,
        mut function: impl FnMut(&mut Client) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        for mut client in self.clients.iter_mut() {
            function(&mut client.1)?;
        }
        Ok(())
    }
    /// Removes a client.
    pub fn remove_client(&mut self, id: NetworkID) {
        let client = self.clients.remove(&id);
        if let Some(client) = client {
            log::debug!("Removed client for {}", client.username());
        }
    }
}
