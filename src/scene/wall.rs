use std::collections::HashMap;

use bevy::{
    pbr::{
        MaterialPipeline, MaterialPipelineKey, StandardMaterialFlags, PBR_PREPASS_SHADER_HANDLE,
    },
    prelude::{
        AlphaMode, App, AssetServer, Assets, Camera, Handle, Image, Material, MaterialMeshBundle,
        MaterialPlugin, Mesh, ParallaxMappingMethod, Plugin, PostUpdate, PreStartup, Query,
        ReflectDefault, ResMut, Resource, Transform, Vec4, Visibility, With,
    },
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

#[derive(Resource)]
pub struct Walls(pub HashMap<String, Wall>);

pub struct Wall {
    pub mesh: Handle<Mesh>,
    pub material: WallMaterial,
}

pub fn into_mesh_bundle(
    wall: &Wall,
    materials: &mut ResMut<Assets<WallMaterial>>,
    transform: Option<Transform>,
) -> MaterialMeshBundle<WallMaterial> {
    match transform {
        Some(t) => MaterialMeshBundle {
            mesh: wall.mesh.clone(),
            material: materials.add(wall.material.clone()),
            transform: t,
            ..Default::default()
        },
        None => MaterialMeshBundle {
            mesh: wall.mesh.clone(),
            material: materials.add(wall.material.clone()),
            ..Default::default()
        },
    }
}

fn load_wall(asset_server: &ResMut<AssetServer>, dir: &'static str) -> Wall {
    Wall {
        mesh: asset_server.load(format!("scenes/{dir}/mesh/mesh.glb#Mesh0/Primitive0")),
        material: WallMaterial {
            normal_map_texture: Some(
                asset_server.load(format!("scenes/{dir}/textures/normal.png")),
            ),
            ..Default::default()
        }, /*
           material: FloorMaterial{
               base_color_texture: Some(
                   asset_server.load(
                       format!("scenes/{dir}/textures/colour.png")
                   )
               ),
               metallic_texture: Some(
                   asset_server.load(
                       format!("scenes/{dir}/textures/metallic.png")
                   )
               ),
               normal_map_texture: Some(
                   asset_server.load(
                       format!("scenes/{dir}/textures/normal.png")
                   )
               ),
               texture_map: Some(
                   asset_server.load(
                       format!("scenes/{dir}/textures/texture_map.png")
                   )
               ),
               ..FloorMaterial::default()
           },
           */
    }
}

pub fn load_walls(mut walls: ResMut<Walls>, asset_server: ResMut<AssetServer>) {
    walls.as_mut().0.insert(
        "dev_playground/wall_1".into(),
        load_wall(&asset_server, "dev_playground/wall_1"),
    );
    walls.as_mut().0.insert(
        "dev_playground/wall_2".into(),
        load_wall(&asset_server, "dev_playground/wall_2"),
    );
    walls.as_mut().0.insert(
        "dev_playground/wall_3".into(),
        load_wall(&asset_server, "dev_playground/wall_3"),
    );
    walls.as_mut().0.insert(
        "dev_playground/wall_4".into(),
        load_wall(&asset_server, "dev_playground/wall_4"),
    );
}

pub fn render_wall(
    camera_query: Query<&Transform, With<Camera>>,
    mut wall_query: Query<(&mut Visibility, &Transform), With<Handle<WallMaterial>>>,
) {
    // get camera dir vector
    let camera_iter = camera_query.iter().next();
    if let Some(camera_transform) = camera_iter {
        let mut camera_dir = camera_transform.forward();
        camera_dir.y = 0.;
        let camera_dir = camera_dir.normalize();

        for (mut visibility, wall_transform) in &mut wall_query {
            // get dir vector
            let mut wall_dir = wall_transform.left();
            wall_dir.y = 0.;
            let wall_dir = wall_dir.normalize();

            // update wall rendering if wall is pointing towards camera else do not render
            *visibility = match camera_dir.dot(wall_dir) <= 0. {
                true => Visibility::Hidden,
                false => Visibility::Visible,
            }
        }
    }
}

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        let walls = Walls(HashMap::new());

        app.insert_resource(walls)
            .add_plugins((MaterialPlugin::<WallMaterial>::default(),))
            .add_systems(PreStartup, load_walls)
            .add_systems(PostUpdate, render_wall);
    }
}

#[derive(AsBindGroup, Reflect, Debug, Clone, TypeUuid)]
#[uuid = "e65799f2-923e-4548-8879-be574f9db998"]
#[bind_group_data(WallMaterialKey)]
#[uniform(0, StandardMaterialUniform)]
#[reflect(Default, Debug)]
pub struct WallMaterial {
    // #[texture(1)]
    // pub base_color_texture: Option<Handle<Image>>,
    // #[texture(2)]
    // pub metallic_texture: Option<Handle<Image>>,
    #[texture(3)]
    pub normal_map_texture: Option<Handle<Image>>,
    // #[texture(4)]
    // pub texture_map: Option<Handle<Image>>,
    //pub cell_size: f32,
    #[reflect(ignore)]
    pub cull_mode: Option<Face>,
    pub unlit: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,
    pub parallax_mapping_method: ParallaxMappingMethod,
}

impl Material for WallMaterial {
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
        "shaders/wall.wgsl".into()
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

impl Default for WallMaterial {
    fn default() -> Self {
        WallMaterial {
            // base_color_texture: None,
            // metallic_texture: None,
            normal_map_texture: None,
            // texture_map: None,
            //cell_size: 1.,
            cull_mode: Some(Face::Back),
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            parallax_mapping_method: ParallaxMappingMethod::Occlusion,
        }
    }
}

impl From<Handle<Image>> for WallMaterial {
    fn from(_texture: Handle<Image>) -> Self {
        WallMaterial {
            // base_color_texture: Some(texture),
            ..Default::default()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WallMaterialKey {
    normal_map: bool,
    cull_mode: Option<Face>,
    depth_bias: i32,
    relief_mapping: bool,
}

impl From<&WallMaterial> for WallMaterialKey {
    fn from(material: &WallMaterial) -> Self {
        WallMaterialKey {
            normal_map: false, // material.normal_map_texture.is_some(),
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

impl AsBindGroupShaderType<StandardMaterialUniform> for WallMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> StandardMaterialUniform {
        let mut flags = StandardMaterialFlags::NONE;
        // if self.base_color_texture.is_some() {
        //     flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        // }
        // if self.emissive_texture.is_some() {
        //     flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
        // }
        // if self.metallic_texture.is_some() {
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
        // if self.fog_enabled {
        //     flags |= StandardMaterialFlags::FOG_ENABLED;
        // }
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
        //     // if self.flip_normal_map_y {
        //     //     flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;
        //     // }
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
