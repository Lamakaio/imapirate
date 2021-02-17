use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::pipeline::PipelineDescriptor;

use crate::loading::GameState;

use super::{player::PlayerPositionUpdate, worldgen::Biome, TILE_SIZE};

#[derive(Default)]
pub struct SeaHandles {
    pub sea_pipeline: Handle<PipelineDescriptor>,
    pub sea_sheet: Handle<TextureAtlas>,
    pub islands_sheet: Handle<TextureAtlas>,
    pub boat: Handle<TextureAtlas>,
    pub islands_material: Handle<ColorMaterial>,
}

#[derive(Clone, Default)]
pub struct BiomeConfig(pub Arc<Vec<(Handle<TextureAtlas>, Biome)>>);
pub struct SeaLoaderPlugin;
impl Plugin for SeaLoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let worldgen_config = {
            // let asset_server = app
            //     .resources()
            //     .get::<AssetServer>()
            //     .expect("Asset server not found");
            // let mut atlases = app
            //     .resources()
            //     .get_mut::<Assets<TextureAtlas>>()
            //     .expect("Asset server not found");
            BiomeConfig(Arc::new(
                read_worldgen_config()
                    .drain(..)
                    .map(|worldgen_config| {
                        // let texture_handle =
                        //     asset_server.load(std::path::Path::new(&worldgen_config.sea_sheet));
                        // let atlas =
                        //     TextureAtlas::from_grid(texture_handle, Vec2::new(16., 16.), 4, 47);
                        (Handle::default(), worldgen_config)
                    })
                    .collect(),
            ))
        };
        app.add_system(enter_island_system.system())
            .add_startup_system(setup.system())
            .init_resource::<SeaHandles>()
            .insert_resource(worldgen_config);
    }
}

fn read_worldgen_config() -> Vec<Biome> {
    let worldgen_config_string =
        std::fs::read_to_string("config/worldgen.ron").expect("worldgen config file not found");
    ron::from_str(&worldgen_config_string).expect("syntax error on worldgen config file")
}

fn setup(
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handles: ResMut<SeaHandles>,
) {
    //loading textures
    let texture_handle_sea_spritesheet = asset_server.load("sprites/sea/seaTileSheet.png");

    let sea_atlas =
        TextureAtlas::from_grid(texture_handle_sea_spritesheet, Vec2::new(64., 64.), 3, 1);
    handles.sea_sheet = atlases.add(sea_atlas);

    let texture_handle_islands_spritesheet = asset_server.load("sprites/sea/sheet2.png");
    let islands_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle_islands_spritesheet,
        Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
        27,
        7,
        Vec2::new(1., 1.),
    );
    handles.islands_material = materials.add(ColorMaterial::texture(islands_atlas.texture.clone()));
    handles.islands_sheet = atlases.add(islands_atlas);
    let texture_handle = asset_server.load("sprites/sea/ship_sheet.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::new(133., 133.),
        8,
        1,
        Vec2::new(1., 1.),
    );
    let texture_atlas_handle = atlases.add(texture_atlas);
    handles.boat = texture_atlas_handle;
}

fn enter_island_system(
    keyboard_input: Res<Input<KeyCode>>,
    pos_update: Res<PlayerPositionUpdate>,
    mut state: ResMut<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::A) && pos_update.island_id.is_some() {
        state.overwrite_next(GameState::Land).unwrap();
    }

    if keyboard_input.just_pressed(KeyCode::B) {
        state.overwrite_next(GameState::Sea).unwrap();
    }
}
