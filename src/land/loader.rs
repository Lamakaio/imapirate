use bevy::prelude::*;

use crate::loading::GameState;

#[derive(Default)]
pub(crate) struct LandHandles {
    pub player: Handle<TextureAtlas>,
    pub tiles: Handle<TextureAtlas>,
    pub island_material: Handle<ColorMaterial>,
    pub sea_sheet: Handle<TextureAtlas>,
}

pub struct LandLoaderPlugin;
impl Plugin for LandLoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .init_resource::<LandHandles>()
            .on_state_exit(
                GameState::STAGE,
                GameState::Land,
                unload_system::<UnloadLandFlag>.system(),
            );
    }
}
pub struct UnloadLandFlag;
fn setup(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handles: ResMut<LandHandles>,
    //mut mobs_config: ResMut<Arc<Vec<(Handle<ColorMaterial>, MobConfig)>>>,
) {
    //loading textures
    let player_texture_handle = asset_server.load("sprites/land/chara_green_base.png");
    let texture_atlas = TextureAtlas::from_grid(player_texture_handle, Vec2::new(64., 64.), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    handles.player = texture_atlas_handle;

    let texture_handle_islands_spritesheet = asset_server.load("sprites/sea/sheet2.png");
    let islands_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle_islands_spritesheet,
        Vec2::new(16. as f32, 16. as f32),
        27,
        7,
        Vec2::new(1., 1.),
    );
    handles.island_material = materials.add(ColorMaterial::texture(islands_atlas.texture.clone()));
    let texture_atlas_handle = texture_atlases.add(islands_atlas);
    handles.tiles = texture_atlas_handle;

    let texture_handle_sea_spritesheet = asset_server.load("sprites/sea/seaTileSheet.png");

    let sea_atlas =
        TextureAtlas::from_grid(texture_handle_sea_spritesheet, Vec2::new(64., 64.), 3, 1);
    handles.sea_sheet = texture_atlases.add(sea_atlas);

    // *mobs_config = Arc::new(
    //     read_mob_config()
    //         .drain(..)
    //         .map(|mob_config| {
    //             let texture_handle =
    //                 asset_server.load(std::path::Path::new(&mob_config.sprite_path));
    //             (materials.add(texture_handle.into()), mob_config)
    //         })
    //         .collect(),
    // );
}

// fn load_system(
//     commands: &mut Commands,
//     events: Res<Events<LoadEvent>>,
//     save: Res<LandSaveState>,
//     handles: Res<LandHandles>,
//     current_island: Res<CurrentIsland>,
//     window: Res<WindowDescriptor>,
//     mut islands: ResMut<HashMap<u64, Island>>,
//     mut transition: ResMut<(f32, Vec3)>,
//     mut event_reader: Local<LoadEventReader>,
//     mut island_events: ResMut<Events<LoadIslandEvent>>,
//     mut camera_query: Query<(&Camera, &mut Transform)>,
// ) {
//     for event in event_reader.reader.iter(&events) {
//         let island_id = current_island.id;
//         if event.state == GameState::Land {
//             island_events.send(LoadIslandEvent { island_id });
//             let island = islands.get_mut(&island_id).expect("Island does no exist");
//             let (tile_x, tile_y) = current_island.entrance;
//             let (x, y) = (
//                 tile_x - island.rect.left as i32,
//                 tile_y - island.rect.bottom as i32,
//             );
//             let player_x = (x * TILE_SIZE) as f32 * LAND_SCALING;
//             let player_y = (y * TILE_SIZE) as f32 * LAND_SCALING;
//             //spawning entities
//             for (_camera, mut camera_transform) in camera_query.iter_mut() {
//                 let camera_x =
//                     player_x - (player_x % window.width as f32) + window.width as f32 / 2. + 0.5;
//                 let camera_y =
//                     player_y - (player_y % window.height as f32) + window.height as f32 / 2. + 0.5;
//                 camera_transform.translation.x = camera_x;
//                 camera_transform.translation.y = camera_y;
//                 *transition = (1., Vec3::new(camera_x, camera_y, 0.));
//             }
//             commands
//                 //player
//                 .spawn(SpriteSheetBundle {
//                     texture_atlas: handles.player.clone(),
//                     transform: Transform {
//                         translation: Vec3::new(player_x, player_y, BOAT_LAYER),
//                         scale: 2. * Vec3::one(),
//                         ..Default::default()
//                     },
//                     ..Default::default()
//                 })
//                 .with(save.player.clone())
//                 //flag
//                 .spawn((LandFlag,));
//             for (mob, transform) in island.mobs.drain(..) {
//                 commands //mob
//                     .spawn(SpriteBundle {
//                         material: mob.material.clone(),
//                         transform,
//                         ..Default::default()
//                     })
//                     .with(mob);
//             }
//         }
//     }
// }

// fn read_mob_config() -> Vec<MobConfig> {
//     let mob_config_string =
//         std::fs::read_to_string("config/mobs.ron").expect("mob config file not found");
//     ron::from_str(&mob_config_string).expect("syntax error on mobs config file")
// }
fn unload_system<T: Component>(commands: &mut Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.despawn_recursive(entity);
    }
}
