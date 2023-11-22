use std::f32::consts::PI;

use bevy::audio::{PlaybackMode, Volume};
use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    App, AssetServer, Assets, Commands, MaterialMeshBundle, PlaybackSettings, Plugin, Res, ResMut,
    SpatialSettings, Startup,
};
use bevy::transform::components::Transform;

use crate::player::target::PlayerTargetSet;

use self::floor::{FloorMaterial, FloorPlugin, Floors};
use self::nav_mesh::NavMeshBundle;
use self::prop::materials::plastic::PlasticMaterial;
use self::prop::sound_source::{PropSoundBundle, SoundSource, SoundVolume};
use self::prop::{PropPlugin, PropVisibility, PropVisibilityBlocker, PropVisibilityTarget, Props};
use self::shadow_caster::ShadowCasterMaterial;
use self::wall::{WallMaterial, WallPlugin, Walls};

pub mod floor;
pub mod nav_mesh;
pub mod prop;
pub mod shadow_caster;
pub mod wall;

fn create_scene(
    mut commands: Commands,
    floors: Res<Floors>,
    walls: Res<Walls>,
    plastic_props: Res<Props<PlasticMaterial>>,
    asset_server: ResMut<AssetServer>,
    mut floor_materials: ResMut<Assets<FloorMaterial>>,
    mut wall_materials: ResMut<Assets<WallMaterial>>,
    mut shadow_caster_material: ResMut<Assets<ShadowCasterMaterial>>,
    mut plastic_material: ResMut<Assets<PlasticMaterial>>,
) {
    //shadow caster
    {
        commands.spawn((
            MaterialMeshBundle {
                mesh: asset_server.load(
                    "scenes/dev_playground/room_shadow_caster/mesh/mesh.glb#Mesh0/Primitive0",
                ),
                material: shadow_caster_material.add(Default::default()),
                ..Default::default()
            },
            PropVisibilityBlocker,
        ));
    }
    //walls
    {
        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_2").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: -10.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 3.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_4").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 14.,
                        y: 0.,
                        z: 0.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 1.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_1").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 0.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_1").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 14.,
                        y: 0.,
                        z: -10.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 2.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));

        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_3").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: -10.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 3.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_3").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 14.,
                        y: 0.,
                        z: 0.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 1.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_3").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 0.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            wall::into_mesh_bundle(
                walls.as_ref().0.get("dev_playground/wall_3").unwrap(),
                &mut wall_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 14.,
                        y: 0.,
                        z: -10.,
                    },
                    rotation: Quat::from_rotation_y(PI / 2. * 2.),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
    }
    //floors
    {
        commands.spawn((
            floor::into_mesh_bundle(
                floors
                    .as_ref()
                    .0
                    .get("dev_playground/metal_grate_floor")
                    .unwrap(),
                &mut floor_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 2.,
                        y: 0.,
                        z: -2.,
                    },
                    rotation: Quat::default(),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            floor::into_mesh_bundle(
                floors
                    .as_ref()
                    .0
                    .get("dev_playground/metal_grate_floor")
                    .unwrap(),
                &mut floor_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 2.,
                        y: 0.,
                        z: -6.,
                    },
                    rotation: Quat::default(),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));
        commands.spawn((
            floor::into_mesh_bundle(
                floors
                    .as_ref()
                    .0
                    .get("dev_playground/stainless_steel_floor")
                    .unwrap(),
                &mut floor_materials,
                Some(Transform {
                    translation: Vec3 {
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    rotation: Quat::default(),
                    scale: Vec3 {
                        x: 1.,
                        y: 1.,
                        z: 1.,
                    },
                }),
            ),
            PlayerTargetSet,
        ));

        commands.spawn((
            NavMeshBundle::new(
                asset_server.load("scenes/dev_playground/nav_mesh/nav_mesh.glb#Mesh0/Primitive0"),
            ),
            PlayerTargetSet,
        ));
    }
    //windows
    {
        let rain_window_loop = asset_server.load("rain/rain_window_loop.ogg");
        commands.spawn((
            //window mesh
            PropSoundBundle {
                sound_source: SoundSource::Point(Vec3 {
                    x: 1.55556,
                    y: 1.58101,
                    z: 0.,
                }),
                source: rain_window_loop.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    volume: Volume::new_relative(2.),
                    ..Default::default()
                },
                spatial: SpatialSettings::new(Transform::default(), 1.0, Vec3::ZERO),
            },
            SoundVolume::new(0.5, 10.),
        ));
        commands.spawn((
            //window mesh
            PropSoundBundle {
                sound_source: SoundSource::Point(Vec3 {
                    x: 4.66667,
                    y: 1.58101,
                    z: 0.,
                }),
                source: rain_window_loop.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    volume: Volume::new_relative(2.),
                    ..Default::default()
                },
                spatial: SpatialSettings::new(Transform::default(), 1.0, Vec3::ZERO),
            },
            SoundVolume::new(0.5, 10.),
        ));
        commands.spawn((
            //window mesh
            PropSoundBundle {
                sound_source: SoundSource::Point(Vec3 {
                    x: 7.77778,
                    y: 1.58101,
                    z: 0.,
                }),
                source: rain_window_loop.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    volume: Volume::new_relative(2.),
                    ..Default::default()
                },
                spatial: SpatialSettings::new(Transform::default(), 1.0, Vec3::ZERO),
            },
            SoundVolume::new(0.5, 10.),
        ));
        commands.spawn((
            //window mesh
            PropSoundBundle {
                sound_source: SoundSource::Point(Vec3 {
                    x: 10.8889,
                    y: 1.58101,
                    z: 0.,
                }),
                source: rain_window_loop.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    volume: Volume::new_relative(2.),
                    ..Default::default()
                },
                spatial: SpatialSettings::new(Transform::default(), 1.0, Vec3::ZERO),
            },
            SoundVolume::new(0.5, 10.),
        ));
    }

    //props
    {
        commands.spawn((
            prop::into_mesh_bundle(
                plastic_props.0.get("plastic_bin_1").unwrap(),
                &mut plastic_material,
                Some(Transform {
                    translation: Vec3 {
                        x: 6.,
                        y: 0.,
                        z: -5.,
                    },
                    scale: Vec3::new(3., 3., 3.),
                    rotation: Quat::from_rotation_y(0.5),
                }),
            ),
            plastic_props.0.get("plastic_bin_1").unwrap().clone(),
            PropVisibility::Hidden,
            //prop::Forgettable,
            PropVisibilityTarget::from(vec![
                Vec3::new(0.051597, 0.046506, 0.031542),
                Vec3::new(0.051597, 0.046506, 0.468458),
                Vec3::new(1.0584, 0.046506, 0.031542),
                Vec3::new(1.0584, 0.046506, 0.468458),
            ]),
            PlayerTargetSet,
        ));
    }
    // commands.spawn((
    //     a
    // ))
    // commands.spawn((
    //     window::into_mesh_bundle(
    //         windows
    //             .as_ref()
    //             .0
    //             .get("dev_playground/window_1")
    //             .unwrap(),
    //         &mut window_materials,
    //         Some(Transform {
    //             translation: Vec3 {
    //                 x: 5.,
    //                 y: 1.,
    //                 z: -5.,
    //             },
    //             rotation: Quat::default(),
    //             scale: Vec3 {
    //                 x: 1.,
    //                 y: 1.,
    //                 z: 1.,
    //             },
    //         }),
    //     ),
    //     PlayerTargetSet,
    // ));
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PropPlugin, FloorPlugin, WallPlugin))
            .add_systems(Startup, (create_scene,)); //PostStartup (Load scene)
                                                    //.add_systems(
                                                    //     Update,
                                                    //     (
                                                    //         move_to,
                                                    //         follow
                                                    //     )
                                                    // );
    }
}
