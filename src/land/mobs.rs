use std::fmt::Debug;

use bevy::prelude::*;

use super::{
    pathfinding::Pathfinder, player::Player, player::PlayerMovedEvent,
    player::PlayerMovedEventReader,
};

pub struct LandMobsPlugin;
impl Plugin for LandMobsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(mob_movement_system.system());
    }
}

#[derive(Debug)]
pub enum MobKind {
    Crab,
}

pub struct Mob {
    pub kind: MobKind,
    pub speed: f32,
    pub pathfinder: Option<Box<dyn Pathfinder + Send + Sync>>,
}

impl Default for Mob {
    fn default() -> Self {
        Mob {
            kind: MobKind::Crab,
            speed: 2.,
            pathfinder: None,
        }
    }
}

fn mob_movement_system(
    mut event_reader: Local<PlayerMovedEventReader>,
    events: Res<Events<PlayerMovedEvent>>,
    time: Res<Time>,
    mut mob_query: Query<(&mut Mob, &mut Transform)>,
    mut player_query: Query<(&Player, &Transform)>,
) {
    let should_update = event_reader.reader.iter(&events).next().is_some();
    for (_, player_transform) in &mut player_query.iter() {
        let player_translation = player_transform.translation.truncate();
        for (mut mob, mut mob_transform) in &mut mob_query.iter() {
            let mob_translation = mob_transform.translation.truncate();
            let speed = mob.speed;
            if let Some(pathfinder) = mob.pathfinder.as_mut() {
                if should_update {
                    if pathfinder
                        .find_path(mob_translation, player_translation)
                        .is_err()
                    {
                        continue;
                    }
                }
                if let Ok(next_pos) = pathfinder.step(speed, time.delta_seconds) {
                    *mob_transform.translation.x_mut() = next_pos.x();
                    *mob_transform.translation.y_mut() = next_pos.y();
                }
            }
        }
    }
}
