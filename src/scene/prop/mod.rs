use bevy::{utils::HashMap, prelude::{Resource, Handle, Mesh, ResMut, Assets, Transform, MaterialMeshBundle, default, AssetServer, Plugin, App, PreStartup, Material, Color}, reflect::TypeUuid, asset::Asset};

use self::materials::{plastic::PlasticMaterial, MaterialsPlugin};

pub mod sound_source;
pub mod materials;

#[derive(Resource)]
pub struct Props<M>(pub HashMap<String, Prop<M>>);

pub struct Prop<M> {
    pub mesh: Handle<Mesh>,
    pub material: M,
}

pub fn into_mesh_bundle<M: TypeUuid + Asset + Material>(
    prop: &Prop<M>,
    materials: &mut ResMut<Assets<M>>,
    transform: Option<Transform>,
) -> MaterialMeshBundle<M> {
    match transform {
        Some(t) => MaterialMeshBundle {
            mesh: prop.mesh.clone(),
            material: materials.add(prop.material.clone()),
            transform: t,
            ..default()
        },
        None => MaterialMeshBundle {
            mesh: prop.mesh.clone(),
            material: materials.add(prop.material.clone()),
            ..default()
        },
    }
}

fn load_plastic_prop(asset_server: &ResMut<AssetServer>, dir: &'static str) -> Prop<PlasticMaterial> {
    Prop {
        mesh: asset_server.load(format!("props/{dir}/mesh/mesh.glb#Mesh0/Primitive0")),
        material: PlasticMaterial {
            colour: Color::rgba(0.8, 0.3, 0., 1.),
            noise_texture_1: Some(asset_server.load(format!("props/{dir}/textures/noise.png"))),
            ..PlasticMaterial::default()
        },
    }
}

pub fn load_plastic_props(mut props: ResMut<Props<PlasticMaterial>>, asset_server: ResMut<AssetServer>) {
    props.as_mut().0.insert(
        "plastic_bin_1".into(),
        load_plastic_prop(&asset_server, "plastic_bin_1"),
    );
}

pub struct PropPlugin;

impl Plugin for PropPlugin {
    fn build(&self, app: &mut App) {
        let plastic_props = Props::<PlasticMaterial>(HashMap::new());

        app.insert_resource(plastic_props)
            .add_plugins((MaterialsPlugin,))
            .add_systems(PreStartup, load_plastic_props);
    }
}