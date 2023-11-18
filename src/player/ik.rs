use std::f32::consts::PI;

use bevy::{
    prelude::{
        Color, EulerRot, Gizmos, GlobalTransform, Quat, Query, Res, Transform, Vec3, With,
    },
    time::Time,
};

use crate::humanoid::Humanoid;

use super::{target::PlayerTarget, Controllable};

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