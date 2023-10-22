#define_import_path bevy_pbr::fragment

#import bevy_pbr::pbr_functions as pbr_functions
//#import bevy_pbr::pbr_bindings as pbr_bindings
#import bevy_pbr::pbr_types as pbr_types
#import bevy_pbr::prepass_utils

#import bevy_pbr::mesh_vertex_output       MeshVertexOutput
#import bevy_pbr::mesh_bindings            mesh
#import bevy_pbr::mesh_view_bindings       view, fog, screen_space_ambient_occlusion_texture
#import bevy_pbr::mesh_view_types          FOG_MODE_OFF
#import bevy_core_pipeline::tonemapping    screen_space_dither, powsafe, tone_mapping
#import bevy_pbr::parallax_mapping         parallaxed_uv

#import bevy_pbr::prepass_utils

#ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
#import bevy_pbr::gtao_utils gtao_multibounce
#endif

// setting textures
// @group(1) @binding(0)
// var<uniform> material: pbr_types::StandardMaterial;

@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(2)
var metallic_texture: texture_2d<f32>;

@group(1) @binding(3)
var normal_map_texture: texture_2d<f32>;

@group(1) @binding(4)
var texture_map: texture_2d<f32>;

// var cell_size: f32 = 1.0;

// @group(1) @binding(4)
// var texture_map: texture_2d<f32>;

// fn get_tile_colour(uv: vec2<f32>, tile_cord: vec2<u32>) -> vec4<f32> {
//     let col_dim: vec2<u32> = textureDimensions(pbr_bindings::base_color_texture);
//     var tile_dim = vec2<f32>(col_dim);
//     tile_dim.x = tile_dim.x / 4.0;
//     tile_dim.y = tile_dim.y / 4.0;
//     let coord: vec2<u32> = vec2<u32>(
//         u32(uv.x * tile_dim.x + f32(tile_cord.x % 4u) * tile_dim.x),
//         u32(uv.y * tile_dim.y + f32(tile_cord.y % 4u) * tile_dim.y)
//     );
//     return textureLoad(pbr_bindings::base_color_texture, coord, 0);
// }

struct AtlasMetaData {
    width: u32,
    height: u32,
}

fn get_meta_data() -> AtlasMetaData {
    let dim: vec2<u32> = textureDimensions(texture_map);

    var atlas_meta_data: AtlasMetaData;

    atlas_meta_data.width = dim.x / 4u;
    atlas_meta_data.height = dim.y;

    return atlas_meta_data;
}

struct TextureMapData {
    position: vec2<u32>,
    rotation: u32,
}

fn get_texture_data(uv: vec2<f32>, meta_data: AtlasMetaData) -> TextureMapData {
    let coord: vec2<u32> = vec2<u32>(
        u32(uv.x * f32(meta_data.width) * 4.),
        u32(uv.y * f32(meta_data.height))
    );

    let texture_atlas_data = textureLoad(texture_map, coord, 0);

    var texture_map_data : TextureMapData;

    texture_map_data.position = vec2<u32>(
        u32(texture_atlas_data.x * f32(meta_data.width)),
        u32(texture_atlas_data.y * f32(meta_data.height))
    );

    texture_map_data.rotation = u32(texture_atlas_data.z * 4.);

    return texture_map_data;
}

fn rotate_uv(coord: vec2<f32>, rotation: u32) -> vec2<f32> {
    var new_coord = vec2<f32>(0.,0.);

    switch rotation % 4u {
        case 0u {
            new_coord = coord;
        }
        case 1u {
            new_coord.x = 1.0 - coord.y;
            new_coord.y = coord.x;
        }
        case 2u {
            new_coord.x = 1.0 - coord.x;
            new_coord.y = 1.0 - coord.y;
        }
        case 3u {
            new_coord.x = coord.y;
            new_coord.y = 1.0 - coord.x;
        }
        default {

        }
    }

    return new_coord;
}

fn uv_coord_to_texel_coord(coord: vec2<f32>, meta_data: AtlasMetaData) -> vec2<u32> {
    return vec2<u32> (
        u32(coord.x * f32(meta_data.width)),
        u32(coord.y * f32(meta_data.height)),
    );
}

fn get_texel(uv: vec2<f32>, texture: texture_2d<f32>) -> vec4<f32> {
    let dim: vec2<u32> = textureDimensions(texture);
    
    let texel_coord = vec2<u32> (
        u32(uv.x * f32(dim.x)),
        u32(uv.y * f32(dim.y))
    );

    return textureLoad(texture, texel_coord, 0);
}
fn get_texel_from_atlas(
    uv: vec2<f32>,
    texture_coord: vec2<u32>,
    meta_data: AtlasMetaData,
    texture: texture_2d<f32>
) -> vec4<f32> {
    var dim: vec2<u32> = textureDimensions(texture);
    
    dim.x = dim.x / meta_data.width;
    dim.y = dim.y / meta_data.height;

    let offset = vec2<u32> (
        dim.x * texture_coord.x,
        dim.y * texture_coord.y,
    );

    let texel_coord = vec2<u32> (
        u32(uv.x * f32(dim.x)),
        u32(uv.y * f32(dim.y))
    );

    return textureLoad(texture, offset+texel_coord, 0);
}

