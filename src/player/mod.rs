use std::f32::consts::PI;

use bevy::{
    app::{App, Plugin},
    ecs::{component::Component, entity::Entity},
    prelude::{
        Camera, First, Input, IntoSystemConfigs, KeyCode, Query, Res, SpatialSettings, SpotLight,
        Startup, Transform, Update, Vec3, With, Without,
    },
    time::Time,
};
use bevy_mod_raycast::prelude::RaycastSystem;

use crate::scene::prop::sound_source::SoundSource;

use self::{controller::ControllerPlugin, movement::MovementPlugin, target::PlayerTarget};

mod controller;
mod create;
pub mod follow;
mod ik;
mod movement;
pub mod target;

pub const EAR_GAP: f32 = 0.25;

#[derive(Component)]
pub struct Controllable;

// fn move_controllable(
//     time: Res<Time>,
//     keyboard_input: Res<Input<KeyCode>>,
//     camera: Query<&Transform, (With<Camera>, Without<Controllable>)>,
//     mut query: Query<&mut Transform, (With<Controllable>, Without<Camera>)>,
// ) {
//     for mut transform in &mut query {
//         let (local_x, local_z) = {
//             let camera: Option<&Transform> = camera.iter().next();
//             match camera {
//                 Some(t) => {
//                     let delta = t.translation - transform.translation;

//                     let forward = Vec3 {
//                         x: delta.x,
//                         y: 0.,
//                         z: delta.z,
//                     }
//                     .normalize();

//                     let side_ways = Vec3 {
//                         x: forward.z,
//                         y: 0.,
//                         z: -1. * forward.x,
//                     };

//                     (-1. * forward, side_ways)
//                 }
//                 None => (
//                     Vec3 {
//                         x: 1.,
//                         y: 0.,
//                         z: 0.,
//                     },
//                     Vec3 {
//                         x: 0.,
//                         y: 0.,
//                         z: 1.,
//                     },
//                 ),
//             }
//         };

//         //println!("{local_x:?}, {local_z:?}");

//         if keyboard_input.pressed(KeyCode::D) {
//             transform.translation = transform.translation + local_z * 5. * time.delta_seconds();
//         }
//         if keyboard_input.pressed(KeyCode::A) {
//             transform.translation = transform.translation + local_z * -5. * time.delta_seconds();
//         }

//         if keyboard_input.pressed(KeyCode::W) {
//             transform.translation = transform.translation + local_x * 5. * time.delta_seconds();
//         }
//         if keyboard_input.pressed(KeyCode::S) {
//             transform.translation = transform.translation + local_x * -5. * time.delta_seconds();
//         }
//     }
// }

fn rotate_camera_view(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,

    follow_target_query: Query<(Entity, &Transform), Without<follow::Follow>>,
    mut query: Query<(&mut follow::Follow, &mut Transform), With<Camera>>,
) {
    for (mut follow, mut transform) in &mut query {
        let transform: &mut Transform = transform.as_mut();
        let follow = follow.as_mut();

        let Some(follow::FollowTarget { target, offset: _ }) = &follow.0 else {
            continue;
        };

        let target_pos = match follow_target_query.get(*target) {
            Ok(val) => val,
            Err(_err) => todo!(),
        }
        .1
        .translation;

        let shift_pressed = keyboard_input.pressed(KeyCode::ShiftLeft);
        if shift_pressed && keyboard_input.pressed(KeyCode::Q) {
            if let Some(target) = &mut follow.0 {
                if let follow::Coord::Spherical {
                    theta,
                    phi: _,
                    r: _,
                } = &mut target.offset
                {
                    *theta = *theta + time.delta_seconds() * 1.;
                }
            }
        } else if shift_pressed && keyboard_input.pressed(KeyCode::E) {
            if let Some(target) = &mut follow.0 {
                if let follow::Coord::Spherical {
                    theta,
                    phi: _,
                    r: _,
                } = &mut target.offset
                {
                    *theta = *theta + time.delta_seconds() * -1.;
                }
            }
        }

        const PHI_DELTA: f32 = 0.75;
        const PHI_RANGE: (f32, f32) = (PI / 6., PI / 4.);

        const RAD_DELTA: f32 = 50.;
        const RAD_RANGE: (f32, f32) = (10., 50.);
        if shift_pressed && keyboard_input.pressed(KeyCode::AltLeft) {
            if let Some(target) = &mut follow.0 {
                if let follow::Coord::Spherical { theta: _, phi, r } = &mut target.offset {
                    *phi =
                        (*phi + PHI_DELTA * time.delta_seconds()).clamp(PHI_RANGE.0, PHI_RANGE.1);
                    *r = (*r - RAD_DELTA * time.delta_seconds()).clamp(RAD_RANGE.0, RAD_RANGE.1);
                }
            }
        } else if shift_pressed && keyboard_input.pressed(KeyCode::Space) {
            if let Some(target) = &mut follow.0 {
                if let follow::Coord::Spherical { theta: _, phi, r } = &mut target.offset {
                    *phi = (*phi - time.delta_seconds()).clamp(PHI_RANGE.0, PHI_RANGE.1);
                    *r = (*r + RAD_DELTA * time.delta_seconds()).clamp(RAD_RANGE.0, RAD_RANGE.1);
                }
            }
        }

        transform.look_at(target_pos, Vec3::Y);
    }
}

