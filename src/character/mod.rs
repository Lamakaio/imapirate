use bevy::prelude::*;

pub struct CharacterPlugin;

pub struct CharacterSheet {
    pub stats: CharacterStats,
    pub values: CharacterValues,
}

pub struct CharacterStats {
    pub max_life: u32,
}
pub struct CharacterValues {
    pub life: u32,
}
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(CharacterSheet {
            stats: CharacterStats { max_life: 100 },
            values: CharacterValues { life: 100 },
        });
    }
}
