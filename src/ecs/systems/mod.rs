use std::{
    any::type_name,
    marker::PhantomData,
    panic::{self, AssertUnwindSafe},
    time::{Duration, Instant}, sync::atomic::Ordering,
};

use rand::Rng;

use crate::{game::{Game, Position}, server::Server, configuration::CONFIGURATION, SHUTDOWN, network::ids::NetworkID, events::WeatherChangeEvent};
pub mod chat;
pub mod entities;
pub mod stdin;
pub mod tablist;
pub mod world;
use super::{HasEcs, HasResources};

// Derived from feather-rs. License can be found in FEATHER_LICENSE.md

/// The result type returned by a system function.
///
/// When a system encounters an internal error, it should return
/// an error instead of panicking. The system executor will then
/// log an error message to the console and attempt to gracefully
/// recover.
///
/// Examples of internal errors include:
/// * An entity was missing a component which it was expected to have.
/// (For example, all entities have a `Position` component; if an entity
/// is missing it, then that is valid grounds for a system to return an error.)
/// * IO errors
///
/// That said, these errors should never happen in production.
pub type SysResult<T = ()> = anyhow::Result<T>;

type SystemFn<Input> = Box<dyn FnMut(&mut Input) -> SysResult>;

struct System<Input> {
    function: SystemFn<Input>,
    name: String,
}

impl<Input> System<Input> {
    fn from_fn<F: FnMut(&mut Input) -> SysResult + 'static>(f: F) -> Self {
        Self {
            function: Box::new(f),
            name: type_name::<F>().to_owned(),
        }
    }
}

/// An executor for systems.
///
/// This executor contains a sequence of systems, each
/// of which is simply a function taking an `&mut Input`.
///
/// Systems may belong to _groups_, where each system
/// gets an additional parameter representing the group state.
/// For example, the `Server` group has state contained in the `Server`
/// struct, so all its systems get `Server` as an extra parameter.
///
/// Systems run sequentially in the order they are added to the executor.
pub struct SystemExecutor<Input> {
    systems: Vec<System<Input>>,

    is_first_run: bool,
}

impl<Input> Default for SystemExecutor<Input> {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
            is_first_run: true,
        }
    }
}

impl<Input> SystemExecutor<Input> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a system to the executor.
    pub fn add_system(
        &mut self,
        system: impl FnMut(&mut Input) -> SysResult + 'static,
    ) -> &mut Self {
        let system = System::from_fn(system);
        self.systems.push(system);
        self
    }

    pub fn add_system_with_name(
        &mut self,
        system: impl FnMut(&mut Input) -> SysResult + 'static,
        name: &str,
    ) {
        let mut system = System::from_fn(system);
        system.name = name.to_owned();
        self.systems.push(system);
    }

    /// Begins a group with the provided group state type.
    ///
    /// The group state must be added to the `resources`.
    pub fn group<State>(&mut self) -> GroupBuilder<Input, State>
    where
        Input: HasResources,
    {
        GroupBuilder {
            systems: self,
            _marker: PhantomData,
        }
    }

    /// Runs all systems in order.
    ///
    /// Errors are logged using the `log` crate.
    pub fn run(&mut self, input: &mut Input)
    where
        Input: HasEcs,
    {
        for (i, system) in self.systems.iter_mut().enumerate() {
            input.ecs_mut().set_current_system_index(i);

            // For the first cycle, we don't want to clear
            // events because some code may have triggered
            // events _before_ the first system run. Without
            // this check, these events would be cleared before
            // any system could observe them.
            if !self.is_first_run {
                input.ecs_mut().remove_old_events();
            }
            let result = panic::catch_unwind(AssertUnwindSafe(|| (system.function)(input)));
            match result {
                Ok(result) => {
                    if let Err(e) = result {
                        log::error!(
                            "System {} returned an error; this is a bug: {:?}",
                            system.name,
                            e
                        );
                    }
                }
                Err(e) => {
                    if CONFIGURATION.shutdown_after_system_panic {
                        log::error!(
                            "System {} panicked! shutting down: {:?}",
                            system.name,
                            e
                        );
                        SHUTDOWN.store(true, Ordering::Relaxed);
                        return;
                    } else {
                        log::error!(
                            "System {} panicked! attempting to recover: {:?}",
                            system.name,
                            e
                        );
                    }
                }
            }
        }

        self.is_first_run = false;
    }

    /// Gets an iterator over system names.
    pub fn system_names(&self) -> impl Iterator<Item = &'_ str> + '_ {
        self.systems.iter().map(|system| system.name.as_str())
    }
}

