use bevy::prelude::{Vec3, Component, Bundle, Handle, AudioSource, PlaybackSettings, SpatialSettings};


#[derive(Component)]
pub enum SoundSource {
    Point(Vec3),
    Area(Vec<Vec3>),
}

impl SoundSource {
    pub fn source(&self, listener_pos: &Vec3) -> Vec3 {
        match self {
            SoundSource::Point(pos) => pos.clone(),
            SoundSource::Area(pos) => {
                pos.iter()
                    .max_by(
                        |vec_1, vec_2| (*listener_pos - **vec_1).length()
                            .total_cmp(&(*listener_pos - **vec_2).length())
                    ).unwrap()//should handle if vec list is empty
                    .clone()
            },
        }
    }
}

#[derive(Component)]
pub struct SoundVolume {
    m: f32,
    b: f32,
}

impl SoundVolume {
    pub fn new(max: f32, drop_of_dist: f32) -> Self {
        Self {
            m: -1. * max / drop_of_dist,
            b: max,
        }
    }
    pub fn sound_level(&self, dist: f32) -> f32 {
        (self.m * dist + self.b).max(0.)
    }
}

impl Default for SoundVolume {
    fn default() -> Self {
        Self::new(1., 10.)
    }
}


impl Default for SoundSource {
    fn default() -> Self {
        SoundSource::Point(Vec3::ZERO)
    }
}

#[derive(Bundle)]
pub struct PropSoundBundle {
    pub sound_source: SoundSource,
    pub source: Handle<AudioSource>,
    pub settings: PlaybackSettings,
    pub spatial: SpatialSettings,
}

// pub struct SoundSourcePlugin;

// impl Plugin for SoundSourcePlugin {

// }
