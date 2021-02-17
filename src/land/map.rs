use std::time::Duration;

use crate::{
    background::{BackgroundBundle, BgFlag, TileUv},
    loading::GameState,
    sea::{map::Islands, player::PlayerPositionUpdate},
};
use bevy::{prelude::*, render::camera::Camera};

use super::{loader::LandHandles, LAND_SCALING};
pub struct LandMapPlugin;
impl Plugin for LandMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LoadIslandEvent>()
            .init_resource::<CurrentIsland>()
            .on_state_enter(
                GameState::STAGE,
                GameState::Land,
                load_island_system.system(),
            )
            .on_state_update(
                GameState::STAGE,
                GameState::Land,
                move_anim_bg_system.system(),
            )
            .on_state_exit(
                GameState::STAGE,
                GameState::Land,
                unload_island_system.system(),
            );
    }
}
#[derive(Default)]
pub struct CurrentIsland {
    pub id: u64,
    pub entrance: (i32, i32),
}
pub struct LoadIslandEvent {
    pub island_id: u64,
}
pub struct LandIsland;
fn load_island_system(
    commands: &mut Commands,
    sea_player_pos: Res<PlayerPositionUpdate>,
    islands: Res<Islands>,
    handles: Res<LandHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let island = &islands.0[sea_player_pos.island_id.unwrap() as usize];
    commands
        .spawn(super::super::sea::map::IslandBundle {
            mesh: island.mesh.clone(),
            transform: Transform {
                translation: Vec3::new(0., 0., 3.),
                scale: Vec3::new(LAND_SCALING, LAND_SCALING, 1.),
                ..Default::default()
            },
            material: handles.island_material.clone(),
            ..Default::default()
        })
        .with(LandIsland);
    //initializing the sea animation
    let mut transform = Transform::from_rotation(Quat::from_rotation_x(3.1415926535 / 2.));
    transform.translation.z = 0.;
    commands
        .spawn(BackgroundBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
            transform,
            texture_atlas: handles.sea_sheet.clone(),
            tile_uv: TileUv {
                uv: Vec2::new(0.0, 0.0),
                scale: 2. * LAND_SCALING,
            },
            ..Default::default()
        })
        .with(BgFlag);
}

fn move_anim_bg_system(
    mut bg_query: Query<(&mut TileUv, &mut Transform)>,
    camera_query: Query<&Transform, With<Camera>>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    if timer.is_none() {
        *timer = Some(Timer::new(Duration::from_millis(500), true));
    }
    let timer = timer.as_mut().unwrap();
    for (mut bg, mut bg_transform) in bg_query.iter_mut() {
        for camera_transform in camera_query.iter() {
            bg_transform.translation.x = camera_transform.translation.x;
            bg_transform.translation.y = camera_transform.translation.y;
        }
        timer.tick(time.delta_seconds());
        if timer.finished() {
            bg.uv += Vec2::new(1. / 3., 0.);
            if bg.uv.x >= 0.99 {
                bg.uv = Vec2::new(0., 0.)
            }
        }
    }
}

fn unload_island_system(
    commands: &mut Commands,
    island_query: Query<Entity, With<LandIsland>>,
    bg_query: Query<Entity, With<BgFlag>>,
) {
    for entity in island_query.iter() {
        commands.despawn_recursive(entity);
    }
    for entity in bg_query.iter() {
        commands.despawn_recursive(entity);
    }
}
