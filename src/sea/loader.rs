use std::sync::Arc;

use bevy::prelude::*;
use bevy::render::pipeline::PipelineDescriptor;

use super::{worldgen::Biome, TILE_SIZE};

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
        app //.add_system(unload_system.system())
            //.add_system(enter_island_system.system())
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
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(168., 168.), 8, 1);
    let texture_atlas_handle = atlases.add(texture_atlas);
    handles.boat = texture_atlas_handle;
}

// fn unload_system(
//     commands: &mut Commands,
//     events: Res<Events<LoadEvent>>,
//     mut save: ResMut<SeaSaveState>,
//     mut event_reader: Local<LoadEventReader>,
//     mut chunks: ResMut<HashMap<(i32, i32), Chunk>>,
//     player_query: Query<(Entity, &Transform, &Player)>,
//     chunk_query: Query<(Entity, &Transform, &ChunkLayer)>,
//     flag_query: Query<(Entity, &SeaFlag)>,
// ) {
//     for event in event_reader.reader.iter(&events) {
//         if event.state != GameState::Sea {
//             for (flag_entity, _) in &mut flag_query.iter() {
//                 //only despawn things if the flag is there
//                 commands.despawn(flag_entity);
//                 for (entity, transform, player) in &mut player_query.iter() {
//                     save.player = player.clone();
//                     save.player_transform = *transform;
//                     commands.despawn(entity);
//                 }
//                 for (entity, tile_pos, _) in &mut chunk_query.iter() {
//                     let tile_pos = tile_pos.translation;
//                     let chunk_x =
//                         (tile_pos.x / (TILE_SIZE * SCALING * CHUNK_SIZE) as f32).floor() as i32;
//                     let chunk_y =
//                         (tile_pos.y / (TILE_SIZE * SCALING * CHUNK_SIZE) as f32).floor() as i32;
//                     if let Some(chunk) = chunks.get_mut(&(chunk_x, chunk_y)) {
//                         chunk.drawn = false;
//                         commands.despawn(entity);
//                     } else {
//                         panic!("Attempted to despawn nonexistent chunk !");
//                     }
//                 }
//             }
//         }
//     }
// }

// fn enter_island_system(
//     keyboard_input: Res<Input<KeyCode>>,
//     pos_update: Res<PlayerPositionUpdate>,
//     mut events: ResMut<Events<LoadEvent>>,
//     mut current_island: ResMut<CurrentIsland>,
// ) {
//     match pos_update.collision_status {
//         CollisionType::Friction(Some(id)) | CollisionType::Rigid(Some(id)) => {
//             if keyboard_input.just_pressed(KeyCode::A) {
//                 current_island.id = id;
//                 current_island.entrance = (pos_update.tile_x, pos_update.tile_y);
//                 events.send(LoadEvent {
//                     state: GameState::Land,
//                 });
//             }
//         }
//         _ => (),
//     }

//     if keyboard_input.just_pressed(KeyCode::B) {
//         events.send(LoadEvent {
//             state: GameState::Sea,
//         });
//     }
// }
