use bevy::{prelude::{MaterialPlugin, Plugin, Update, ResMut, Assets, Query, Handle, GlobalTransform, Vec3}, reflect::TypeUuid, asset::Asset};

use self::plastic::PlasticMaterial;

pub mod plastic;

// create rotation trait -> maybe with derive trait
// create generic update_rotation method as the code is basically the same

pub(self) trait Directions {
    fn set_direction(&mut self, forward: Vec3, right: Vec3, up: Vec3);
    fn get_direction(&self) -> (&Vec3, &Vec3, &Vec3);
}
pub(self) trait Position {
    fn set_position(&mut self, new_position: Vec3);
    fn get_position(&self) -> &Vec3;
}


fn update_direction_system<M: TypeUuid + Asset + Directions>(
    mut materials: ResMut<Assets<M>>,
    query: Query<(&Handle<M>, &GlobalTransform)>
) {
    for (material, global_transform) in &query {
    
        let Some(material) = materials.as_mut().get_mut(material) else {
            continue;
        };

        let (forward, right, up) =  (global_transform.forward(), global_transform.right(), global_transform.up(),);

        if material.get_direction() != (&forward, &right, &up) {
            material.set_direction(forward, right, up);
        }
    }
}
fn update_position_system<M: TypeUuid + Asset + Position>(
    mut materials: ResMut<Assets<M>>,
    query: Query<(&Handle<M>, &GlobalTransform)>
) {
    for (material, global_transform) in &query {
    
        let Some(material) = materials.as_mut().get_mut(material) else {
            continue;
        };

        let (_scale, _rotation, position) = global_transform.to_scale_rotation_translation();
        
        if material.get_position() != &position {
            material.set_position(position.clone());
        }
    }
}

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((MaterialPlugin::<PlasticMaterial>::default(),))
            .add_systems(
                Update,
                (
                    update_direction_system::<PlasticMaterial>,
                    update_position_system::<PlasticMaterial>,
                )
            );
    }
}
