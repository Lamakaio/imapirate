use std::sync::Arc;

use bevy::prelude::*;
use rapier2d::{
    crossbeam::channel::{Receiver, Sender},
    dynamics::{
        IntegrationParameters, JointSet, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
    },
    geometry::{
        BroadPhase, Collider, ColliderBuilder, ColliderHandle, ColliderSet, ContactEvent,
        InteractionGroups, IntersectionEvent, NarrowPhase, SharedShape, TriMesh,
    },
    math::{Isometry, Vector},
    na::Vector2,
    pipeline::{EventHandler, PhysicsPipeline},
};

use crate::loading::GameState;

use super::{
    player::{CollisionType, PlayerPositionUpdate},
    worldgen::Island,
};

pub struct SeaCollisionPlugin;
impl Plugin for SeaCollisionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let (collision_wrapper, handles) = CollisionWrapper::new();
        app.add_resource(collision_wrapper)
            .add_resource(handles)
            .on_state_update(GameState::STAGE, GameState::Sea, collision_system.system())
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                spawn_island_colliders.system(),
            )
            .add_event::<IslandSpawnEvent>();
    }
}

pub enum IslandSpawnEvent {
    Spawn(u32),
    Despawn(u32),
}

struct InteractionChannelEventHandler {
    channel: Sender<IntersectionEvent>,
}
impl EventHandler for InteractionChannelEventHandler {
    fn handle_intersection_event(&self, event: rapier2d::geometry::IntersectionEvent) {
        self.channel.send(event).unwrap();
    }

    fn handle_contact_event(&self, _event: ContactEvent) {}
}
pub struct CollisionEvent {
    collider1: ColliderHandle,
    user_data_1: UserData,
    collider2: ColliderHandle,
    user_data_2: UserData,
    intersecting: bool,
}
pub struct CollisionWrapper {
    pub pos_send: Sender<(f32, f32)>,
    body_send: Sender<(RigidBody, Vec<Collider>)>,
    event_receiver: Receiver<CollisionEvent>,
}
impl CollisionWrapper {
    fn new() -> (Self, CollisionHandles) {
        let (event_send, event_recv) = rapier2d::crossbeam::channel::unbounded();
        let (body_send, body_recv): (_, Receiver<(RigidBody, Vec<Collider>)>) =
            rapier2d::crossbeam::channel::unbounded();
        let (pos_send, pos_recv) = rapier2d::crossbeam::channel::unbounded();
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let boat_rigidbody = RigidBodyBuilder::new_dynamic()
            .lock_translations()
            .lock_rotations()
            .build();
        let boat_rb_handle = bodies.insert(boat_rigidbody);
        let boat_collider = ColliderBuilder::ball(5.0)
            //.sensor(true)
            .collision_groups(InteractionGroups::new(
                0b0000_0000_0000_0001,
                0b0000_0000_0000_0001,
            ))
            .build();
        let screen_collider = ColliderBuilder::ball(800.)
            //.sensor(true)
            .collision_groups(InteractionGroups::new(
                0b0000_0000_0000_0010,
                0b0000_0000_0000_0010,
            ))
            .build();
        let boat_coll_handle = colliders.insert(boat_collider, boat_rb_handle, &mut bodies);
        let screen_coll_handle = colliders.insert(screen_collider, boat_rb_handle, &mut bodies);
        std::thread::spawn(move || {
            let mut pipeline = PhysicsPipeline::new();
            let gravity = Vector2::new(0.0, 0.0);
            let integration_parameters = IntegrationParameters {
                max_velocity_iterations: 1,
                ..Default::default()
            };
            let mut broad_phase = BroadPhase::new();
            let mut narrow_phase = NarrowPhase::new();

            let mut joints = JointSet::new();
            let (send, recv) = rapier2d::crossbeam::channel::unbounded();
            let event_handler = InteractionChannelEventHandler { channel: send };
            loop {
                for (body, mut collider_vec) in body_recv.try_iter() {
                    let rb_handle = bodies.insert(body);
                    for c in collider_vec.drain(..) {
                        colliders.insert(c, rb_handle, &mut bodies);
                    }
                }
                let rb = bodies.get_mut(boat_rb_handle).unwrap();
                for (x, y) in pos_recv.try_iter() {
                    rb.set_position(Isometry::new(Vector::new(x, y), 0.), true)
                }

                pipeline.step(
                    &gravity,
                    &integration_parameters,
                    &mut broad_phase,
                    &mut narrow_phase,
                    &mut bodies,
                    &mut colliders,
                    &mut joints,
                    None,
                    None,
                    &event_handler,
                );
                for IntersectionEvent {
                    collider1,
                    collider2,
                    intersecting,
                } in recv.try_iter()
                {
                    event_send
                        .send(CollisionEvent {
                            collider1,
                            user_data_1: colliders.get(collider1).unwrap().user_data.into(),
                            collider2,
                            user_data_2: colliders.get(collider2).unwrap().user_data.into(),
                            intersecting,
                        })
                        .unwrap();
                }
            }
        });
        (
            CollisionWrapper {
                event_receiver: event_recv,
                body_send,
                pos_send,
            },
            CollisionHandles {
                screen_collider: screen_coll_handle,
                boat_collider: boat_coll_handle,
                boat_rb: boat_rb_handle,
            },
        )
    }
}

