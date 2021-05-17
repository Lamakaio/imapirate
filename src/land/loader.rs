use std::sync::Arc;

use bevy::prelude::*;

use crate::loading::GameState;

use super::mobs::MobConfig;

#[derive(Default)]
pub(crate) struct LandHandles {
    pub player: Handle<TextureAtlas>,
    pub player_sword: Handle<TextureAtlas>,
    pub player_gun: Handle<TextureAtlas>,
    pub player_sword_collisions: Handle<TextureAtlas>,
    pub tiles: Handle<TextureAtlas>,
    pub island_material: Handle<ColorMaterial>,
    pub sea_sheet: Handle<TextureAtlas>,
    pub bullet_material: Handle<ColorMaterial>,
}

pub struct LandLoaderPlugin;
impl Plugin for LandLoaderPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .init_resource::<LandHandles>()
            .on_state_exit(
                GameState::STAGE,
                GameState::Land,
                unload::<UnloadLandFlag>.system(),
            );
    }
}
#[derive(Default)]
pub struct MobsConfig(pub Arc<Vec<(Handle<ColorMaterial>, MobConfig)>>);
pub struct UnloadLandFlag;
fn setup(
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut handles: ResMut<LandHandles>,
    mut mobs_config: ResMut<MobsConfig>,
) {
    //loading textures
    let player_texture_handle = asset_server.load("sprites/land/chara_green_base.png");
    let texture_atlas = TextureAtlas::from_grid_with_padding(
        player_texture_handle,
        Vec2::new(64., 64.),
        4,
        1,
        Vec2::new(1., 1.),
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    handles.player = texture_atlas_handle;

    let sword_texture_handle = asset_server.load("sprites/land/chara_sword.png");
    let texture_atlas = TextureAtlas::from_grid(sword_texture_handle, Vec2::new(64., 64.), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    handles.player_sword = texture_atlas_handle;

    let gun_texture_handle = asset_server.load("sprites/land/chara_gun.png");
    let texture_atlas = TextureAtlas::from_grid(gun_texture_handle, Vec2::new(64., 64.), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    handles.player_gun = texture_atlas_handle;

    let sword_collisions_texture_handle =
        asset_server.load("sprites/land/chara_sword_collisions.png");
    let texture_atlas =
        TextureAtlas::from_grid(sword_collisions_texture_handle, Vec2::new(64., 64.), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    handles.player_sword_collisions = texture_atlas_handle;

    let texture_handle_islands_spritesheet = asset_server.load("sprites/sea/sheet2.png");
    let islands_atlas = TextureAtlas::from_grid_with_padding(
        texture_handle_islands_spritesheet,
        Vec2::new(16., 16.),
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

    handles.bullet_material = materials.add(asset_server.load("sprites/land/bullet.png").into());

    *mobs_config = MobsConfig(Arc::new(
        read_mob_config()
            .drain(..)
            .map(|mob_config| {
                let texture_handle =
                    asset_server.load(std::path::Path::new(&mob_config.sprite_path));
                (materials.add(texture_handle.into()), mob_config)
            })
            .collect(),
    ));
}

fn read_mob_config() -> Vec<MobConfig> {
    let mob_config_string =
        std::fs::read_to_string("config/mobs.ron").expect("mob config file not found");
    ron::from_str(&mob_config_string).expect("syntax error on mobs config file")
}
fn unload<T: Component>(commands: &mut Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.despawn_recursive(entity);
    }
}
