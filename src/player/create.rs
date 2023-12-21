use std::f32::consts::PI;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{With, Without},
        system::Query,
    },
    prelude::{
        default, AssetServer, Assets, Camera3d, Camera3dBundle, Color, Commands, Quat, Res, ResMut,
        SpotLight, SpotLightBundle, StandardMaterial, Transform, Vec3,
    },
    render::mesh::skinning::SkinnedMeshInverseBindposes,
    transform::components::GlobalTransform,
};
use bevy_mod_raycast::prelude::RaycastPluginState;

use crate::{
    humanoid::{load_humanoid, Humanoid},
    scene::prop::PropVisibilitySource,
};

use super::{
    follow::{Coord, Follow, FollowTarget},
    ik::{self, LegInitializeEvent},
    movement,
    target::{PlayerTarget, PlayerTargetSet},
    Controllable,
};

#[derive(Component)]
pub struct Player;

pub fn create_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,

    mut ik_set_up_event: EventWriter<LegInitializeEvent>,
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

    ik_set_up_event.send(LegInitializeEvent(player));

    commands
        .entity(player)
        .insert((Controllable, movement::Direction(Vec3::ZERO), Player));

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
