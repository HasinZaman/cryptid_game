use std::collections::HashMap;

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
            ShaderType, SpecializedMeshPipelineError, TextureFormat,
        },
    },
};

#[derive(Resource)]
pub struct Floors(pub HashMap<String, Floor>);

pub struct Floor {
    pub mesh: Handle<Mesh>,
    pub material: FloorMaterial,
}

pub fn into_mesh_bundle(
    floor: &Floor,
    materials: &mut ResMut<Assets<FloorMaterial>>,
    transform: Option<Transform>,
) -> MaterialMeshBundle<FloorMaterial> {
    match transform {
        Some(t) => MaterialMeshBundle {
            mesh: floor.mesh.clone(),
            material: materials.add(floor.material.clone()),
            transform: t,
            ..default()
        },
        None => MaterialMeshBundle {
            mesh: floor.mesh.clone(),
            material: materials.add(floor.material.clone()),
            ..default()
        },
    }
}

fn load_floor(asset_server: &ResMut<AssetServer>, dir: &'static str) -> Floor {
    Floor {
        mesh: asset_server.load(format!("scenes/{dir}/mesh/mesh.glb#Mesh0/Primitive0")),
        material: FloorMaterial {
            base_color_texture: Some(
                asset_server.load(format!("scenes/{dir}/textures/colour.png")),
            ),
            metallic_texture: Some(
                asset_server.load(format!("scenes/{dir}/textures/metallic.png")),
            ),
            normal_map_texture: Some(
                asset_server.load(format!("scenes/{dir}/textures/normal.png")),
            ),
            texture_map: Some(asset_server.load(format!("scenes/{dir}/textures/texture_map.png"))),
            ..FloorMaterial::default()
        },
    }
}

pub fn load_floors(mut floors: ResMut<Floors>, asset_server: ResMut<AssetServer>) {
    floors.as_mut().0.insert(
        "dev_playground/metal_grate_floor".into(),
        load_floor(&asset_server, "dev_playground/metal_grate_floor"),
    );
    floors.as_mut().0.insert(
        "dev_playground/stainless_steel_floor".into(),
        load_floor(&asset_server, "dev_playground/stainless_steel_floor"),
    );
}

pub struct FloorPlugin;

impl Plugin for FloorPlugin {
    fn build(&self, app: &mut App) {
        let floors = Floors(HashMap::new());

        app.insert_resource(floors)
            .add_plugins((MaterialPlugin::<FloorMaterial>::default(),))
            .add_systems(PreStartup, load_floors);
    }
}

#[derive(AsBindGroup, Reflect, Debug, Clone, TypeUuid)]
#[uuid = "e65799f2-923e-4548-8879-be574f9db999"]
#[bind_group_data(FloorMaterialKey)]
#[uniform(0, StandardMaterialUniform)]
#[reflect(Default, Debug)]
pub struct FloorMaterial {
    #[texture(1)]
    pub base_color_texture: Option<Handle<Image>>,
    #[texture(2)]
    pub metallic_texture: Option<Handle<Image>>,
    #[texture(3)]
    pub normal_map_texture: Option<Handle<Image>>,
    #[texture(4)]
    pub texture_map: Option<Handle<Image>>,
    //pub cell_size: f32,
    #[reflect(ignore)]
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,
    pub parallax_mapping_method: ParallaxMappingMethod,
}

impl Material for FloorMaterial {
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
        "shaders/floor.wgsl".into()
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

impl Default for FloorMaterial {
    fn default() -> Self {
        FloorMaterial {
            base_color_texture: None,
            metallic_texture: None,
            normal_map_texture: None,
            texture_map: None,
            //cell_size: 1.,
            cull_mode: Some(Face::Back),
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            parallax_mapping_method: ParallaxMappingMethod::Occlusion,
        }
    }
}

impl From<Handle<Image>> for FloorMaterial {
    fn from(texture: Handle<Image>) -> Self {
        FloorMaterial {
            base_color_texture: Some(texture),
            ..Default::default()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FloorMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
    depth_bias: i32,
    relief_mapping: bool,
}

impl From<&FloorMaterial> for FloorMaterialKey {
    fn from(material: &FloorMaterial) -> Self {
        FloorMaterialKey {
            normal_map: material.normal_map_texture.is_some(),
            cull_mode: material.cull_mode,
            depth_bias: material.depth_bias as i32,
            relief_mapping: matches!(
                material.parallax_mapping_method,
                ParallaxMappingMethod::Relief { .. }
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

#[derive(Clone, Default, ShaderType)]
pub struct StandardMaterialUniform {
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

impl AsBindGroupShaderType<StandardMaterialUniform> for FloorMaterial {
    fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> StandardMaterialUniform {
        let mut flags = StandardMaterialFlags::NONE;
        if self.base_color_texture.is_some() {
            flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        }
        // if self.emissive_texture.is_some() {
        //     flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
        // }
        if self.metallic_texture.is_some() {
            flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        }
        // if self.occlusion_texture.is_some() {
        //     flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
        // }
        // if self.double_sided {
        //     flags |= StandardMaterialFlags::DOUBLE_SIDED;
        // }
        if self.unlit {
            flags |= StandardMaterialFlags::UNLIT;
        }
        // if self.fog_enabled {
        //     flags |= StandardMaterialFlags::FOG_ENABLED;
        // }
        // if self.depth_map.is_some() {
        //     flags |= StandardMaterialFlags::DEPTH_MAP;
        // }
        let has_normal_map = self.normal_map_texture.is_some();
        if has_normal_map {
            if let Some(texture) = images.get(self.normal_map_texture.as_ref().unwrap()) {
                match texture.texture_format {
                    // All 2-component unorm formats
                    TextureFormat::Rg8Unorm
                    | TextureFormat::Rg16Unorm
                    | TextureFormat::Bc5RgUnorm
                    | TextureFormat::EacRg11Unorm => {
                        flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP;
                    }
                    _ => {}
                }
            }
            // if self.flip_normal_map_y {
            //     flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;
            // }
        }
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

        StandardMaterialUniform {
            base_color: Vec4::ZERO, //self.base_color.as_linear_rgba_f32().into(),
            emissive: Vec4::ZERO,   //self.emissive.as_linear_rgba_f32().into(),
            roughness: 0.,          //self.perceptual_roughness,
            metallic: 0.,           //self.metallic,
            reflectance: 0.,        //self.reflectance,
            flags: flags.bits(),
            alpha_cutoff,
            parallax_depth_scale: 0.,     //self.parallax_depth_scale,
            max_parallax_layer_count: 0., //self.max_parallax_layer_count,
            max_relief_mapping_search_steps: parallax_mapping_method_max_steps(
                self.parallax_mapping_method,
            ),
        }
    }
}