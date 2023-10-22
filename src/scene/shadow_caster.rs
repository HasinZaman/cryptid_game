use bevy::{
    pbr::{
        MaterialPipeline, MaterialPipelineKey, StandardMaterialFlags, PBR_PREPASS_SHADER_HANDLE,
    },
    prelude::*,
    reflect::{Reflect, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, AsBindGroupShaderType, Face, RenderPipelineDescriptor, ShaderRef,
            ShaderType, SpecializedMeshPipelineError,
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
#[uniform(0, ShadowCasterMaterialUniform)]
#[reflect(Default, Debug)]
pub struct ShadowCasterMaterial {
    // pub base_color: Color,
    // #[texture(1)]
    // #[sampler(2)]
    // pub base_color_texture: Option<Handle<Image>>,
    // pub emissive: Color,
    // #[texture(3)]
    // #[sampler(4)]
    // pub emissive_texture: Option<Handle<Image>>,
    // pub perceptual_roughness: f32,
    // pub metallic: f32,
    // #[texture(5)]
    // #[sampler(6)]
    // pub metallic_roughness_texture: Option<Handle<Image>>,
    // #[doc(alias = "specular_intensity")]
    // pub reflectance: f32,
    // #[texture(9)]
    // #[sampler(10)]
    // pub normal_map_texture: Option<Handle<Image>>,
    // pub flip_normal_map_y: bool,
    // #[texture(7)]
    // #[sampler(8)]
    // pub occlusion_texture: Option<Handle<Image>>,
    // pub double_sided: bool,
    #[reflect(ignore)]
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub fog_enabled: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,
    // #[texture(11)]
    // #[sampler(12)]
    // pub depth_map: Option<Handle<Image>>,
    pub parallax_depth_scale: f32,
    pub parallax_mapping_method: ParallaxMappingMethod,
    pub max_parallax_layer_count: f32,
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
            // depth_map: None,
            parallax_depth_scale: 0.1,
            max_parallax_layer_count: 16.0,
            parallax_mapping_method: ParallaxMappingMethod::Occlusion,
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

#[derive(Clone, Default, ShaderType)]
pub struct ShadowCasterMaterialUniform {
    pub base_color: Vec4,
    pub emissive: Vec4,
    pub roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
    pub flags: u32,
    pub alpha_cutoff: f32,
    pub parallax_depth_scale: f32,
    pub max_parallax_layer_count: f32,
    pub max_relief_mapping_search_steps: u32,
}

impl AsBindGroupShaderType<ShadowCasterMaterialUniform> for ShadowCasterMaterial {
    fn as_bind_group_shader_type(
        &self,
        _images: &RenderAssets<Image>,
    ) -> ShadowCasterMaterialUniform {
        let mut flags = StandardMaterialFlags::NONE;
        // if self.base_color_texture.is_some() {
        //     flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        // }
        // if self.emissive_texture.is_some() {
        //     flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
        // }
        // if self.metallic_roughness_texture.is_some() {
        //     flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        // }
        // if self.occlusion_texture.is_some() {
        //     flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
        // }
        // if self.double_sided {
        //     flags |= StandardMaterialFlags::DOUBLE_SIDED;
        // }
        if self.unlit {
            flags |= StandardMaterialFlags::UNLIT;
        }
        if self.fog_enabled {
            flags |= StandardMaterialFlags::FOG_ENABLED;
        }
        // if self.depth_map.is_some() {
        //     flags |= StandardMaterialFlags::DEPTH_MAP;
        // }
        // let has_normal_map = self.normal_map_texture.is_some();
        // if has_normal_map {
        //     if let Some(texture) = images.get(self.normal_map_texture.as_ref().unwrap()) {
        //         match texture.texture_format {
        //             // All 2-component unorm formats
        //             TextureFormat::Rg8Unorm
        //             | TextureFormat::Rg16Unorm
        //             | TextureFormat::Bc5RgUnorm
        //             | TextureFormat::EacRg11Unorm => {
        //                 flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP;
        //             }
        //             _ => {}
        //         }
        //     }
        //     if self.flip_normal_map_y {
        //         flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;
        //     }
        // }
        // NOTE: 0.5 is from the glTF default - do we want this?
        let mut alpha_cutoff = 0.5;
        match self.alpha_mode {
            AlphaMode::Opaque => flags |= StandardMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(c) => {
                alpha_cutoff = c;
                flags |= StandardMaterialFlags::ALPHA_MODE_MASK;
            }
            AlphaMode::Blend => flags |= StandardMaterialFlags::ALPHA_MODE_BLEND,
            AlphaMode::Premultiplied => flags |= StandardMaterialFlags::ALPHA_MODE_PREMULTIPLIED,
            AlphaMode::Add => flags |= StandardMaterialFlags::ALPHA_MODE_ADD,
            AlphaMode::Multiply => flags |= StandardMaterialFlags::ALPHA_MODE_MULTIPLY,
        };

        ShadowCasterMaterialUniform {
            base_color: Vec4::ZERO, //self.base_color.as_linear_rgba_f32().into(),
            emissive: Vec4::ZERO,   //self.emissive.as_linear_rgba_f32().into(),
            roughness: 0.,          //self.perceptual_roughness,
            metallic: 0.,           //self.metallic,
            reflectance: 0.,        //self.reflectance,
            flags: flags.bits(),
            alpha_cutoff,
            parallax_depth_scale: self.parallax_depth_scale,
            max_parallax_layer_count: self.max_parallax_layer_count,
            max_relief_mapping_search_steps: parallax_mapping_method_max_steps(
                self.parallax_mapping_method,
            ),
        }
    }
}

pub fn parallax_mapping_method_max_steps(p: ParallaxMappingMethod) -> u32 {
    match p {
        ParallaxMappingMethod::Occlusion => 0,
        ParallaxMappingMethod::Relief { max_steps } => max_steps,
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
                material.parallax_mapping_method,
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
