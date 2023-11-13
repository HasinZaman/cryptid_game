use bevy::{
    pbr::{
        MaterialPipeline, MaterialPipelineKey, PBR_PREPASS_SHADER_HANDLE,
    },
    prelude::*,
    reflect::{Reflect, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, Face, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

/// A material with "standard" properties used in PBR lighting
/// Standard property values with pictures here
/// <https://google.github.io/filament/Material%20Properties.pdf>.
///
/// May be created directly from a [`Color`] or an [`Image`].
#[derive(AsBindGroup, Reflect, Debug, Clone, TypeUuid)]
#[uuid = "e65799f2-923e-4548-8879-be574f9db996"]
#[bind_group_data(ShadowCasterMaterialKey)]
#[reflect(Default, Debug)]
pub struct ShadowCasterMaterial {
    
    #[reflect(ignore)]
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub fog_enabled: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,
}

impl Default for ShadowCasterMaterial {
    fn default() -> Self {
        ShadowCasterMaterial {
            // White because it gets multiplied with texture values if someone uses
            // a texture.
            // base_color: Color::rgb(1.0, 1.0, 1.0),
            // base_color_texture: None,
            // emissive: Color::BLACK,
            // emissive_texture: None,
            // Matches Blender's default roughness.
            // perceptual_roughness: 0.5,
            // Metallic should generally be set to 0.0 or 1.0.
            // metallic: 0.0,
            // metallic_roughness_texture: None,
            // Minimum real-world reflectance is 2%, most materials between 2-5%
            // Expressed in a linear scale and equivalent to 4% reflectance see
            // <https://google.github.io/filament/Material%20Properties.pdf>
            // reflectance: 0.5,
            // occlusion_texture: None,
            // normal_map_texture: None,
            // flip_normal_map_y: false,
            // double_sided: false,
            cull_mode: Some(Face::Back),
            unlit: false,
            fog_enabled: true,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
        }
    }
}

impl From<Color> for ShadowCasterMaterial {
    fn from(color: Color) -> Self {
        ShadowCasterMaterial {
            //base_color: Vec4::ZERO,
            alpha_mode: if color.a() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..Default::default()
        }
    }
}

impl From<Handle<Image>> for ShadowCasterMaterial {
    fn from(_texture: Handle<Image>) -> Self {
        ShadowCasterMaterial {
            //base_color_texture: Some(texture),
            ..Default::default()
        }
    }
}

/// The pipeline key for [`ShadowCasterMaterial`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ShadowCasterMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
    depth_bias: i32,
    relief_mapping: bool,
}

impl From<&ShadowCasterMaterial> for ShadowCasterMaterialKey {
    fn from(material: &ShadowCasterMaterial) -> Self {
        ShadowCasterMaterialKey {
            normal_map: false, //material.normal_map_texture.is_some(),
            cull_mode: material.cull_mode,
            depth_bias: material.depth_bias as i32,
            relief_mapping: matches!(
                ParallaxMappingMethod::Occlusion,
                ParallaxMappingMethod::Relief { .. }
            ),
        }
    }
}

impl Material for ShadowCasterMaterial {
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            let shader_defs = &mut fragment.shader_defs;

            if key.bind_group_data.normal_map {
                shader_defs.push("STANDARDMATERIAL_NORMAL_MAP".into());
            }
            if key.bind_group_data.relief_mapping {
                shader_defs.push("RELIEF_MAPPING".into());
            }
        }
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        if let Some(label) = &mut descriptor.label {
            *label = format!("pbr_{}", *label).into();
        }
        if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
            depth_stencil.bias.constant = key.bind_group_data.depth_bias;
        }
        Ok(())
    }

    fn prepass_fragment_shader() -> ShaderRef {
        PBR_PREPASS_SHADER_HANDLE.typed().into()
    }

    fn fragment_shader() -> ShaderRef {
        // let tmp: ShaderRef = "NOT A REAL SHADER".into();
        // println!("{}", tmp);
        "NOT A REAL SHADER".into()
    }

    #[inline]
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    #[inline]
    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
}
