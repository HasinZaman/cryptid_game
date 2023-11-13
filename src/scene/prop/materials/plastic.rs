use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey, PBR_PREPASS_SHADER_HANDLE},
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

use super::{Directions, Position};

/// A material with "standard" properties used in PBR lighting
/// Standard property values with pictures here
/// <https://google.github.io/filament/Material%20Properties.pdf>.
///
/// May be created directly from a [`Color`] or an [`Image`].
#[derive(AsBindGroup, Reflect, Debug, Clone, TypeUuid)]
#[uuid = "e65799f2-923e-4548-8879-be574f9dc989"]
#[bind_group_data(PlasticMaterialKey)]
#[uniform(0, PlasticMaterialUniform)]
#[reflect(Default, Debug)]
pub struct PlasticMaterial {
    pub colour: Color,
    pub metallic: f32,
    pub scale_1: Vec2,
    pub offset_1: Vec2,
    pub scale_2: Vec2,
    pub offset_2: Vec2,

    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,

    pub position: Vec3,
    #[texture(1)]
    pub noise_texture_1: Option<Handle<Image>>,
    #[texture(2)]
    pub noise_texture_2: Option<Handle<Image>>,
    #[texture(4)]
    #[sampler(5)]
    pub depth_map: Option<Handle<Image>>,
}

impl Default for PlasticMaterial {
    fn default() -> Self {
        PlasticMaterial {
            colour: Color::Rgba {
                red: 1.,
                green: 1.,
                blue: 1.,
                alpha: 1.,
            },
            metallic: 0.1,
            scale_1: Vec2 { x: 1., y: 1. },
            offset_1: Vec2 { x: 0., y: 0. },
            scale_2: Vec2 { x: 1., y: 1. },
            offset_2: Vec2 { x: 0., y: 0. },

            forward: Vec3::X,
            right: Vec3::Z,
            up: Vec3::Y,

            position: Vec3::ZERO,
            noise_texture_1: None,
            noise_texture_2: None,
            depth_map: None,
        }
    }
}

impl From<Color> for PlasticMaterial {
    fn from(colour: Color) -> Self {
        PlasticMaterial {
            colour: colour,
            ..Default::default()
        }
    }
}

impl From<Handle<Image>> for PlasticMaterial {
    fn from(texture: Handle<Image>) -> Self {
        PlasticMaterial {
            noise_texture_1: Some(texture),
            ..Default::default()
        }
    }
}

#[derive(Clone, Default, ShaderType)]
pub struct PlasticMaterialUniform {
    scale_1: Vec2,
    offset_1: Vec2,
    scale_2: Vec2,
    offset_2: Vec2,

    colour: Vec4,
    metallic: f32,

    forward: Vec3,
    right: Vec3,
    up: Vec3,

    position: Vec3,
}

impl AsBindGroupShaderType<PlasticMaterialUniform> for PlasticMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> PlasticMaterialUniform {
        PlasticMaterialUniform {
            scale_1: self.scale_1,
            offset_1: self.offset_1,
            scale_2: self.scale_2,
            offset_2: self.offset_2,

            colour: self.colour.as_linear_rgba_f32().into(),
            metallic: self.metallic,

            forward: self.forward,
            right: self.right,
            up: self.up,

            position: self.position,
        }
    }
}

/// The pipeline key for [`StandardMaterial`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PlasticMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
    depth_bias: i32,
    relief_mapping: bool,
}

impl From<&PlasticMaterial> for PlasticMaterialKey {
    fn from(_material: &PlasticMaterial) -> Self {
        PlasticMaterialKey {
            normal_map: false,
            cull_mode: Some(Face::Back),
            depth_bias: 0.0 as i32,
            relief_mapping: matches!(
                ParallaxMappingMethod::Occlusion,
                ParallaxMappingMethod::Relief { .. }
            ),
        }
    }
}

impl Material for PlasticMaterial {
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
        "shaders/plastic.wgsl".into()
    }

    #[inline]
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    #[inline]
    fn depth_bias(&self) -> f32 {
        0.0
    }
}

impl Directions for PlasticMaterial {
    fn set_direction(&mut self, forward: Vec3, right: Vec3, up: Vec3) {
        self.forward = forward;
        self.right = right;
        self.up = up;
    }
    fn get_direction(&self) -> (&Vec3, &Vec3, &Vec3) {
        return (&self.forward, &self.right, &self.up);
    }
}
impl Position for PlasticMaterial {
    fn set_position(&mut self, new_position: Vec3) {
        self.position = new_position;
    }

    fn get_position(&self) -> &Vec3 {
        return &self.position;
    }
}