pub struct CollisionHandles {
    pub screen_collider: ColliderHandle,
    pub boat_collider: ColliderHandle,
    pub boat_rb: RigidBodyHandle,
}
impl Default for CollisionHandles {
    fn default() -> Self {
        CollisionHandles {
            screen_collider: ColliderHandle::invalid(),
            boat_collider: ColliderHandle::invalid(),
            boat_rb: RigidBodyHandle::invalid(),
        }
    }
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
    collision_wrapper: Res<CollisionWrapper>,
    mut spawn_events: ResMut<Events<IslandSpawnEvent>>,
    mut player_pos_update: ResMut<PlayerPositionUpdate>,
    colliders: Res<CollisionHandles>,
) {
    for event in collision_wrapper.event_receiver.try_iter() {
        let CollisionEvent {
            intersecting,
            collider1,
            user_data_1,
            collider2,
            user_data_2,
        } = event;
        if collider1 == colliders.screen_collider || collider2 == colliders.screen_collider {
            let user_data = if collider1 == colliders.screen_collider {
                user_data_2
            } else {
                user_data_1
            };
            spawn_events.send(if intersecting {
                IslandSpawnEvent::Spawn(user_data.island_id)
            } else {
                IslandSpawnEvent::Despawn(user_data.island_id)
            })
        } else if collider1 == colliders.boat_collider || collider2 == colliders.boat_collider {
            let user_data = if collider1 == colliders.screen_collider {
                user_data_2
            } else {
                user_data_1
            };
            let collision_type = if user_data.is_rigid {
                CollisionType::Rigid(0.3)
            } else {
                CollisionType::Friction(0.3)
            };
            if intersecting {
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
    mut islands_to_spawn: ResMut<Vec<(Island, Option<TriMesh>, TriMesh)>>,
    collision_wrapper: Res<CollisionWrapper>,
    mut islands: ResMut<Vec<Island>>,
) {
    for (island, rigid, friction) in islands_to_spawn.drain(..) {
        let island_id = islands.len() as u32;
        //spawn_events.send(IslandSpawnEvent::Spawn(island_id));
        //println!("{}", island_id);
        let island_rigidbody = RigidBodyBuilder::new_dynamic()
            .lock_translations()
            .lock_rotations()
            .translation(island.left as f32 * 16., island.down as f32 * 16.)
            .build();
        let mut colliders = Vec::new();
        if let Some(rigid) = rigid {
            let rigid_collider = ColliderBuilder::new(SharedShape(Arc::new(rigid)))
                .sensor(true)
                .collision_groups(InteractionGroups::all())
                .user_data(UserData::new(island_id, true).into())
                .build();
            colliders.push(rigid_collider);
        }
        //println!("{:#?}", friction.indices());
        let friction_collider = ColliderBuilder::new(SharedShape(Arc::new(friction)))
            .sensor(true)
            .collision_groups(InteractionGroups::all())
            .user_data(UserData::new(island_id, false).into())
            .build();

        colliders.push(friction_collider);
        collision_wrapper
            .body_send
            .send((island_rigidbody, colliders))
            .unwrap();
        islands.push(island);
    }
}
