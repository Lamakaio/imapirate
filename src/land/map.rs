use std::time::Duration;

use crate::{
    background::{BackgroundBundle, TileUv},
    loading::GameState,
    sea::{map::Islands, player::PlayerPositionUpdate},
    util::SeededHasher,
};
use bevy::{prelude::*, render::camera::Camera};

use super::{
    loader::{LandHandles, MobsConfig, UnloadLandFlag},
    mobs::generate_mobs,
    LAND_SCALING,
};
pub struct LandMapPlugin;
impl Plugin for LandMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LoadIslandEvent>()
            .init_resource::<CurrentIsland>()
            .on_state_enter(GameState::STAGE, GameState::Land, load_island.system())
            .on_state_update(GameState::STAGE, GameState::Land, move_anim_bg.system())
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                generate_islands_features.system(),
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

fn load_island(
    commands: &mut Commands,
    sea_player_pos: Res<PlayerPositionUpdate>,
    mut islands: ResMut<Islands>,
    handles: Res<LandHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let island = &mut islands.0[sea_player_pos.island_id.unwrap() as usize];
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
        .with(UnloadLandFlag);
    //initializing the sea animation
    let mut transform = Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.));
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
        .with(UnloadLandFlag);
}

fn move_anim_bg(
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

fn generate_islands_features(
    mut islands: ResMut<Islands>,
    mut id: Local<usize>,
    hasher: Res<SeededHasher>,
    mobs_config: Res<MobsConfig>,
) {
    for i in *id..islands.0.len() {
        generate_mobs(&mobs_config, &mut islands.0[i], hasher.get_hasher())
    }
    *id = islands.0.len();
}
