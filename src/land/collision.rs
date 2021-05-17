use bevy::prelude::*;
use kdtree_collisions::KdValue;

pub struct LandCollisionPlugin;
impl Plugin for LandCollisionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<LandCollisionTree>();
    }
}

#[derive(Debug, Default)]
pub struct LandCollisionTree(pub kdtree_collisions::KdTree<LandValue, 16>);

#[derive(Clone, Debug, Default)]
pub struct LandValue {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub id: LandId,
}

#[derive(Clone, Debug)]
pub enum LandId {
    None,
    Mob(Entity),
}
impl Default for LandId {
    fn default() -> Self {
        LandId::None
    }
}

impl KdValue for LandValue {
    type Position = f32;

    fn min_x(&self) -> Self::Position {
        self.min_x
    }

    fn min_y(&self) -> Self::Position {
        self.min_y
    }

    fn max_x(&self) -> Self::Position {
        self.max_x
    }

    fn max_y(&self) -> Self::Position {
        self.max_y
    }
}