/// Builder for a group. Created with [`SystemExecutor::group`].
pub struct GroupBuilder<'a, Input, State> {
    systems: &'a mut SystemExecutor<Input>,
    _marker: PhantomData<State>,
}

impl<'a, Input, State> GroupBuilder<'a, Input, State>
where
    Input: HasResources + 'static,
    State: 'static,
{
    /// Adds a system to the group.
    pub fn add_system<F: FnMut(&mut Input, &mut State) -> SysResult + 'static>(
        &mut self,
        system: F,
    ) -> &mut Self {
        let function = Self::make_function(system);
        self.systems
            .add_system_with_name(function, type_name::<F>());
        self
    }

    fn make_function(
        mut system: impl FnMut(&mut Input, &mut State) -> SysResult + 'static,
    ) -> impl FnMut(&mut Input) -> SysResult + 'static {
        move |input: &mut Input| {
            let resources = input.resources();
            let mut state = resources
                .get_mut::<State>()
                .expect("missing state resource for group");
            system(input, &mut *state)
        }
    }
}

pub fn default_systems(g: &mut Game, s: &mut SystemExecutor<Game>) -> anyhow::Result<()> {
    entities::player::init_systems(s);
    entities::default_systems(g, s)?;
    chat::register(g, s);
    crate::world::view::register(s);
    crate::world::chunk_subscriptions::register(s);
    world::register(g, s);
    s.group::<Server>()
        .add_system(send_keepalives)
        .add_system(time_update);
    crate::entities::register(g, s);
    crate::world::chunk_entities::register(s);
    tablist::register(s);
    stdin::register(g, s);
    Ok(())
}
/// Sends out keepalive packets at an interval.
fn send_keepalives(_game: &mut Game, server: &mut Server) -> SysResult {
    let interval = Duration::from_secs(5);
    if server.last_keepalive_time + interval < Instant::now() {
        server.broadcast_keepalive();
    }
    Ok(())
}

fn time_update(game: &mut Game, server: &mut Server) -> SysResult {
    let mut list = Vec::new();
    for (_, world) in game.worlds.iter_mut() {
        let mut do_event = false;
        {
            let mut level_dat = world.level_dat.lock();
            level_dat.time += 1;
    
            level_dat.rain_time -= 1;
            level_dat.thunder_time -= 1;
    
    
            if level_dat.rain_time == 0 {
                level_dat.raining ^= true;
                level_dat.rain_time = game.rng.gen_range(1..10000);
                do_event = true;
            }
            if level_dat.thunder_time == 0 {
                level_dat.thundering ^= true;
                level_dat.thunder_time = game.rng.gen_range(1..10000);
                do_event = true;
            }
            if do_event {
                game.ecs.insert_event(WeatherChangeEvent { is_raining: level_dat.raining, is_thundering: level_dat.thundering, world: world.id })
            }
        }
        
        
    }
    for (_, (id, position)) in game.ecs.query::<(&NetworkID, &Position)>().iter() {
        list.push((*id, *position));
    }
    for (id, pos) in list {
        if let Some(client) = server.clients.get(&id) {
            let time = game.worlds.get(&pos.world).unwrap().level_dat.lock().time;
            client.notify_time(time);
        }
    }
    Ok(())
}
