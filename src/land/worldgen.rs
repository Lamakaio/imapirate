use bevy::ecs::bevy_utils::HashMap;
use seahash::SeaHasher;

use super::islands_from_map::Island;

pub fn generate_features(islands: &mut HashMap<u64, Island>, mut _hasher: SeaHasher) {
    for (_key, _island) in islands.iter() {}
}
