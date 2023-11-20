use bevy::{
    app::{Plugin, Startup, Update},
    ecs::system::{Commands, Res, ResMut, Resource},
    input::{keyboard::KeyCode, Input},
};

#[derive(Clone)]
pub enum Magnitude {
    Positive,
    Negative,
    Zero,
}

impl From<Magnitude> for i32 {
    fn from(value: Magnitude) -> Self {
        match value {
            Magnitude::Positive => 1,
            Magnitude::Negative => -1,
            Magnitude::Zero => 0,
        }
    }
}
impl From<Magnitude> for f32 {
    fn from(value: Magnitude) -> Self {
        match value {
            Magnitude::Positive => 1.,
            Magnitude::Negative => -1.,
            Magnitude::Zero => 0.,
        }
    }
}

#[derive(Resource, Clone)]
pub struct MovementInput {
    pub forward: Magnitude,
    pub right: Magnitude,
}

impl Default for MovementInput {
    fn default() -> Self {
        Self {
            forward: Magnitude::Zero,
            right: Magnitude::Zero,
        }
    }
}

fn key_board_input(
    keyboard_input: Res<Input<KeyCode>>,

    mut movement_event: ResMut<MovementInput>, //need previous input
) {
    //movement keys
    {
        let w = keyboard_input.pressed(KeyCode::W);
        let s = keyboard_input.pressed(KeyCode::S);
        let a = keyboard_input.pressed(KeyCode::A);
        let d = keyboard_input.pressed(KeyCode::D);

        movement_event.right = match (a, d, &movement_event.right) {
            //two keys are pressed
            (true, true, dir) => dir.clone(),

            (false, false, _) => Magnitude::Zero,

            (true, false, _) => Magnitude::Negative,
            (false, true, _) => Magnitude::Positive,
        };

        movement_event.forward = match (s, w, &movement_event.forward) {
            //two keys are pressed
            (true, true, dir) => dir.clone(),

            (false, false, _) => Magnitude::Zero,

            (true, false, _) => Magnitude::Negative,
            (false, true, _) => Magnitude::Positive,
        };
    }

    // if keyboard_input.pressed(KeyCode::D) {
    //     transform.translation = transform.translation + local_z * 5. * time.delta_seconds();
    // }
    // if keyboard_input.pressed(KeyCode::A) {
    //     transform.translation = transform.translation + local_z * -5. * time.delta_seconds();
    // }

    // if keyboard_input.pressed(KeyCode::W) {
    //     transform.translation = transform.translation + local_x * 5. * time.delta_seconds();
    // }
    // if keyboard_input.pressed(KeyCode::S) {
    //     transform.translation = transform.translation + local_x * -5. * time.delta_seconds();
    // }
}

pub struct ControllerPlugin;

impl ControllerPlugin {
    pub fn start(mut commands: Commands) {
        commands.insert_resource(MovementInput::default());
    }
}

impl Plugin for ControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, (ControllerPlugin::start,))
            .add_systems(Update, (key_board_input,));
    }
}
