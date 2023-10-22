use std::f32::consts::PI;

use bevy::math::{Quat, Vec3};
use bevy::prelude::{
    App, AssetServer, Assets, Commands, MaterialMeshBundle, Plugin, ResMut, Startup, Res,
};
use bevy::transform::components::Transform;
use bevy_mod_raycast::RaycastMesh;

use crate::player::PlayerTargetSet;

use self::floor::{FloorMaterial, FloorPlugin, Floors};
use self::shadow_caster::ShadowCasterMaterial;
use self::wall::{WallMaterial, WallPlugin, Walls};

pub mod floor;
pub mod shadow_caster;
pub mod wall;

fn create_scene(
    mut commands: Commands,
    floors: Res<Floors>,
    walls: Res<Walls>,
    asset_server: ResMut<AssetServer>,
    mut floor_materials: ResMut<Assets<FloorMaterial>>,
    mut wall_materials: ResMut<Assets<WallMaterial>>,
    mut shadow_caster_material: ResMut<Assets<ShadowCasterMaterial>>,
) {
    //shadow caster
    {
        commands.spawn(MaterialMeshBundle {
            mesh: asset_server
                .load("scenes/dev_playground/room_shadow_caster/mesh/mesh.glb#Mesh0/Primitive0"),
            material: shadow_caster_material.add(Default::default()),
            ..Default::default()
        });
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
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
            RaycastMesh::<PlayerTargetSet>::default(),
        ));
    }
    //windows
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
    //     RaycastMesh::<PlayerTargetSet>::default(),
    // ));
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FloorPlugin, WallPlugin))
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
