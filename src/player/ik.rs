use std::f32::consts::PI;

use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::Without,
        system::Commands,
    },
    math::Ray,
    prelude::{Color, EulerRot, Gizmos, GlobalTransform, Quat, Query, Res, Transform, Vec3, With},
    render::{
        camera::Camera,
        mesh::{Indices, Mesh},
    },
    time::Time,
};
use bevy_mod_raycast::{
    immediate::{Raycast, RaycastSettings, RaycastVisibility},
    primitives::Ray3d,
};

use crate::{humanoid::Humanoid, scene::nav_mesh::NavMesh};

use super::{create::Player, movement::Direction, target::PlayerTarget, Controllable};

pub struct IKPlugin;

impl Plugin for IKPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<LegInitializeEvent>()
            .add_systems(Update, initialize_player_leg_ik)
            .add_systems(Update, (update_head_dir, update_body_dir, update_legs));
    }
}

#[derive(Event)]
pub struct LegInitializeEvent(pub Entity);

pub fn initialize_player_leg_ik(
    mut commands: Commands,
    mut ik_set_up_event: EventReader<LegInitializeEvent>,
    player_query: Query<&Humanoid, With<Player>>,
    transform_query: Query<&GlobalTransform, Without<Humanoid>>,
) {
    for &LegInitializeEvent(entity) in ik_set_up_event.iter() {
        println!("Creating ik goals");
        let humanoid = player_query.get(entity).unwrap();

        commands.entity(entity).insert(HumanoidFeetTarget::new(
            transform_query
                .get(humanoid.left_leg.2)
                .unwrap()
                .translation(),
            transform_query
                .get(humanoid.right_leg.2)
                .unwrap()
                .translation(),
        ));
    }
}

pub fn update_body_dir(
    time: Res<Time>,

    target: Res<PlayerTarget>,
    player_query: Query<&Humanoid, With<Controllable>>,
    mut bone_entities: Query<(&mut Transform, &GlobalTransform)>,

    mut gizmos: Gizmos,
) {
    let PlayerTarget(Some(target)) = target.as_ref() else {
        return;
    };

    for humanoid in &player_query {
        let (mut transform, global_transform) = bone_entities.get_mut(humanoid.body).unwrap();
        let transform = transform.as_mut();

        let dir = {
            let mut dir =
                match (target.1.position() - global_transform.translation()).try_normalize() {
                    Some(dir) => dir,
                    None => continue,
                };

            dir.y = 0.; //can be used to lean back or forward

            dir
        };

        let goal = transform.looking_to(dir, Vec3::Y).rotation;

        let target_angle = Quat::angle_between(transform.rotation, goal);

        {
            gizmos.sphere(
                global_transform.translation(),
                Quat::default(),
                0.01,
                Color::RED,
            );
            gizmos.ray(global_transform.translation(), dir, Color::RED);
        }

        //should beable to change based on state
        // -> default
        // -> actively aim (gun)
        // -> kinematic restriction
        const MIN_ANGLE: f32 = PI / 4.;

        if target_angle.abs() < MIN_ANGLE {
            continue;
        }

        const ROT_SPEED: f32 = 5.;

        transform.rotation = transform
            .rotation
            .slerp(goal, (*time).delta().as_secs_f32() * ROT_SPEED);
    }
}
pub fn update_head_dir(
    target: Res<PlayerTarget>,
    player_query: Query<&Humanoid, With<Controllable>>,
    mut bone_entities: Query<(&mut Transform, &GlobalTransform)>,
) {
    let PlayerTarget(Some(target)) = target.as_ref() else {
        return;
    };

    for humanoid in &player_query {
        //rotate head
        let dir = {
            let (_, global_head_transform) = bone_entities.get(humanoid.head).unwrap();

            match (target.1.position() - global_head_transform.translation()).try_normalize() {
                Some(dir) => dir,
                None => continue,
            }
        };

        let (mut head, _) = bone_entities.get_mut(humanoid.head).unwrap();
        let head = head.as_mut();

        let (mut x_rot, mut y_rot, _) = head
            .looking_to(dir, Vec3::Y)
            .rotation
            .to_euler(EulerRot::XYZ);

        y_rot = y_rot.clamp(-PI / 2., PI / 2.);

        let x_const = 1. - 0.98 * (y_rot.abs() / (PI / 2.));

        x_rot = x_rot.clamp(-2. * PI / 6. * x_const, 2. * PI / 6. * x_const);

        head.rotation = Quat::from_euler(EulerRot::XYZ, x_rot, y_rot, 0.);
    }
}

#[derive(Debug)]
pub enum FootTarget {
    Locked(Vec3),
    Active(Vec3),
}

#[derive(Component, Debug)]
pub struct HumanoidFeetTarget {
    pub left: FootTarget,
    pub right: FootTarget,
}

