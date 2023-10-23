use bevy::prelude::{Plugin, MaterialPlugin};

use self::plastic::PlasticMaterial;

pub mod plastic;

pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(
            (
                MaterialPlugin::<PlasticMaterial>::default(),
            )
        );
    }
}