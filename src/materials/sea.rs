use bevy::{
    prelude::*,
    render::{
        camera::Camera,
        pipeline::{DynamicBinding, PipelineDescriptor, PipelineSpecialization, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
    type_registry::TypeUuid,
};

pub struct SeaMaterialPlugin;
impl Plugin for SeaMaterialPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<SeaMaterial>()
            .add_system(sea_mat_system.system());
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "7ebc8e5f-5722-4eee-9c7d-148611405be1"]
#[repr(C)]
pub struct SeaMaterial {
    pub time: Vec4,
    pub values: Vec3, //can't make it work otherwise
}

fn sea_mat_system(
    time: Res<Time>,
    mut materials: ResMut<Assets<SeaMaterial>>,
    mut material_query: Query<(&Draw, &Handle<SeaMaterial>)>,
    mut camera_query: Query<(&Camera, &Transform)>,
) {
    for (_, camera_transform) in &mut camera_query.iter() {
        for (_, mat_handle) in &mut material_query.iter() {
            let material = materials.get_mut(mat_handle).unwrap();
            material.time = Vec4::new(
                time.seconds_since_startup as f32,
                0.1,
                camera_transform.translation.x(),
                -camera_transform.translation.y(),
            );
            // material.values[2] = camera_transform.translation.x();
            // material.values[3] = camera_transform.translation.y();
        }
    }
}

const VERTEX_SHADER: &str = r#"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = include_str!("../shaders/sea.frag");

pub fn get_sea_material_pipeline(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut render_graph: ResMut<RenderGraph>,
) -> RenderPipelines {
    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to our shader
    render_graph.add_system_node(
        "sea_material",
        AssetRenderResourcesNode::<SeaMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge("sea_material", base::node::MAIN_PASS)
        .unwrap();

    // Setup our world
    RenderPipelines::from_pipelines(vec![RenderPipeline::specialized(
        pipeline_handle,
        // NOTE: in the future you wont need to manually declare dynamic bindings
        PipelineSpecialization {
            dynamic_bindings: vec![
                // Transform
                DynamicBinding {
                    bind_group: 1,
                    binding: 0,
                },
                // zoom
                DynamicBinding {
                    bind_group: 1,
                    binding: 1,
                },
            ],
            ..Default::default()
        },
    )])
}
