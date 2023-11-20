use bevy::{
    app::{Plugin, Update},
    ecs::{
        component::Component,
        query::{With, Without},
        system::{Query, Res},
    },
    math::Vec3,
    render::camera::Camera,
    time::Time,
    transform::components::Transform,
};

use super::{controller::MovementInput, Controllable};

#[derive(Component)]
pub struct Direction(pub Vec3);

fn update_direction_from_input(
    input: Res<MovementInput>,
    mut player_query: Query<&mut Direction, With<Controllable>>,
) {
    for mut dir in &mut player_query {
        let dir = dir.as_mut();

        let mut new_dir = Vec3::ZERO;

        new_dir.x = input.forward.clone().into();
        new_dir.z = input.right.clone().into();

        dir.0 = new_dir;
    }
}

pub fn update_pos(
    time: Res<Time>,
    camera_query: Query<&Transform, With<Camera>>,
    mut player_query: Query<(&mut Transform, &Direction), (With<Controllable>, Without<Camera>)>,
) {
    for (mut transform, direction) in &mut player_query {
        let transform = transform.as_mut();

        let (local_x, local_z) = {
            let camera: Option<&Transform> = camera_query.iter().next();
            match camera {
                Some(t) => {
                    let delta = t.translation - transform.translation;

                    let forward = Vec3 {
                        x: delta.x,
                        y: 0.,
                        z: delta.z,
                    }
                    .normalize();

                    let side_ways = Vec3 {
                        x: forward.z,
                        y: 0.,
                        z: -1. * forward.x,
                    };

                    (-1. * forward, side_ways)
                }
                None => (
                    Vec3 {
                        x: 1.,
                        y: 0.,
                        z: 0.,
                    },
                    Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 1.,
                    },
                ),
            }
        };

        let forward = local_z * direction.0.z;
        let right = local_x * direction.0.x;

        transform.translation += time.delta_seconds() * (forward + right) * 5.;
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (update_direction_from_input, update_pos));
    }
}
