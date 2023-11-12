use std::f32::consts::PI;

use bevy::{
    app::{App, Plugin},
    core_pipeline::clear_color::ClearColorConfig,
    ecs::entity::Entity,
    prelude::{
        default, AssetServer, Assets, Camera, Camera3d, Camera3dBundle, Color, Commands, Component,
        EulerRot, EventReader, First, GlobalTransform, Input, IntoSystemConfigs, KeyCode, Quat,
        Query, Ray, Res, ResMut, Resource, SpatialSettings, SpotLight, SpotLightBundle,
        StandardMaterial, Startup, Transform, Update, Vec3, With, Without,
    },
    reflect::Reflect,
    render::mesh::skinning::SkinnedMeshInverseBindposes,
    time::Time,
    window::{CursorMoved, PrimaryWindow, Window},
};
use bevy_mod_raycast::{
    prelude::{
        DeferredRaycastingPlugin, Raycast, RaycastMethod, RaycastPluginState, RaycastSettings,
        RaycastSource, RaycastSystem, RaycastVisibility,
    },
    primitives::{IntersectionData, Ray3d},
};

use crate::{
    humanoid::{load_humanoid, Humanoid},
    scene::prop::{sound_source::SoundSource, PropVisibilitySource},
};

pub const EAR_GAP: f32 = 0.25;

#[derive(Component)]
pub struct Controllable;

#[derive(Clone, Copy, Debug)]
pub enum Coord {
    Spherical { theta: f32, phi: f32, r: f32 },
    Cartesian { x: f32, y: f32, z: f32 },
}

impl Into<Vec3> for Coord {
    fn into(self) -> Vec3 {
        match self {
            Coord::Spherical { theta, phi, r } => Vec3 {
                x: r * phi.sin() * theta.cos(),
                y: r * phi.cos(),
                z: r * phi.sin() * theta.sin(),
            },
            Coord::Cartesian { x, y, z } => Vec3 { x: x, y: y, z: z },
        }
    }
}

#[derive(Debug)]
pub struct FollowTarget {
    target: Entity,
    offset: Coord,
}

#[derive(Component)]
pub struct Follow(Option<FollowTarget>);

fn follow(
    //mut controllable_query: Query<&Transform, With<Controllable>>,
    follow_target_query: Query<(Entity, &GlobalTransform), Without<Follow>>,
    mut follow_query: Query<(&mut Transform, &Follow)>,
) {
    for (mut transform, follow) in &mut follow_query {
        let transform: &mut Transform = transform.as_mut();

        let Some(FollowTarget { target, offset }) = &follow.0 else {
            continue;
        };

        let target_transform = match follow_target_query.get(*target) {
            Ok(val) => val,
            Err(_err) => todo!(),
        }
        .1
        .clone();

        let target_pos = target_transform.translation();

        //target_transform.translation = Vec3::ZERO;

        // let target_transform_matrix = transform.compute_matrix();

        //println!("{}", target);

        let offset: Vec3 = offset.clone().into();
        /*{
            let p = Vec4::new(
                offset.x,
                offset.y,
                offset.z,
                0.
            );

            let q_1 = target_rot;

            let q_2 = q_1 * (-1. / q_1.length());

            let tmp = target_rot.mul_vec3(*offset);

            let offset_new = q_1 * p * q_2;

            Vec3 {
                x: offset_new.x,
                y: offset_new.y,
                z: offset_new.z,
            }
        };*/

        transform.translation = target_pos + offset;

        // Vec3 {
        //     x: target.x + offset.x,// - 5.0,
        //     y: target.y + offset.y,// - 5.0,
        //     z: target.z+ offset.z,
        // };

        // let target = Query::get::<((Entity, Transform))>(target);
    }
    //let controllable = controllable_query.
    //todo!()
}