impl HumanoidFeetTarget {
    pub fn new(left_start: Vec3, right_start: Vec3) -> Self {
        HumanoidFeetTarget {
            left: FootTarget::Locked(left_start),
            right: FootTarget::Active(right_start),
        }
    }
}

fn get_edges_and_triangles(mesh: &Mesh) {
    // Get the indices and vertices
    let Some(data) = mesh.indices() else {
        return;
    };
    let indices = match data {
        Indices::U16(indices) => indices.iter().map(|i| *i as usize).collect::<Vec<_>>(),
        Indices::U32(indices) => indices.iter().map(|i| *i as usize).collect::<Vec<_>>(),
    };

    let vertices = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
        bevy::render::mesh::VertexAttributeValues::Float32x3(vertices) => vertices.clone(),
        _ => panic!("Unexpected vertex type"),
    };

    // Get triangles
    let triangles: Vec<([f32; 3], [f32; 3], [f32; 3])> = indices
        .chunks_exact(3)
        .map(|tri| {
            let v1 = vertices[tri[0]];
            let v2 = vertices[tri[1]];
            let v3 = vertices[tri[2]];
            (v1, v2, v3)
        })
        .collect();

    // Get edges
    let edges: Vec<([f32; 3], [f32; 3])> = indices
        .windows(2)
        .map(|edge| {
            let v1 = vertices[edge[0]];
            let v2 = vertices[edge[1]];
            (v1, v2)
        })
        .collect();

    // Print triangles and edges
    println!("Triangles: {:?}", triangles);
    println!("Edges: {:?}", edges);
}

fn nav_mesh_intercept(
    ray: &Ray3d,
    ray_cast: &mut Raycast,
    mesh_query: &Query<&Handle<Mesh>, With<NavMesh>>,

    gizmos: &mut Gizmos,
) -> Option<Vec3> {
    let intercepts = ray_cast.debug_cast_ray(
        ray.clone(),
        &RaycastSettings {
            visibility: RaycastVisibility::Ignore,
            filter: &|entity| mesh_query.contains(entity),
            early_exit_test: &|_| true,
        },
        gizmos,
    );

    match intercepts.len() {
        0 => None,
        _ => Some(intercepts[0].1.position()),
    }
}
fn nearest_nav_mesh_intercept(
    ray: &Ray3d,
    meshes: &Res<Assets<Mesh>>,
    mesh_query: &Query<&Handle<Mesh>, With<NavMesh>>,
) -> Option<Vec3> {
    for handle in mesh_query {
        let Some(mesh) = meshes.get(handle) else {
            continue;
        };

        // mesh.attributes()
        //     .filter(|(x, _)| x == &Mesh::ATTRIBUTE_POSITION.id)
        //     .for_each(|data| println!("{data:#?}"));
    }
    None
}

fn straight_legs_target(
    humanoid: &Humanoid,
    player_position_query: &Query<&GlobalTransform>,
    dist: f32,
) -> Vec3 {
    let down = player_position_query.get(humanoid.body).unwrap().down();

    down * dist
}

pub fn update_legs(
    // 🦵
    mut leg_query: Query<(&mut HumanoidFeetTarget, &Direction, &Humanoid)>,
    transform_query: Query<&GlobalTransform>,
    camera_query: Query<&Transform, With<Camera>>,

    // ray cast nav mesh
    mut ray_cast: Raycast,

    // gets closet nav mesh target
    meshes: Res<Assets<Mesh>>,
    nav_mesh_query: Query<&Handle<Mesh>, With<NavMesh>>,

    //debug stuff
    mut gizmos: Gizmos,
) {
    for (mut feetTarget, dir, humanoid) in &mut leg_query {
        let feet_targets: &mut HumanoidFeetTarget = feetTarget.as_mut();

        let ray = Ray3d::new(transform_query.get(humanoid.body).unwrap().translation(), {
            let dir = dir.0;

            let camera_transform = camera_query.iter().next().unwrap();
            let player_transform = transform_query.get(humanoid.body).unwrap();

            let forward = camera_transform.forward();
            let right = camera_transform.right();
            let down = player_transform.down();

            (forward * dir.x + right * dir.z + down * dir.y + down).normalize()
        });

        let mesh_pos = nav_mesh_intercept(&ray, &mut ray_cast, &nav_mesh_query, &mut gizmos);
        let nearest_pos = nearest_nav_mesh_intercept(&ray, &meshes, &nav_mesh_query);

        let foot_target = match (mesh_pos, nearest_pos) {
            (None, Some(val)) | (Some(val), None) => {
                // if val is too far - legs be straight
                val
            }
            (Some(val_1), Some(val_2)) => {
                //pick which ever one is closest
                todo!();
            }
            (None, None) => {
                // legs be straight
                straight_legs_target(humanoid, &transform_query, 2.)
            }
        };
        // if let Some(mesh_pos) = mesh_pos {
        //     gizmos.sphere(mesh_pos, Quat::default(), 2.0, Color::RED);
        // }
    }
}
