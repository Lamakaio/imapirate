use crate::loading::GameState;

use super::{
    super::background::{BackgroundBundle, TileUv},
    loader::SeaHandles,
};
use bevy::{prelude::*, render::camera::Camera};
use bevy_rapier2d::rapier::geometry::InteractionGroups;
use bevy_tilemap::{
    prelude::{LayerKind, TilemapBuilder, TilemapBundle, TilemapDefaultPlugins},
    TilemapLayer,
};

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileKind {
    Sand(bool),
    Forest,
    Sea(bool),
}
impl Default for TileKind {
    fn default() -> Self {
        TileKind::Sea(false)
    }
}

pub struct SeaMapPlugin;
impl Plugin for SeaMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugins(TilemapDefaultPlugins)
            .on_state_enter(GameState::STAGE, GameState::Sea, load_map_system.system())
            .init_resource::<Time>()
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                move_anim_bg_system.system(),
            );
    }
}

fn move_anim_bg_system(
    mut bg_query: Query<(&mut TileUv, &mut Transform)>,
    camera_query: Query<(&Camera, &Transform)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    if timer.is_none() {
        *timer = Some(Timer::new(Duration::from_millis(500), true));
    }
    let timer = timer.as_mut().unwrap();
    for (mut bg, mut bg_transform) in bg_query.iter_mut() {
        for (_, camera_transform) in camera_query.iter() {
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

fn load_map_system(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    handles: Res<SeaHandles>,
) {
    let tilemap = TilemapBuilder::new()
        .texture_atlas(handles.islands_sheet.clone())
        .tile_dimensions(16, 16)
        .add_layer(
            TilemapLayer {
                kind: LayerKind::Sparse,
                interaction_groups: InteractionGroups::new(
                    0b0000_0000_0000_0001,
                    0b0000_0000_0000_0010,
                ),
                is_sensor: true,
            },
            0,
        )
        .add_layer(
            TilemapLayer {
                kind: LayerKind::Sparse,
                interaction_groups: InteractionGroups::new(
                    0b0000_0000_0000_0001,
                    0b0000_0000_0000_0010,
                ),
                ..Default::default()
            },
            1,
        )
        .z_layers(2)
        .auto_chunk()
        .auto_spawn(1, 1)
        .finish()
        .unwrap();
    let tilemap_components = TilemapBundle {
        tilemap,
        transform: Default::default(),
        global_transform: Default::default(),
    };
    //initializing the sea animation
    let mut transform = Transform::from_rotation(Quat::from_rotation_x(3.1415926535 / 2.));
    transform.translation.z = -10.;
    commands
        .spawn(BackgroundBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
            transform,
            texture_atlas: handles.sea_sheet.clone(),
            tile_uv: TileUv {
                uv: Vec2::new(0.0, 0.0),
                scale: 1.,
            },
            ..Default::default()
        })
        .spawn(tilemap_components);
}
