use bevy::{
    prelude::*,
    render::{pipeline::PipelineDescriptor, render_graph::RenderGraph},
};

pub use self::sea::SeaMaterial;
use self::sea::{get_sea_material_pipeline, SeaMaterialPlugin};
mod sea;

pub struct MaterialsPlugin;
impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_plugin(SeaMaterialPlugin)
            .init_resource::<Pipelines>();
    }
}

#[derive(Default)]
pub struct Pipelines {
    pub sea: RenderPipelines,
}

fn setup(
    mut pipelines_storage: ResMut<Pipelines>,
    pipelines: ResMut<Assets<PipelineDescriptor>>,
    shaders: ResMut<Assets<Shader>>,
    render_graph: ResMut<RenderGraph>,
) {
    let pipelines = get_sea_material_pipeline(pipelines, shaders, render_graph);
    pipelines_storage.sea = pipelines.clone();
}