fn apply_normal_mapping(
    world_normal: vec3<f32>,
    
    #ifdef VERTEX_TANGENTS
        world_tangent: vec4<f32>,
    #endif,
    normal_texture: vec3<f32>
) -> vec3<f32> {
    var N: vec3<f32> = world_normal;

    #ifdef VERTEX_TANGENTS
        var T: vec3<f32> = world_tangent.xyz;
        var B: vec3<f32> = world_tangent.w * cross(N, T);

        var Nt = normal_texture * 2.0 - 1.0;

        //Nt.y = -1 * Nt.y;

        N = Nt.x * T + Nt.y * B + Nt.z * N;

    #endif

    return normalize(N);
}

@fragment
fn fragment(
    in: MeshVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let meta_data = get_meta_data();
    
    let is_orthographic = view.projection[3].w == 1.0;
    let V = pbr_functions::calculate_view(in.world_position, is_orthographic);
    
    let texture_data = get_texture_data(in.uv, meta_data);

    var uv = vec2<f32> (
        abs(in.world_position.x - in.position.x) % 1.0,
        abs(in.world_position.z - in.position.z) % 1.0
    );

    let old_uv = uv;

    uv = rotate_uv(uv, texture_data.rotation);

    let normal: vec3<f32> = get_texel(
        old_uv,
        normal_map_texture
    ).rgb;

    let colour = get_texel_from_atlas(
        uv,
        texture_data.position,
        meta_data,
        base_color_texture
    );

    let metallic_and_roughness = get_texel_from_atlas(
        uv,
        texture_data.position,
        meta_data,
        metallic_texture
    ).x;

    let metallic = clamp(
        metallic_and_roughness,
        0.2,
        0.95
    );
    
    let roughness = clamp(
        1. - metallic_and_roughness,
        0.3,
        1.0
    );

    var output_color: vec4<f32> = colour;

    var pbr_input: pbr_functions::PbrInput;
    
    //pbr material
    pbr_input.material.base_color = output_color;
    pbr_input.material.emissive = vec4<f32>(0.);
    pbr_input.material.perceptual_roughness = roughness;
    pbr_input.material.metallic = metallic;

    pbr_input.material.flags = pbr_types::STANDARD_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE;
    pbr_input.material.alpha_cutoff = 0.5;
    pbr_input.material.parallax_depth_scale = 0.1;
    pbr_input.material.max_parallax_layer_count = 16.0;
    pbr_input.material.max_relief_mapping_search_steps = 5u;
    //pbr_input.material.deferred_lighting_pass_id = 1u;
    
    //occlusion
    var occlusion: vec3<f32> = vec3(1.0);
    #ifdef SCREEN_SPACE_AMBIENT_OCCLUSION
        let ssao = textureLoad(screen_space_ambient_occlusion_texture, vec2<i32>(in.position.xy), 0i).r;
        let ssao_multibounce = gtao_multibounce(ssao, pbr_input.material.base_color.rgb);
        occlusion = min(occlusion, ssao_multibounce);

    #endif
    pbr_input.occlusion = occlusion;
    
    //frag_coord
    pbr_input.frag_coord = in.position;

    //world_pos
    pbr_input.world_position = in.world_position;

    //world_normals
    pbr_input.world_normal = pbr_functions::prepare_world_normal(
        in.world_normal,
        false,
        is_front,
    );

    //ortho
    pbr_input.is_orthographic = is_orthographic;

    //normal todo!()
    #ifdef LOAD_PREPASS_NORMALS
        pbr_input.N = bevy_pbr::prepass_utils::prepass_normal(in.position, 0u);
    #else
        pbr_input.N = apply_normal_mapping(
            pbr_input.world_normal,
            #ifdef VERTEX_TANGENTS
                in.world_tangent,
            #endif
            normal
        );
    #endif

    //world veiw
    pbr_input.V = V;


    //flag    
    pbr_input.flags = mesh.flags;

    output_color = pbr_functions::pbr(pbr_input);


    // fog
    // if (fog.mode != FOG_MODE_OFF && (pbr_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT) != 0u) {
    //     output_color = pbr_functions::apply_fog(fog, output_color, in.world_position.xyz, view.world_position.xyz);
    // }

    #ifdef TONEMAP_IN_SHADER
        output_color = tone_mapping(output_color, view.color_grading);
        #ifdef DEBAND_DITHER
            var output_rgb = output_color.rgb;
            output_rgb = powsafe(output_rgb, 1.0 / 2.2);
            output_rgb = output_rgb + screen_space_dither(in.position.xy);
            // This conversion back to linear space is required because our output texture format is
            // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
            output_rgb = powsafe(output_rgb, 2.2);
            output_color = vec4(output_rgb, output_color.a);
        #endif
    #endif
    #ifdef PREMULTIPLY_ALPHA
        output_color = pbr_functions::premultiply_alpha(pbr_bindings::material.flags, output_color);
    #endif
    return output_color;
}