#define_import_path bevy_pbr::fragment

#import bevy_pbr::pbr_functions as pbr_functions
#import bevy_pbr::pbr_bindings as pbr_bindings
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

@fragment
fn fragment(
    in: MeshVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let is_orthographic = view.projection[3].w == 1.0;
    let V = pbr_functions::calculate_view(in.world_position, is_orthographic);
    
    let colour = vec4<f32>(0.);

    let metallic = 0.;
    
    let roughness = 0.;

    var output_color: vec4<f32> = colour;

    var pbr_input: pbr_functions::PbrInput;
    
    //pbr material
    pbr_input.material.base_color = output_color;
    pbr_input.material.emissive = vec4<f32>(0.0);
    pbr_input.material.perceptual_roughness = roughness;
    pbr_input.material.metallic = metallic;

    pbr_input.material.flags = 536870912u;//pbr_functions::STANDARD_MATERIAL_FLAGS_APHA_MODE_MASK;
    pbr_input.material.alpha_cutoff = 1.0;
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
        pbr_input.N = pbr_functions::apply_normal_mapping(
            pbr_bindings::material.flags,
            pbr_input.world_normal,
            #ifdef VERTEX_TANGENTS
            #ifdef STANDARDMATERIAL_NORMAL_MAP
                in.world_tangent,
            #endif
            #endif
            #ifdef VERTEX_UVS
                uv,
            #endif
                view.mip_bias,
        );
    #endif
        //pbr_input.N = bevy_pbr::prepass_utils::prepass_normal(in.position, 0u);
        // pbr_input.N = apply_normal_mapping(
        //     pbr_input.world_normal,
        //     #ifdef VERTEX_TANGENTS
        //         in.world_tangent,
        //     #endif
        //     normal
        // );
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

    // #ifdef TONEMAP_IN_SHADER
    //     output_color = tone_mapping(output_color, view.color_grading);
    //     #ifdef DEBAND_DITHER
    //         var output_rgb = output_color.rgb;
    //         output_rgb = powsafe(output_rgb, 1.0 / 2.2);
    //         output_rgb = output_rgb + screen_space_dither(in.position.xy);
    //         // This conversion back to linear space is required because our output texture format is
    //         // SRGB; the GPU will assume our output is linear and will apply an SRGB conversion.
    //         output_rgb = powsafe(output_rgb, 2.2);
    //         output_color = vec4(output_rgb, output_color.a);
    //     #endif
    // #endif
    // #ifdef PREMULTIPLY_ALPHA
    //     output_color = pbr_functions::premultiply_alpha(pbr_bindings::material.flags, output_color);
    // #endif
    return output_color;
}