fn move_controllable(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    camera: Query<&Transform, (With<Camera>, Without<Controllable>)>,
    mut query: Query<&mut Transform, (With<Controllable>, Without<Camera>)>,
) {
    for mut transform in &mut query {
        let (local_x, local_z) = {
            let camera: Option<&Transform> = camera.iter().next();
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

        //println!("{local_x:?}, {local_z:?}");

        if keyboard_input.pressed(KeyCode::D) {
            transform.translation = transform.translation + local_z * 5. * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation = transform.translation + local_z * -5. * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::W) {
            transform.translation = transform.translation + local_x * 5. * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation = transform.translation + local_x * -5. * time.delta_seconds();
        }
    }
}

fn rotate_camera_view(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,

    follow_target_query: Query<(Entity, &Transform), Without<Follow>>,
    mut query: Query<(&mut Follow, &mut Transform), With<Camera>>,
) {
    for (mut follow, mut transform) in &mut query {
        let transform: &mut Transform = transform.as_mut();
        let follow = follow.as_mut();

        let Some(FollowTarget { target, offset: _ }) = &follow.0 else {
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
                if let Coord::Spherical {
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
                if let Coord::Spherical {
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
                if let Coord::Spherical { theta: _, phi, r } = &mut target.offset {
                    *phi =
                        (*phi + PHI_DELTA * time.delta_seconds()).clamp(PHI_RANGE.0, PHI_RANGE.1);
                    *r = (*r - RAD_DELTA * time.delta_seconds()).clamp(RAD_RANGE.0, RAD_RANGE.1);
                }
            }
        } else if shift_pressed && keyboard_input.pressed(KeyCode::Space) {
            if let Some(target) = &mut follow.0 {
                if let Coord::Spherical { theta: _, phi, r } = &mut target.offset {
                    *phi = (*phi - time.delta_seconds()).clamp(PHI_RANGE.0, PHI_RANGE.1);
                    *r = (*r + RAD_DELTA * time.delta_seconds()).clamp(RAD_RANGE.0, RAD_RANGE.1);
                }
            }
        }

        transform.look_at(target_pos, Vec3::Y);
    }
}

#[derive(Resource)]
pub struct PlayerTarget(Option<(Entity, IntersectionData)>);

#[derive(Component)]
pub struct PlayerTargetSet;

fn update_player_target(
    mut cursor: EventReader<CursorMoved>,

    camera_query: Query<(&Camera, &GlobalTransform)>,

    window: Query<&Window, With<PrimaryWindow>>,

    target_set_query: Query<(), With<PlayerTargetSet>>,
    mut player_target: ResMut<PlayerTarget>,

    mut ray_cast: Raycast,
) {
    let ray = {
        let Some(cursor_moved) = cursor.iter().last() else {
            return;
        };
        let Some((camera, transform)) = camera_query.iter().last() else {
            return;
        };
        let Some(window) = window.iter().last() else {
            return;
        };

        let ray = Ray3d::from_screenspace(cursor_moved.position, camera, transform, window);

        match ray {
            Some(ray) => ray,
            None => return,
        }
    };

    let settings = RaycastSettings {
        visibility: RaycastVisibility::MustBeVisibleAndInView,
        filter: &|entity| target_set_query.contains(entity),
        early_exit_test: &|_| true,
    };

    let Some(hit) = ray_cast.cast_ray(ray, &settings).iter().next() else {
        return;
    };

    let player_target = player_target.as_mut();
    player_target.0 = Some(hit.clone());
}

fn create_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    commands.insert_resource(RaycastPluginState::<PlayerTargetSet>::default());

    commands.insert_resource(PlayerTarget(None));

    //load mesh
    let (player, humanoid) = load_humanoid(
        "character\\mesh\\character.gltf",
        &mut commands,
        &asset_server,
        &mut materials,
        &mut inverse_bindposes,
    )
    .unwrap();

    commands.entity(player).insert((Controllable,));

    //followable camera
    let camera_and_light_transform = Transform::from_xyz(0., 0., 10.).looking_to(
        Vec3 {
            x: 0.,
            y: -1.,
            z: 0.,
        },
        Vec3 {
            x: 1.0,
            y: 0.,
            z: 0.,
        },
    );
    //.looking_at(Vec3::ZERO, Vec3::X);
    commands.spawn((
        Camera3dBundle {
            transform: camera_and_light_transform, // point down
            // camera: Camera {
            // // Define a viewport so we can verify screenspace rays are being constructed to
            // // account for viewport size.
            //     viewport: Some(bevy::render::camera::Viewport {
            //         ..default()
            //     }),
            //     ..default()
            // },
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0., 0., 0.)),
                ..default()
            },
            ..default()
        },
        Follow(Some(FollowTarget {
            target: player.into(),
            offset: Coord::Spherical {
                theta: PI,
                phi: PI / 6.,
                r: 50.,
            },
        })),
    ));
    commands
        .entity(humanoid.head)
        .insert((PropVisibilitySource::from_angle(PI / 4.),));

    commands.spawn((
        SpotLightBundle {
            spot_light: SpotLight {
                intensity: 2500.,
                range: 20.,
                shadows_enabled: true,
                //inner_angle: PI / 10.0,
                outer_angle: PI / 10.0,
                // radius: todo!(),
                // shadows_enabled: todo!(),
                // shadow_depth_bias: todo!(),
                // shadow_normal_bias: todo!(),
                // outer_angle: todo!(),
                // inner_angle: todo!(),
                ..default()
            },
            transform: Transform {
                rotation: Quat::from_rotation_x(PI / 2.),
                ..default()
            },
            ..default()
        },
        Follow(Some(FollowTarget {
            target: humanoid.right_arm.2,
            offset: Coord::Cartesian {
                x: 0.,
                y: 0.5,
                z: 0.,
            },
        })),
    ));
    //create camera
    //camera follows controllable

    // commands.spawn((..Components));
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

