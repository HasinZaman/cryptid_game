use bevy::{
    asset::Asset,
    prelude::{
        default, App, AssetServer, Assets, Color, Commands, Component, Entity, GlobalTransform,
        Handle, Material, MaterialMeshBundle, Mesh, Plugin, PreStartup, Query, ResMut, Resource,
        Startup, Transform, Update, Vec3, With,
    },
    reflect::TypeUuid,
    utils::HashMap,
};
use bevy_mod_raycast::{
    prelude::{Raycast, RaycastSettings, RaycastVisibility},
    primitives::Ray3d,
};

use self::materials::{plastic::PlasticMaterial, MaterialsPlugin};

use super::shadow_caster::ShadowCasterMaterial;

pub mod materials;
pub mod sound_source;

#[derive(Component)]
pub struct PropVisibilityBlocker;
#[derive(Component)]
pub struct PropVisibilitySource(f32);

impl PropVisibilitySource {
    pub fn from_angle(angle: f32) -> Self {
        Self(f32::cos(angle))
    }
    pub fn from_cos(cos: f32) -> Self {
        Self(cos)
    }
}

impl From<f32> for PropVisibilitySource {
    fn from(value: f32) -> Self {
        PropVisibilitySource::from_cos(value)
    }
}

impl Default for PropVisibilitySource {
    fn default() -> Self {
        Self(0.25)
    }
}

#[derive(Component)]
pub struct PropVisibilityTarget(pub Vec<Vec3>);

