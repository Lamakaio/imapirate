use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{
            base::{self, MainPass},
            RenderGraph, RenderResourcesNode,
        },
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};
#[derive(Default)]
pub struct SeaBackgroundPlugin;
impl Plugin for SeaBackgroundPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let resources = app.resources_mut();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        render_graph.add_sea_background_graph(resources);
    }
}

const VERTEX_SHADER: &str = include_str!("sea_bg.vert");

const FRAGMENT_SHADER: &str = include_str!("sea_bg.frag");

pub const SEA_BACKGROUND_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 0xe5081d9abddbfad6);

fn build_sea_background_pipeline(shaders: &mut Assets<Shader>) -> PipelineDescriptor {
    PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    })
}

pub trait SeaBackgroundRenderGraphBuilder {
    fn add_sea_background_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl SeaBackgroundRenderGraphBuilder for RenderGraph {
    fn add_sea_background_graph(&mut self, resources: &Resources) -> &mut Self {
        // Add an AssetRenderResourcesNode to our Render Graph. This will bind TileUv resources to our shader
        self.add_system_node("sea_background", RenderResourcesNode::<TileUv>::new(true));

        // Add a Render Graph edge connecting our new "sea_background" node to the main pass node. This ensures "sea_background" runs before the main pass
        self.add_node_edge("sea_background", base::node::MAIN_PASS)
            .unwrap();
        let mut pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
        let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();
        pipelines.set_untracked(
            SEA_BACKGROUND_PIPELINE_HANDLE,
            build_sea_background_pipeline(&mut shaders),
        );
        self
    }
}
#[derive(Debug, Default, RenderResources, TypeUuid, Reflect)]
#[uuid = "66fb00ca-60e9-4852-bf71-d82048b706a2"]
pub struct TileUv {
    pub uv: Vec2,
    pub scale: f32,
}
#[derive(Bundle)]
pub(crate) struct BackgroundBundle {
    /// The handle for a TextureAtlas which contains multiple textures.
    pub texture_atlas: Handle<TextureAtlas>,
    /// A component that indicates how to draw a component.
    pub draw: Draw,
    /// A component that indicates if the component is visible.
    pub visible: Visible,
    /// The pipeline for the renderer.
    pub render_pipelines: RenderPipelines,
    /// A component that indicates that an entity should be drawn in the
    /// "main pass"
    pub main_pass: MainPass,
    /// A mesh of vertices for a component.
    pub mesh: Handle<Mesh>,
    /// The transform location in a space for a component.
    pub transform: Transform,
    /// The global transform location in a space for a component.
    pub global_transform: GlobalTransform,
    pub tile_uv: TileUv,
    pub flag: BgFlag,
}
pub struct BgFlag;
impl Default for BackgroundBundle {
    fn default() -> Self {
        Self {
            mesh: Default::default(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                SEA_BACKGROUND_PIPELINE_HANDLE.typed(),
            )]),
            transform: Transform::from_rotation(Quat::from_rotation_x(3.1415926535 / 2.)),
            texture_atlas: Default::default(),
            draw: Default::default(),
            visible: Default::default(),
            main_pass: Default::default(),
            global_transform: Default::default(),
            tile_uv: TileUv {
                uv: Default::default(),
                scale: 1.,
            },
            flag: BgFlag,
        }
    }
}
