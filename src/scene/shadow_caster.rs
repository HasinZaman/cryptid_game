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

#[derive(AsBindGroup, Reflect, Debug, Clone, TypeUuid)]
#[uuid = "e65799f2-923e-4548-8879-be574f9db996"]
#[bind_group_data(ShadowCasterMaterialKey)]
#[reflect(Default, Debug)]
pub struct ShadowCasterMaterial {
    #[reflect(ignore)]
    pub cull_mode: Option<Face>,
    pub depth_bias: f32,
}

impl Default for ShadowCasterMaterial {
    fn default() -> Self {
        ShadowCasterMaterial {
            cull_mode: Some(Face::Back),
            depth_bias: 0.0,
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
            cull_mode: Some(Face::Back),
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
        AlphaMode::Opaque
    }

    #[inline]
    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }
}
