use std::sync::Arc;

use bevy::render::pipeline::PipelineDescriptor;
use bevy::{asset::LoadState, prelude::*};
use parry2d::shape::TriMesh;

use crate::{loading::GameState, util::texture_atlas_to_trimeshes};

use super::{player::PlayerPositionUpdate, worldgen::Biome, ISLAND_SCALING, TILE_SIZE};

#[derive(Default)]
pub struct SeaHandles {
    pub sea_pipeline: Handle<PipelineDescriptor>,
    pub sea_sheet: Handle<TextureAtlas>,
    pub islands_sheet: Handle<TextureAtlas>,
    pub boat: Handle<TextureAtlas>,
    pub boat_collisions: Handle<TextureAtlas>,
    pub boat_meshes: Vec<TriMesh>,
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
            .add_system(on_loaded.system())
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
    let texture_handle = asset_server.load("sprites/sea/ship_collisions_sheet2.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::new(133., 133.),
        8,
        1,
        Vec2::new(1., 1.),
    );
    let texture_atlas_handle = atlases.add(texture_atlas);
    handles.boat_collisions = texture_atlas_handle;
}

fn on_loaded(
    asset_server: Res<AssetServer>,
    mut handles: ResMut<SeaHandles>,
    atlases: Res<Assets<TextureAtlas>>,
    textures: Res<Assets<Texture>>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }
    let texture_atlas = atlases.get(handles.boat_collisions.clone()).unwrap();
    if asset_server.get_load_state(texture_atlas.texture.clone()) != LoadState::Loaded {
        return;
    };
    *loaded = true;
    let texture = textures.get(texture_atlas.texture.clone()).unwrap();
    let trimeshes = texture_atlas_to_trimeshes(texture_atlas, texture, 1. / ISLAND_SCALING);
    handles.boat_meshes = trimeshes;
}

fn enter_island_system(
    keyboard_input: Res<Input<KeyCode>>,
    pos_update: Res<PlayerPositionUpdate>,
    mut state: ResMut<State<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Return)
        && pos_update.island_id.is_some()
        && state.current() == &GameState::Sea
    {
        state.overwrite_next(GameState::Land).unwrap();
    }

    if keyboard_input.just_pressed(KeyCode::Return) && state.current() == &GameState::Land {
        state.overwrite_next(GameState::Sea).unwrap();
    }
}
