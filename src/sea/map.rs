use crate::loading::GameState;

use super::{
    super::background::{BackgroundBundle, TileUv},
    collision::IslandSpawnEvent,
    loader::SeaHandles,
    worldgen::Island,
};
use bevy::{
    prelude::*,
    render::{camera::Camera, render_graph::base::MainPass},
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
        app.on_state_enter(GameState::STAGE, GameState::Sea, load_map_system.system())
            .init_resource::<Time>()
            //.init_resource::<Vec<Island>>()
            .on_state_update(
                GameState::STAGE,
                GameState::Sea,
                move_anim_bg_system.system(),
            )
            // .on_state_update(
            //     GameState::STAGE,
            //     GameState::Sea,
            //     spawn_island_system.system(),
            // )
            ;
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
    //initializing the sea animation
    let mut transform = Transform::from_rotation(Quat::from_rotation_x(3.1415926535 / 2.));
    transform.translation.z = -10.;
    commands.spawn(BackgroundBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
        transform,
        texture_atlas: handles.sea_sheet.clone(),
        tile_uv: TileUv {
            uv: Vec2::new(0.0, 0.0),
            scale: 1.,
        },
        ..Default::default()
    });
}
#[derive(Bundle)]
pub struct IslandBundle {
    pub sprite: Sprite,
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Clone for IslandBundle {
    fn clone(&self) -> Self {
        IslandBundle {
            main_pass: MainPass,
            sprite: Sprite {
                size: self.sprite.size,
                resize_mode: match self.sprite.resize_mode {
                    SpriteResizeMode::Automatic => SpriteResizeMode::Automatic,
                    SpriteResizeMode::Manual => SpriteResizeMode::Manual,
                }, //SpriteResizeMode doesn't derive Clone
            },
            material: self.material.clone(),
            render_pipelines: self.render_pipelines.clone(),
            draw: self.draw.clone(),
            mesh: self.mesh.clone(),
            visible: self.visible.clone(),
            transform: self.transform,
            global_transform: self.global_transform,
        }
    }
}

impl Default for IslandBundle {
    fn default() -> Self {
        Self {
            mesh: bevy::sprite::QUAD_HANDLE.typed(),
            render_pipelines: RenderPipelines::from_pipelines(vec![
                bevy::render::pipeline::RenderPipeline::new(
                    bevy::sprite::SPRITE_PIPELINE_HANDLE.typed(),
                ),
            ]), //the default sprite render pipeline
            sprite: Sprite {
                size: Vec2::new(1., 1.),
                resize_mode: SpriteResizeMode::Manual,
            }, //SpriteResizeMode must be set to manual because we use a spritesheet and not an individual sprite.
            main_pass: MainPass,
            visible: Visible {
                is_transparent: true,
                is_visible: true,
            },
            material: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            draw: Draw::default(),
        }
    }
}
fn spawn_island_system(
    commands: &mut Commands,
    events: Res<Events<IslandSpawnEvent>>,
    mut event_reader: Local<EventReader<IslandSpawnEvent>>,
    mut islands: ResMut<Vec<Island>>,
) {
    for event in event_reader.iter(&events) {
        match event {
            IslandSpawnEvent::Spawn(island_id) => {
                let island = &mut islands[*island_id as usize];
                // let entity = commands
                //     .spawn(IslandBundle {
                //         mesh: island.mesh.clone(),
                //         transform: Transform::from_translation(Vec3::new(0., 0., 3.)),
                //         ..Default::default()
                //     })
                //     .current_entity();
                // island.entity = entity;
            }
            IslandSpawnEvent::Despawn(island_id) => {
                let island = &mut islands[*island_id as usize];
                let entity = island.entity.take();
                if let Some(entity) = entity {
                    commands.despawn_recursive(entity);
                }
            }
        }
    }
}
