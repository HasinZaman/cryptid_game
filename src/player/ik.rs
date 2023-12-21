use std::{f32::consts::PI, mem};

use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::{ReadOnlyWorldQuery, Without},
        system::Commands,
    },
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

use crate::{humanoid::Humanoid, ik::IKChain, scene::nav_mesh::NavMesh};

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
        let humanoid = player_query.get(entity).unwrap();

        let (_, _, left_leg, right_leg) = humanoid.get_iks(&transform_query);

        commands.entity(entity).insert(HumanoidFeetTarget::new(
            left_leg,
            right_leg,
            &transform_query,
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

impl FootTarget {
    pub fn map(&mut self, new: Vec3) -> &mut Self {
        match self {
            FootTarget::Locked(old) | FootTarget::Active(old) => {
                *old = new
            },
        }

        self
    }
    pub fn swap(&mut self) -> &mut Self {

        *self = match self {
            FootTarget::Locked(val) => FootTarget::Active(*val),
            FootTarget::Active(val) => FootTarget::Locked(*val),
        };


        self
    }
}

#[derive(Component, Debug)]
pub struct HumanoidFeetTarget {
    pub left_target: FootTarget,
    pub right_target: FootTarget,

    left_ik: IKChain,
    right_ik: IKChain,

    leg_offset: f32,
    leg_length_cache: f32,
}

impl HumanoidFeetTarget {
    pub fn new<F: ReadOnlyWorldQuery>(
        left_leg: IKChain,
        right_leg: IKChain,
        global_transform_query: &Query<&GlobalTransform, F>,
    ) -> Self {
        let left_target = {
            let entity = left_leg.iter().last().unwrap().0;

            global_transform_query.get(entity).unwrap().translation()
        };
        let right_target = {
            let entity = right_leg.iter().last().unwrap().0;

            global_transform_query.get(entity).unwrap().translation()
        };

        HumanoidFeetTarget {
            leg_length_cache: left_leg.length(),

            left_target: FootTarget::Locked(left_target),
            right_target: FootTarget::Active(right_target),

            left_ik: left_leg,
            right_ik: right_leg,

            leg_offset: left_target.distance(right_target) / 2.,
        }
    }

    pub fn draw_gizmo(&self, gizmos: &mut Gizmos) {
        match self.left_target {
            FootTarget::Locked(pos) => gizmos.sphere(pos, Quat::default(), 0.01, Color::RED),
            FootTarget::Active(pos) => gizmos.sphere(pos, Quat::default(), 0.05, Color::RED),
        };
        match self.right_target {
            FootTarget::Locked(pos) => gizmos.sphere(pos, Quat::default(), 0.01, Color::GREEN),
            FootTarget::Active(pos) => gizmos.sphere(pos, Quat::default(), 0.05, Color::GREEN),
        };
    }

    pub fn active_target(&self) -> &FootTarget{
        match (&self.left_target, &self.right_target) {
            (FootTarget::Locked(_), FootTarget::Active(_)) => &self.right_target,
            (FootTarget::Active(_), FootTarget::Locked(_)) => &self.left_target,
            _ => todo!()
        }
    }
    pub fn locked_target(&self) -> &FootTarget{
        match (&self.left_target, &self.right_target) {
            (FootTarget::Locked(_), FootTarget::Active(_)) => &self.left_target,
            (FootTarget::Active(_), FootTarget::Locked(_)) => &self.right_target,
            _ => todo!()
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
    down: Vec3,
    pos: Vec3,
    dist: f32,
) -> Vec3 {
    down * dist + pos
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
    for (mut feet_targets, dir, humanoid) in &mut leg_query {
        let feet_targets: &mut HumanoidFeetTarget = feet_targets.as_mut();

        let body_transform = transform_query.get(humanoid.body).unwrap();

        feet_targets.draw_gizmo(&mut gizmos);

        match (dir.0.length() > 0.01,) {
            (false, ..) => {
                let transform = transform_query.get(humanoid.body).unwrap();

                let target = straight_legs_target(transform.down(), transform.translation(), feet_targets.leg_length_cache);

                feet_targets.left_target.map(target + transform.left() * feet_targets.leg_offset);
                feet_targets.right_target.map(target + transform.right() * feet_targets.leg_offset);
                //not moving
            }
            (true, ..) => {
                //update position
                new_target(body_transform, feet_targets, dir, &camera_query, &transform_query, humanoid, &mut ray_cast, &nav_mesh_query, &mut gizmos, &meshes);
            },
        }

        //update ik to new target        
    }
}

fn new_target(body_transform: &GlobalTransform, feet_targets: &mut HumanoidFeetTarget, dir: &Direction, camera_query: &Query<'_, '_, &Transform, With<Camera>>, transform_query: &Query<'_, '_, &GlobalTransform>, humanoid: &Humanoid, ray_cast: &mut Raycast<'_, '_>, nav_mesh_query: &Query<'_, '_, &Handle<Mesh>, With<NavMesh>>, gizmos: &mut Gizmos<'_>, meshes: &Res<'_, Assets<Mesh>>) {
    let ray = Ray3d::new(
        body_transform.translation()
            + match (&feet_targets.left_target, &feet_targets.right_target) {
                (FootTarget::Locked(_), FootTarget::Active(_)) => {
                    body_transform.right() * feet_targets.leg_offset
                }
                (FootTarget::Active(_), FootTarget::Locked(_)) => {
                    body_transform.left() * feet_targets.leg_offset
                }
                _ => todo!(),
            },
        {
            let dir = dir.0;

            let camera_transform = camera_query.iter().next().unwrap();
            let player_transform = transform_query.get(humanoid.body).unwrap();

            let forward = camera_transform.forward();
            let right = camera_transform.right();
            let down = player_transform.down();

            (forward * dir.x + right * dir.z + down * dir.y + down).normalize()
        },
    );

    let mesh_pos = nav_mesh_intercept(&ray, ray_cast, nav_mesh_query, gizmos);
    let nearest_pos = nearest_nav_mesh_intercept(&ray, meshes, nav_mesh_query);

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
            let transform = transform_query.get(humanoid.body).unwrap();

            straight_legs_target(transform.down(), transform.translation(), feet_targets.leg_length_cache)
        }
    };
}
