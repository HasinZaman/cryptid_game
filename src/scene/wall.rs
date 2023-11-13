use std::collections::HashMap;

use bevy::{
    pbr::{
        MaterialPipeline, MaterialPipelineKey, PBR_PREPASS_SHADER_HANDLE,
    },
    prelude::{
        AlphaMode, App, AssetServer, Assets, Camera, Handle, Image, Material, MaterialMeshBundle,
        MaterialPlugin, Mesh, ParallaxMappingMethod, Plugin, PostUpdate, PreStartup, Query,
        ReflectDefault, ResMut, Resource, Transform, Visibility, With,
    },
    reflect::{Reflect, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, Face, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
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
        },
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
#[reflect(Default, Debug)]
pub struct WallMaterial {
    //required properties
    //- Direction vectors
    //- texture vectors
    #[texture(3)]
    pub normal_map_texture: Option<Handle<Image>>,
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
            normal_map_texture: None,
            cull_mode: Some(Face::Back),
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            parallax_mapping_method: ParallaxMappingMethod::Occlusion,
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