impl From<Vec3> for PropVisibilityTarget {
    fn from(value: Vec3) -> Self {
        PropVisibilityTarget(vec![value])
    }
}
impl From<Vec<Vec3>> for PropVisibilityTarget {
    fn from(value: Vec<Vec3>) -> Self {
        PropVisibilityTarget(value)
    }
}
impl From<&Transform> for PropVisibilityTarget {
    fn from(value: &Transform) -> Self {
        PropVisibilityTarget(vec![value.translation])
    }
}
impl Default for PropVisibilityTarget {
    fn default() -> Self {
        Self(vec![Vec3::ZERO])
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum PropVisibility {
    Seen,
    Hidden,
}

#[derive(Component)]
pub struct Forgettable;

pub fn hide_unseen_props<M: TypeUuid + Asset + Material>(
    mut commands: Commands,
    query: Query<(Entity, &PropVisibility), (With<Handle<M>>, With<Prop<M>>)>,
    mut shadow_caster_material: ResMut<Assets<ShadowCasterMaterial>>,
) {
    for (entity, visibility) in &query {
        if PropVisibility::Seen == *visibility {
            continue;
        };

        commands
            .entity(entity)
            .remove::<Handle<M>>()
            .insert(shadow_caster_material.add(ShadowCasterMaterial::default()));
    }
}
pub fn show_seen_props<M: TypeUuid + Asset + Material>(
    mut commands: Commands,
    query: Query<(Entity, &Prop<M>, &PropVisibility), With<Handle<ShadowCasterMaterial>>>,
    mut material: ResMut<Assets<M>>,
) {
    for (entity, prop, visibility) in &query {
        if PropVisibility::Hidden == *visibility {
            continue;
        };

        commands
            .entity(entity)
            .remove::<Handle<ShadowCasterMaterial>>()
            .insert(material.add(prop.material.clone()));
    }
}

pub fn update_prop_visibility(
    source_query: Query<(&GlobalTransform, &PropVisibilitySource)>,

    mut prop_query: Query<
        (
            Entity,
            &GlobalTransform,
            Option<&Forgettable>,
            &mut PropVisibility,
        ),
        (With<Handle<Mesh>>, With<PropVisibilityTarget>),
    >,
    target_query: Query<&PropVisibilityTarget, With<Handle<Mesh>>>,
    blocker_query: Query<(), (With<PropVisibilityBlocker>, With<Handle<Mesh>>)>,

    mut ray_cast: Raycast,
    // mut gizmos: Gizmos,
) {
    for (source_transform, source) in &source_query {
        let origin = source_transform.translation();

        // gizmos.ray(origin, source_transform.forward(), Color::RED);

        for target in &mut prop_query {
            let (target, target_transform, forgettable, mut visibility) = target;

            let target = target_query.get(target).unwrap();

            'pos_loop: for target_pos in target.0.iter() {
                let target_pos = {
                    let mut pos = *target_pos;
                    pos.z *= -1.;

                    target_transform.compute_matrix().project_point3(pos)
                };

                // gizmos.sphere(target_pos, Quat::IDENTITY, 0.05, Color::RED);

                let direction = (target_pos - origin).normalize();

                if source_transform.forward().dot(direction) < source.0 {
                    continue;
                }

                let ray = Ray3d::new(origin, direction);
                let settings = RaycastSettings {
                    visibility: RaycastVisibility::MustBeVisible,
                    filter: &|entity: Entity| {
                        target_query.contains(entity) || blocker_query.contains(entity)
                    },
                    early_exit_test: &|_| true,
                };
                // ray_cast.debug_cast_ray(ray, &settings, &mut gizmos);

                let ray_hits = ray_cast.cast_ray(ray, &settings);

                let Some((entity, _)) = ray_hits.iter().next() else {
                    continue;
                };
                {
                    let visibility = visibility.as_mut();
                    match (target_query.contains(*entity), &visibility, forgettable) {
                        (false, PropVisibility::Seen, None)
                        | (false, PropVisibility::Hidden, _)
                        | (true, PropVisibility::Seen, _) => {}

                        (true, PropVisibility::Hidden, _) => {
                            *visibility = PropVisibility::Seen;
                            break 'pos_loop;
                        }

                        (false, PropVisibility::Seen, Some(_)) => {
                            *visibility = PropVisibility::Hidden;
                        }
                    }
                }
            }
        }
    }
}

//create macro to add prop
macro_rules! prop_visibility_system {
    [$material: ty] => {(
        hide_unseen_props::<$material>,
        show_seen_props::<$material>,
    )};
}
#[derive(Resource)]
pub struct Props<M>(pub HashMap<String, Prop<M>>);

#[derive(Component, Clone)]
pub struct Prop<M> {
    pub mesh: Handle<Mesh>,
    pub material: M,
}

pub fn into_mesh_bundle<M: TypeUuid + Asset + Material>(
    prop: &Prop<M>,
    materials: &mut ResMut<Assets<M>>,
    transform: Option<Transform>,
) -> MaterialMeshBundle<M> {
    // (
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
    } //,
      //     cached_material
      // )
}

fn load_plastic_prop(
    asset_server: &ResMut<AssetServer>,
    dir: &'static str,
) -> Prop<PlasticMaterial> {
    Prop {
        mesh: asset_server.load(format!("props/{dir}/mesh/mesh.glb#Mesh0/Primitive0")),
        material: PlasticMaterial {
            colour: Color::rgba(0.8, 0.3, 0., 1.),
            noise_texture_1: Some(asset_server.load(format!("props/{dir}/textures/noise.png"))),
            ..PlasticMaterial::default()
        },
    }
}

pub fn load_plastic_props(
    mut props: ResMut<Props<PlasticMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    props.as_mut().0.insert(
        "plastic_bin_1".into(),
        load_plastic_prop(&asset_server, "plastic_bin_1"),
    );
}

pub fn setup(mut _commands: Commands) {}

pub struct PropPlugin;

impl Plugin for PropPlugin {
    fn build(&self, app: &mut App) {
        let plastic_props = Props::<PlasticMaterial>(HashMap::new());

        app.insert_resource(plastic_props)
            .add_plugins((MaterialsPlugin,))
            .add_systems(Startup, setup)
            .add_systems(PreStartup, load_plastic_props)
            .add_systems(Update, update_prop_visibility)
            .add_systems(Update, prop_visibility_system![PlasticMaterial]);
    }
}