fn update_sound_sink_pos(
    player_query: Query<&Transform, With<Controllable>>,
    mut sound_emitter_query: Query<(
        &mut SpatialSettings,
        Option<&Transform>,
        Option<&SoundSource>,
    )>,
) {
    let Some(player) = player_query.iter().next() else {
        return;
    };

    for (mut emitter, transform, sound_source) in &mut sound_emitter_query {
        let emitter: &mut SpatialSettings = emitter.as_mut();

        let new_state = match (transform, sound_source) {
            (Some(transform), None) => {
                SpatialSettings::new(player.clone(), EAR_GAP, transform.translation)
            }
            (Some(_), Some(source)) | (None, Some(source)) => {
                SpatialSettings::new(player.clone(), EAR_GAP, source.source(&player.translation))
            }
            _ => emitter.clone(),
        };

        *emitter = new_state;
    }
}
/*
pub fn update_sound_level(
    player_query: Query<&Transform, With<Controllable>>,
    mut sound_settings: Query<(&mut PlaybackSettings, Option<&Transform>, Option<&SoundSource>, &SoundVolume)>
) {
    let Some(player) = player_query.iter().next() else {
        return;
    };

    println!("Start");
    for (mut sound_setting, transform, sound_source, volume) in &mut sound_settings {
        let sound_setting: &mut PlaybackSettings = sound_setting.as_mut();

        sound_setting.volume = match (transform, sound_source) {
            (Some(transform), None) => Volume::new_relative(
                volume.sound_level(
                    (transform.translation-player.translation).length()
                )
            ),
            (Some(_), Some(source)) |
            (None, Some(source)) => {
                println!(
                    "dist: {} -> {}",
                    (source.source(&player.translation) - player.translation).length(),
                    volume.sound_level(
                        (source.source(&player.translation) - player.translation).length()
                    )
                );
                Volume::new_relative(
                    volume.sound_level(
                        (source.source(&player.translation) - player.translation).length()
                    )
                )
            },
            _=> sound_setting.volume,
        };

        sound_setting.paused = match sound_setting.volume {
            Volume::Relative(val) |
            Volume::Absolute(val) => {
                val.get() > 0.001
            },
        };
    }
}
*/
fn update_light_dir(
    target: Res<PlayerTarget>,
    mut player_query: Query<&mut Transform, With<SpotLight>>,
) {
    for mut transform in &mut player_query {
        let transform = transform.as_mut();

        let PlayerTarget(Some(target)) = target.as_ref() else {
            continue;
        };

        let point = target.1.position();

        transform.look_at(point, Vec3::Y);
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ControllerPlugin, MovementPlugin))
            .add_systems(
                First,
                target::update_player_target
                    .before(RaycastSystem::BuildRays::<target::PlayerTargetSet>),
            )
            .add_systems(Startup, (create::create_player,))
            .add_systems(
                //player movement
                Update,
                (
                    // move_controllable,
                    rotate_camera_view,
                    // movement::update_pos,
                    follow::follow,
                    update_light_dir,
                    ik::update_head_dir,
                    ik::update_body_dir,
                ),
            )
            .add_systems(
                //update sound
                Update,
                (
                    update_sound_sink_pos,
                    //update_sound_level
                ),
            );
    }
}
