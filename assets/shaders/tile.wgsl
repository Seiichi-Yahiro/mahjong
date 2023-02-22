#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::pbr_types
#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

@group(1) @binding(0)
var mesh_texture: texture_2d<f32>;
@group(1) @binding(1)
var mesh_texture_sampler: sampler;
@group(1) @binding(2)
var cover_texture: texture_2d_array<f32>;
@group(1) @binding(3)
var cover_texture_sampler: sampler;
@group(1) @binding(4)
var<uniform> cover_id: i32;

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {

    // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
    // the material members
    var pbr_input: PbrInput = pbr_input_new();

    var cover_color = textureSample(cover_texture, cover_texture_sampler, in.uv / vec2(0.4), cover_id);
    var tile_color = textureSample(mesh_texture, mesh_texture_sampler, in.uv);

    var base_color = cover_color.w * cover_color.xyz + (1.0 - cover_color.w) * tile_color.xyz;
    pbr_input.material.base_color = vec4(base_color, 0.0);

    pbr_input.material.perceptual_roughness = cover_color.w * 0.7 + (1.0 - cover_color.w) * 0.15;


#ifdef VERTEX_COLORS
    pbr_input.material.base_color = pbr_input.material.base_color * in.color;
#endif

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = prepare_world_normal(
        in.world_normal,
        (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u,
        in.is_front,
    );

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
#ifdef VERTEX_TANGENTS
#ifdef STANDARDMATERIAL_NORMAL_MAP
        in.world_tangent,
#endif
#endif
        in.uv,
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    return tone_mapping(pbr(pbr_input));
}