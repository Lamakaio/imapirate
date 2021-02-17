#version 450
layout(location = 0) in vec4 world_position;
layout(location = 0) out vec4 o_Target;
layout(set=2, binding = 0) uniform TileUv_uv {
    vec2 uv;
};
layout(set=2, binding = 2) uniform TileUv_scale {
    float scale;
};
layout(set = 3, binding = 0) uniform texture2D TextureAtlas_texture;
layout(set = 3, binding = 1) uniform sampler TextureAtlas_texture_sampler;
void main() {
    vec2 screen_offset = mod(world_position.xy/scale , 64.) / vec2(192., 64.);
    o_Target = texture(
        sampler2D(TextureAtlas_texture, TextureAtlas_texture_sampler),
        uv+screen_offset);
}