use bevy::ecs::bevy_utils::HashMap;

use super::islands_from_map::Island;

pub fn generate_features(islands: &mut HashMap<u64, Island>) {
    for (_key, _island) in islands.iter() {}
}
