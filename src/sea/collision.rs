use std::sync::Arc;

use bevy::prelude::*;
use rapier2d::{
    crossbeam::channel::{Receiver, Sender},
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
    geometry::{
        BroadPhase, ColliderBuilder, ColliderHandle, ColliderSet, ContactEvent, InteractionGroups,
        NarrowPhase, SharedShape, TriMesh,
    },
    pipeline::{CollisionPipeline, EventHandler},
};

use crate::loading::GameState;

use super::{
    player::{CollisionType, PlayerPositionUpdate},
    worldgen::Island,
};

pub struct SeaCollisionPlugin;
impl Plugin for SeaCollisionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<CollisionWrapper>()
            .init_resource::<CollisionHandles>()
            //.on_state_update(GameState::STAGE, GameState::Sea, collision_system.system())
            // .on_state_update(
            //     GameState::STAGE,
            //     GameState::Sea,
            //     spawn_island_colliders.system(),
            // )
            //.on_state_enter(GameState::STAGE, GameState::Sea, setup.system())
            .add_event::<IslandSpawnEvent>();
    }
}

pub enum IslandSpawnEvent {
    Spawn(u32),
    Despawn(u32),
}

struct ContactChannelEventHandler {
    channel: Sender<ContactEvent>,
}
impl EventHandler for ContactChannelEventHandler {
    fn handle_intersection_event(&self, _event: rapier2d::geometry::IntersectionEvent) {}

    fn handle_contact_event(&self, event: ContactEvent) {
        self.channel.send(event).unwrap();
    }
}
pub struct CollisionWrapper {
    pipeline: CollisionPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    colliders: ColliderSet,
    event_handler: ContactChannelEventHandler,
    event_receiver: Receiver<ContactEvent>,
}
impl Default for CollisionWrapper {
    fn default() -> Self {
        let (contact_send, contact_recv) = rapier2d::crossbeam::channel::unbounded();
        CollisionWrapper {
            pipeline: CollisionPipeline::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            event_handler: ContactChannelEventHandler {
                channel: contact_send,
            },
            event_receiver: contact_recv,
        }
    }
}
impl CollisionWrapper {
    fn step(&mut self) {
        self.pipeline.step(
            0.,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            None,
            None,
            &self.event_handler,
        );
    }
}

pub struct CollisionHandles {
    pub screen_collider: ColliderHandle,
    pub boat_collider: ColliderHandle,
    pub islands_rb: RigidBodyHandle,
    pub boat_rb: RigidBodyHandle,
}
impl Default for CollisionHandles {
    fn default() -> Self {
        CollisionHandles {
            screen_collider: ColliderHandle::invalid(),
            boat_collider: ColliderHandle::invalid(),
            islands_rb: RigidBodyHandle::invalid(),
            boat_rb: RigidBodyHandle::invalid(),
        }
    }
}

fn setup(mut handles: ResMut<CollisionHandles>, mut collision_wrapper: ResMut<CollisionWrapper>) {
    let collision_wrapper = &mut *collision_wrapper;
    let bodies = &mut collision_wrapper.bodies;
    let colliders = &mut collision_wrapper.colliders;
    let islands_rigidbody = RigidBodyBuilder::new_dynamic()
        .lock_translations()
        .lock_rotations()
        .build();
    let boat_rigidbody = RigidBodyBuilder::new_kinematic().build();
    let island_rb_handle = bodies.insert(islands_rigidbody);
    let boat_rb_handle = bodies.insert(boat_rigidbody);
    let boat_collider = ColliderBuilder::ball(5.0)
        .sensor(true)
        .collision_groups(InteractionGroups::new(
            0b0000_0000_0000_0001,
            0b0000_0000_0000_0001,
        ))
        .build();
    let screen_collider = ColliderBuilder::cuboid(5000., 3000.)
        .sensor(true)
        .collision_groups(InteractionGroups::new(
            0b0000_0000_0000_0010,
            0b0000_0000_0000_0010,
        ))
        .build();
    let boat_coll_handle = colliders.insert(boat_collider, boat_rb_handle, bodies);
    let screen_coll_handle = colliders.insert(screen_collider, boat_rb_handle, bodies);
    *handles = CollisionHandles {
        screen_collider: screen_coll_handle,
        boat_collider: boat_coll_handle,
        islands_rb: island_rb_handle,
        boat_rb: boat_rb_handle,
    };
}

