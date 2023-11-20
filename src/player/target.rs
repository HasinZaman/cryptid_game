use bevy::{
    ecs::{component::Component, entity::Entity, system::Resource},
    prelude::{Camera, EventReader, GlobalTransform, Query, ResMut, With},
    window::{CursorMoved, PrimaryWindow, Window},
};
use bevy_mod_raycast::{
    prelude::{Raycast, RaycastSettings, RaycastVisibility},
    primitives::{IntersectionData, Ray3d},
};

#[derive(Resource)]
pub struct PlayerTarget(pub Option<(Entity, IntersectionData)>);

#[derive(Component)]
pub struct PlayerTargetSet;

pub fn update_player_target(
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
