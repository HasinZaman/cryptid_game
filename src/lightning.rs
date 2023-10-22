use std::f32::consts::PI;

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::{
        default, AssetServer, AudioSource, Commands, Component, DirectionalLight,
        DirectionalLightBundle, Entity, Handle, PlaybackSettings, Plugin, Quat, Query, Res, ResMut,
        Resource, SpatialAudioBundle, SpatialSettings, Startup, Transform, Update, Vec3,
        Visibility, With,
    },
    time::{Time, Timer, TimerMode},
};

use rand::SeedableRng;
use rand::{rngs::SmallRng, RngCore};

use crate::player::{Controllable, EAR_GAP};

#[derive(Resource)]
struct ThunderSoundEffect(Vec<Handle<AudioSource>>);

const SOURCE_HEIGHT: f32 = 5.;
// const VISIBILITY_TIME: f32 = 0.25;

#[derive(Debug)]
pub enum ScaryState {
    Lightning(Timer),
    Wait(Timer),
    Thunder(Timer),
    Done,
}
impl ScaryState {
    pub fn next(&mut self, timer: Option<Timer>) -> bool {
        match self {
            ScaryState::Lightning(_) => {
                *self = ScaryState::Wait(match timer {
                    Some(timer) => timer,
                    None => return false,
                });
                return true;
            }
            ScaryState::Wait(_) => {
                *self = ScaryState::Thunder(match timer {
                    Some(timer) => timer,
                    None => return false,
                });
                return true;
            }
            ScaryState::Thunder(_) => {
                *self = ScaryState::Done;
                return true;
            }
            ScaryState::Done => {
                return false;
            }
        }
    }
}

#[derive(Component, Debug)]
pub enum Lightning {
    Calm { wait_timer: Timer, rng: SmallRng },
    Scary { state: ScaryState, rng: SmallRng },
}

fn generate_calm_state(rng: &mut SmallRng) -> Lightning {
    let time = get_float(rng) * 10.; // + 10.;
    Lightning::Calm {
        wait_timer: Timer::from_seconds(time, TimerMode::Once),
        rng: rng.clone(),
    }
}

fn generate_scary_state(
    transform: &mut Transform,
    visibility: &mut Visibility,
    rng: &mut SmallRng,
) -> Lightning {
    let _ = transform.looking_to(
        Vec3 {
            x: get_float(rng) * 50. - 25.,
            y: SOURCE_HEIGHT,
            z: get_float(rng) * 50. - 25.,
        } - transform.translation,
        Vec3::Y,
    );

    *visibility = Visibility::Visible;

    let time = get_float(rng) * 0.15 + 0.10;
    Lightning::Scary {
        state: ScaryState::Lightning(Timer::from_seconds(time, TimerMode::Once)),
        rng: rng.clone(),
    }
}

fn set_up_lightning(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let sound_effects = ThunderSoundEffect(
        (1..9)
            .map(|index| format!("lightning/thunder_{index}.ogg"))
            .map(|path| asset_server.load::<AudioSource, &str>(&path))
            .collect(),
    );

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 50_000.,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, SOURCE_HEIGHT, 0.0),
                rotation: Quat::from_rotation_z(-PI / 6.),
                ..Default::default()
            },
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        generate_calm_state(&mut SmallRng::from_entropy()),
    ));

    commands.insert_resource(sound_effects);
}

fn get_float(rng: &mut SmallRng) -> f32 {
    (rng.next_u32() % 1000) as f32 / 1000.
}

fn update_light_state(
    mut commands: Commands,
    sound_effects: Res<ThunderSoundEffect>,
    player_query: Query<&Transform, With<Controllable>>,
    mut query: Query<(
        Entity,
        &Transform,
        &mut DirectionalLight,
        &mut Lightning,
        &mut Visibility,
    )>,
) {
    for (
        entity,
        transform,
        mut light,
        mut lightning_state,
        mut visibility, /*, mut play_back_settings */
    ) in &mut query
    {
        let Lightning::Scary { state, rng } = lightning_state.as_mut() else {
            continue;
        };

        match state {
            ScaryState::Lightning(timer) => {
                let percent = timer.percent() - 0.5;

                let x = percent - 0.5;
                light.illuminance = (get_float(rng) * 75_000. + 25_000.) * (-4. * (x * x) + 1.);

                if timer.finished() {
                    *visibility = Visibility::Hidden;
                    state.next(Some(Timer::from_seconds(
                        get_float(rng) * 2.0,
                        TimerMode::Once,
                    )));
                }
            }
            ScaryState::Wait(timer) => {
                if timer.finished() {
                    let Some(listener) = player_query.iter().next() else {
                        continue;
                    };

                    commands.entity(entity).insert(SpatialAudioBundle {
                        source: sound_effects.0[rng.next_u32() as usize % sound_effects.0.len()]
                            .clone(),
                        settings: PlaybackSettings {
                            mode: PlaybackMode::Remove,
                            volume: Volume::new_relative(get_float(rng) * 20. + 15.),
                            ..default()
                        },
                        spatial: SpatialSettings::new(
                            listener.clone(),
                            EAR_GAP,
                            transform.translation,
                        ),
                    });
                    state.next(Some(Timer::from_seconds(25., TimerMode::Once)));
                }
            }
            ScaryState::Thunder(timer) => {
                if timer.finished() {
                    state.next(None);
                }
            }
            ScaryState::Done => {
                //do nothing
            }
        };
    }
}

fn update_lightning_timer(time: Res<Time>, mut query: Query<&mut Lightning>) {
    for mut lightning in &mut query {
        match &mut lightning.as_mut() {
            Lightning::Calm { wait_timer, .. } => {
                wait_timer.tick(time.delta());
            }
            Lightning::Scary { state, .. } => {
                match state {
                    ScaryState::Lightning(wait_timer)
                    | ScaryState::Wait(wait_timer)
                    | ScaryState::Thunder(wait_timer) => {
                        wait_timer.tick(time.delta());
                    }
                    ScaryState::Done => {}
                };
            }
        };
    }
}

fn update_lightning(
    // sound_effects: Res<ThunderSoundEffect>,
    mut lightning_query: Query<(
        // &mut PlaybackSettings,
        // &mut Handle<AudioSource>,
        &mut Transform,
        &mut Visibility,
        &mut Lightning,
    )>,
) {
    for (
        /*mut play_back_settings, mut audio_source,*/ mut transform,
        mut visibility,
        mut state,
    ) in &mut lightning_query
    {
        let change_state = match state.as_mut() {
            Lightning::Calm { wait_timer, .. } => wait_timer.finished(),
            Lightning::Scary { state, .. } => match state {
                ScaryState::Done => true,
                _ => false,
            },
        };

        if change_state {
            *state = match state.as_mut() {
                Lightning::Calm { wait_timer: _, rng } => {
                    //play_back_settings.paused = true;
                    generate_scary_state(transform.as_mut(), visibility.as_mut(), rng)
                }
                Lightning::Scary { state: _, rng } => {
                    *visibility = Visibility::Hidden;

                    // *audio_source = sound_effects.0[0].clone();
                    // play_back_settings.paused = false;

                    generate_calm_state(rng)
                }
            };
        }
    }
}

pub struct LightningPlugin;

impl Plugin for LightningPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, set_up_lightning).add_systems(
            Update,
            (update_lightning_timer, update_light_state, update_lightning),
        );
    }
}
