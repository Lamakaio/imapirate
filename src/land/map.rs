use crate::tilemap::{Layer, TileMapBuilder};
use bevy::{ecs::bevy_utils::HashMap, prelude::*};

use super::{islands_from_map::Island, loader::LandHandles, LAND_SCALING};
pub struct LandMapPlugin;
impl Plugin for LandMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<LoadIslandEvent>()
            .init_resource::<LoadIslandEventReader>()
            .init_resource::<CurrentIsland>()
            .add_system(load_island_system.system());
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

#[derive(Default)]
struct LoadIslandEventReader {
    pub reader: EventReader<LoadIslandEvent>,
}

fn load_island_system(
    mut commands: Commands,
    events: Res<Events<LoadIslandEvent>>,
    islands: Res<HashMap<u64, Island>>,
    handles: Res<LandHandles>,
    mut event_reader: Local<LoadIslandEventReader>,
) {
    for event in event_reader.reader.iter(&events) {
        let island = islands.get(&event.island_id).expect("Island does no exist");
        let tilemap_builder = TileMapBuilder {
            layers: vec![Layer {
                tiles: island.map.clone(),
                atlas_handle: handles.tiles.clone(),
                ..Default::default()
            }],
            layer_offset: 1,
            transform: Transform::from_scale(LAND_SCALING * Vec3::one()),
            center: false,
            ..Default::default()
        };
        commands.spawn((tilemap_builder,));
    }
}
