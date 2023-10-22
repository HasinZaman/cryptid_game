use bevy::{
    app::Plugin,
    audio::{PlaybackMode, Volume},
    ecs::{component::Component, system::Commands},
    prelude::{AssetServer, AudioBundle, PlaybackSettings, ResMut, Startup},
};

#[derive(Component)]
pub struct Rain;

pub fn add_rain(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("rain/rain_loop.ogg"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new_relative(1.),
                speed: 1.,
                paused: false,
            },
        },
        Rain,
    ));
}
pub struct RainPlugin;

impl Plugin for RainPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, add_rain);
        //update volume based on distance to outside
    }
}
