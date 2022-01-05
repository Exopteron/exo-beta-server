use std::collections::HashSet;

use itertools::Either;

use crate::{game::{Game, Position, ChunkCoords}, ecs::{systems::{SysResult, SystemExecutor}, entities::player::{Username, CurrentWorldInfo}}, events::{ViewUpdateEvent, PlayerJoinEvent}};

// feather license in FEATHER_LICENSE.md

/// Registers systems to update the `View` of a player.
pub fn register(systems: &mut SystemExecutor<Game>) {
    systems
        .add_system(update_player_views)
        .add_system(update_view_on_join);
}

/// Updates players' views when they change chunks.
fn update_player_views(game: &mut Game) -> SysResult {
    let mut events = Vec::new();
    for (player, (view, &position, name)) in
        game.ecs.query::<(&mut View, &Position, &Username)>().iter()
    {
        if position.to_chunk_coords() != view.center() {
            let old_view = *view;
            let new_view = View::new(position.to_chunk_coords(), old_view.view_distance);

            let event = ViewUpdateEvent::new(old_view, new_view);
            events.push((player, event));

            *view = new_view;
            log::trace!("View of {} has been updated", name.0);
        }
    }

    for (player, event) in events {
        game.ecs.insert_entity_event(player, event)?;
    }
    Ok(())
}

/// Triggers a ViewUpdateEvent when a player joins the game.
fn update_view_on_join(game: &mut Game) -> SysResult {
    let mut events = Vec::new();
    for (player, (&view, name, _, world)) in game.ecs.query::<(&View, &Username, &PlayerJoinEvent, &CurrentWorldInfo)>().iter() {
        let event = ViewUpdateEvent::new(View::empty(world.world_id), view);
        events.push((player, event));
        log::trace!("View of {} has been updated (player joined)", name.0);
    }
    for (player, event) in events {
        game.ecs.insert_entity_event(player, event)?;
    }
    Ok(())
}

/// The view of a player, representing the set of chunks
/// within their view distance.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct View {
    center: ChunkCoords,
    view_distance: u32,
}

impl View {
    /// Creates a `View` from a center chunk (the position of the player)
    /// and the view distance.
    pub fn new(center: ChunkCoords, view_distance: u32) -> Self {
        Self {
            center,
            view_distance,
        }
    }

    /// Gets the empty view, i.e., the view containing no chunks.
    pub fn empty(world: i32) -> Self {
        Self::new(ChunkCoords::new(0, 0, world), 0)
    }

    /// Determines whether this is the empty view.
    pub fn is_empty(&self) -> bool {
        self.view_distance == 0
    }

    pub fn center(&self) -> ChunkCoords {
        self.center
    }

    pub fn view_distance(&self) -> u32 {
        self.view_distance
    }

    pub fn set_center(&mut self, center: ChunkCoords) {
        self.center = center;
    }

    pub fn set_view_distance(&mut self, view_distance: u32) {
        self.view_distance = view_distance;
    }

    /// Iterates over chunks visible to the player.
    pub fn iter(self) -> impl Iterator<Item = ChunkCoords> {
        if self.is_empty() {
            Either::Left(std::iter::empty())
        } else {
            Either::Right(Self::iter_2d(
                self.min_x(),
                self.min_z(),
                self.max_x(),
                self.max_z(),
                self.center.world
            ))
        }
    }

    /// Returns the set of chunks that are in `self` but not in `other`.
    pub fn difference(self, other: View) -> impl Iterator<Item = ChunkCoords> {
        // PERF: consider analytical approach instead of sets
        let self_chunks: HashSet<_> = self.iter().collect();
        let other_chunks: HashSet<_> = other.iter().collect();
        self_chunks
            .difference(&other_chunks)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// Determines whether the given chunk is visible.
    pub fn contains(&self, pos: ChunkCoords) -> bool {
        if self.center.world != pos.world {
            return false;
        }
        pos.x >= self.min_x()
            && pos.x <= self.max_x()
            && pos.z >= self.min_z()
            && pos.z <= self.max_z()
    }

    fn iter_2d(
        min_x: i32,
        min_z: i32,
        max_x: i32,
        max_z: i32,
        world: i32,
    ) -> impl Iterator<Item = ChunkCoords> {
        (min_x..=max_x)
            .flat_map(move |x| (min_z..=max_z).map(move |z| (x, z, world)))
            .map(|(x, z, world)| ChunkCoords { x, z, world })
    }

    /// Returns the minimum X chunk coordinate.
    pub fn min_x(&self) -> i32 {
        self.center.x - self.view_distance as i32
    }

    /// Returns the minimum Z coordinate.
    pub fn min_z(&self) -> i32 {
        self.center.z - self.view_distance as i32
    }

    /// Returns the maximum X coordinate.
    pub fn max_x(&self) -> i32 {
        self.center.x + self.view_distance as i32
    }

    /// Returns the maximum Z coordinate.
    pub fn max_z(&self) -> i32 {
        self.center.z + self.view_distance as i32
    }
}
