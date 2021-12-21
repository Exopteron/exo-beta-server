use std::{any::type_name, marker::PhantomData, time::{Duration, Instant}};

use crate::{game::Game, server::Server};
mod entities;
pub mod world;
pub mod chat;
pub mod tablist;
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

            let result = (system.function)(input);
            if let Err(e) = result {
                log::error!(
                    "System {} returned an error; this is a bug: {:?}",
                    system.name,
                    e
                );
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

pub fn default_systems(g: &mut Game, s: &mut SystemExecutor<Game>) {
    entities::player::init_systems(s);
    entities::default_systems(g, s);
    chat::register(g, s);
    crate::world::view::register(s);
    crate::world::chunk_subscriptions::register(s);
    world::register(g, s);
    s.group::<Server>().add_system(|_game, server| {
        for client in server.clients.iter_mut() {
            client.1.tick();
        }
        Ok(())
    }).add_system(send_keepalives).add_system(time_update);
    crate::entities::register(g, s);
    crate::world::chunk_entities::register(s);
    tablist::register(s);
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
    server.clients.iter().for_each(|(_, cl)| {
        cl.notify_time(game.time);
    });
    game.time += 1;
    Ok(())
}