pub struct UserData {
    island_id: u32,
    is_rigid: bool,
}
impl UserData {
    pub fn new(island_id: u32, is_rigid: bool) -> Self {
        UserData {
            island_id,
            is_rigid,
        }
    }
}
impl From<u128> for UserData {
    fn from(n: u128) -> Self {
        UserData {
            island_id: (n >> 96) as u32,
            is_rigid: (n & 1) == 1,
        }
    }
}
impl From<UserData> for u128 {
    fn from(u_d: UserData) -> Self {
        let mut n = 0;
        if u_d.is_rigid {
            n += 1
        };
        n += (u_d.island_id as u128) << 96;
        n
    }
}
fn collision_system(
    mut collision_wrapper: ResMut<CollisionWrapper>,
    mut spawn_events: ResMut<Events<IslandSpawnEvent>>,
    mut player_pos_update: ResMut<PlayerPositionUpdate>,
    colliders: Res<CollisionHandles>,
) {
    collision_wrapper.step();
    for contact_event in collision_wrapper.event_receiver.try_iter() {
        let (started, collider_a, collider_b) = match contact_event {
            ContactEvent::Started(c_a, c_b) => (true, c_a, c_b),
            ContactEvent::Stopped(c_a, c_b) => (false, c_a, c_b),
        };
        if collider_a == colliders.screen_collider || collider_b == colliders.screen_collider {
            let other = if collider_a == colliders.screen_collider {
                collider_b
            } else {
                collider_a
            };
            let collider = collision_wrapper.colliders.get(other).unwrap();
            spawn_events.send(if started {
                IslandSpawnEvent::Spawn(UserData::from(collider.user_data).island_id)
            } else {
                IslandSpawnEvent::Despawn(UserData::from(collider.user_data).island_id)
            })
        } else if collider_a == colliders.boat_collider || collider_b == colliders.boat_collider {
            let other = if collider_a == colliders.boat_collider {
                collider_b
            } else {
                collider_a
            };
            let collider = collision_wrapper.colliders.get(other).unwrap();
            let collision_type = if UserData::from(collider.user_data).is_rigid {
                CollisionType::Rigid(0.3)
            } else {
                CollisionType::Friction(0.3)
            };
            if started {
                player_pos_update.collision_status = collision_type;
            } else {
                match (&collision_type, &player_pos_update.collision_status) {
                    (CollisionType::Friction(_), CollisionType::Friction(_)) => {
                        player_pos_update.collision_status = CollisionType::None
                    }
                    (CollisionType::Rigid(_), CollisionType::Rigid(_)) => {
                        player_pos_update.collision_status = CollisionType::Friction(0.3)
                    }
                    _ => (),
                }
            }
        }
    }
}

fn spawn_island_colliders(
    mut islands_to_spawn: ResMut<Vec<(Island, TriMesh, TriMesh)>>,
    mut collision_wrapper: ResMut<CollisionWrapper>,
    mut islands: ResMut<Vec<Island>>,
    handles: Res<CollisionHandles>,
) {
    let collision_wrapper = &mut *collision_wrapper;
    let bodies = &mut collision_wrapper.bodies;
    let colliders = &mut collision_wrapper.colliders;
    for (mut island, rigid, friction) in islands_to_spawn.drain(..) {
        let island_id = islands.len() as u32;

        let rigid_collider = ColliderBuilder::new(SharedShape(Arc::new(rigid)))
            .sensor(true)
            .collision_groups(InteractionGroups::all())
            .user_data(UserData::new(island_id, true).into())
            .build();
        let friction_collider = ColliderBuilder::new(SharedShape(Arc::new(friction)))
            .sensor(true)
            .collision_groups(InteractionGroups::all())
            .user_data(UserData::new(island_id, false).into())
            .build();
        island.rigid_collider = colliders.insert(rigid_collider, handles.islands_rb, bodies);
        island.friction_collider = colliders.insert(friction_collider, handles.islands_rb, bodies);
        islands.push(island);
    }
}
