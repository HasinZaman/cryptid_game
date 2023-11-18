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

fn sample_noise(
    coord: vec2<f32>,
    scale: vec2<f32>,
    offset: vec2<f32>,
    texture: texture_2d<f32>
) -> f32 {
    let dim: vec2<u32> = textureDimensions(texture);
    
    let texel_coord = vec2<u32> (
        u32((scale.x * coord.x + offset.x) * f32(dim.x)) % dim.x,
        u32((scale.y * coord.y + offset.y) * f32(dim.y)) % dim.y
    );

    return textureLoad(texture, vec2<u32>(texel_coord), 0).r;
}

fn ramp(x: f32) -> f32 {
    let m: f32 = 1. / (1. - 0.215);
    let b: f32 = 1. - m;
    return max(x*m+b, 0.);
}

fn overlay(a: f32, b: f32) -> f32 {
    if (a < 0.5) {
        return 2. * a * b;
    }
    return 1. - 2. * (1.-a) * (1.-b);
}

fn bump(
    coord: vec2<f32>, 
    scale: vec2<f32>,
    offset: vec2<f32>,
    texture: texture_2d<f32>,
) -> vec3<f32> {
    let o: vec3<f32> = vec3<f32>(1., 1., 0.);

    let fx0: f32 = sample_noise(coord-o.xz, scale, offset, texture);
    let fx1: f32 = sample_noise(coord+o.xz, scale, offset, texture);

    let fy0: f32 = sample_noise(coord-o.zy, scale, offset, texture);
    let fy1: f32 = sample_noise(coord+o.zy, scale, offset, texture);

    let eps: f32 = 1.;

    return normalize(
        vec3<f32>( (fx0-fx1)/(2.*eps), (fy0-fy1)/(2.*eps), 1.)
    );
}

fn projectVectorMagnitude(a: vec3<f32>, b: vec3<f32>) -> f32 {
    let v = (dot(a, b) / dot(b, b)) * b;
    return sqrt(dot(v,v));
}
fn get_voxel_position(in: MeshVertexOutput, voxel_size: f32) -> vec3<i32> {
    var pos = vec3<f32> (
        (in.world_position.x - properties.position.x),
        (in.world_position.y - properties.position.y),
        (in.world_position.z - properties.position.z)
    );
    let local_pos = vec3<f32> (
        projectVectorMagnitude(pos, properties.forward),
        projectVectorMagnitude(pos, properties.up),
        projectVectorMagnitude(pos, properties.right)
    );

    return vec3<i32> (
        i32(local_pos.x / voxel_size),
        i32(local_pos.y / voxel_size),
        i32(local_pos.z / voxel_size)
    );
}

struct PlasticProperties {
    scale_1: vec2<f32>,
    offset_1: vec2<f32>,
    scale_2: vec2<f32>,
    offset_2: vec2<f32>,
    colour: vec4<f32>,
    metallic: f32,

    forward: vec3<f32>,
    right: vec3<f32>,
    up: vec3<f32>,

    position: vec4<f32>,
}
@group(1) @binding(0)
var<uniform> properties: PlasticProperties;
@group(1) @binding(1)
var noise_texture_1: texture_2d<f32>;
@group(1) @binding(2)
var noise_texture_2: texture_2d<f32>;//if not defined use noise_texture_1
@group(1) @binding(3)
var depth_map_texture: texture_2d<f32>;
@group(1) @binding(4)
var depth_map_sampler: sampler;

@fragment
fn fragment(
    in: MeshVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    
    let is_orthographic = view.projection[3].w == 1.0;
    let V = pbr_functions::calculate_view(in.world_position, is_orthographic);

    //var uv: vec2<f32> = in.uv;
    let voxel = get_voxel_position(in, 1.0/32.);//1.0 / 16.);

    //convert voxel into uv
    let layer_offset: i32 = 547;
    var uv = vec2<f32> (
        f32(voxel.x) / 500. % 500.0,
        f32(voxel.z + voxel.y * layer_offset) / 500. % 500.0,
    );

    var output_color: vec4<f32> = properties.colour;
    var metallic: f32 = properties.metallic;
    
    let normal: vec3<f32> = bump(uv, properties.scale_2, properties.offset_2, noise_texture_1);//textureLoad(normal_texture, vec2<u32>(0u), 0).rgb;
    var roughness: f32 = overlay(ramp(sample_noise(uv, properties.scale_1, properties.offset_1, noise_texture_1)), 0.2);
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