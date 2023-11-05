// use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_mod_raycast::DefaultRaycastingPlugin;
use bevy_mod_raycast::prelude::DeferredRaycastingPlugin;
// use bevy::diagnostic::*;
use humanoid::HumanoidPlugin;
use lightning::LightningPlugin;
use player::PlayerPlugin;
use rain::RainPlugin;
use scene::shadow_caster::ShadowCasterMaterial;
use scene::WorldPlugin;

pub mod humanoid;
pub mod lightning;
pub mod player;
pub mod rain;
pub mod scene;
pub mod standard_material;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy_mod_raycast::low_latency_window_plugin()),
            DefaultRaycastingPlugin
        ))
        .add_plugins((
            //DefaultPlugins,
            WorldPlugin,
            PlayerPlugin,
            LightningPlugin,
            RainPlugin,
            MaterialPlugin::<ShadowCasterMaterial>::default(),
            HumanoidPlugin,
        ))
        //debug plugins
        // .add_plugins((
        //     LogDiagnosticsPlugin::default(),
        //     FrameTimeDiagnosticsPlugin::default()
        // ))
        .run();
}