fn update_head_dir(
    target: Res<PlayerTarget>,
    player_query: Query<&Humanoid, With<Controllable>>,
    mut bone_entity: Query<(&mut Transform, &GlobalTransform)>,
) {
    let PlayerTarget(Some(target)) = target.as_ref() else {
        return;
    };

    for humanoid in &player_query {
        let (forward, right, up) = {
            let transform = bone_entity.get(humanoid.body).unwrap().1;

            (transform.forward(), transform.right(), transform.up())
        };

        //rotate body (root)
        {
            let dir = {
                let (_, global_body_transform) = bone_entity.get(humanoid.body).unwrap();

                match (target.1.position() - global_body_transform.translation()).try_normalize() {
                    Some(mut dir) => {
                        dir.y = 0.;

                        dir
                    }
                    None => continue,
                }
            };

            println!("{dir:?}");

            let (mut body, _) = bone_entity.get_mut(humanoid.body).unwrap();
            let body = body.as_mut();

            let (x_rot, mut y_rot, z_rot) = body.rotation.to_euler(EulerRot::XYZ);

            let (_, y_rot_goal, _) = body
                .looking_at(target.1.position(), Vec3::Y)
                .rotation
                .to_euler(EulerRot::XYZ);

            // if (body.rotation * body.forward()).dot(dir) < 0. {
            y_rot = y_rot_goal;
            // }

            body.rotation = Quat::from_euler(EulerRot::XYZ, x_rot, y_rot, z_rot);
        }

        //rotate head
        {
            let dir = {
                let (_, global_head_transform) = bone_entity.get(humanoid.head).unwrap();

                match (target.1.position() - global_head_transform.translation()).try_normalize() {
                    Some(dir) => dir,
                    None => continue,
                }
            };

            let (mut head, _) = bone_entity.get_mut(humanoid.head).unwrap();
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
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            update_player_target.before(RaycastSystem::BuildRays::<PlayerTargetSet>),
        )
        .add_systems(Startup, (create_player,))
        .add_systems(
            //player movement
            Update,
            (
                move_controllable,
                rotate_camera_view,
                follow,
                update_light_dir,
                update_head_dir,
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
