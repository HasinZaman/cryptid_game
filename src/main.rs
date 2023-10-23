use bevy::prelude::*;
use bevy_mod_raycast::prelude::low_latency_window_plugin;
use lightning::LightningPlugin;
use player::PlayerPlugin;
use rain::RainPlugin;
use scene::shadow_caster::ShadowCasterMaterial;
use scene::WorldPlugin;

pub mod lightning;
pub mod player;
pub mod rain;
pub mod scene;
pub mod standard_material;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            //DefaultPlugins,
            WorldPlugin,
            PlayerPlugin,
            LightningPlugin,
            RainPlugin,
        ))
        .add_plugins((MaterialPlugin::<ShadowCasterMaterial>::default(),))
        .run();
}
