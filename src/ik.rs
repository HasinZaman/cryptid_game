use bevy::{ecs::{component::Component, entity::Entity, system::Query}, math::Vec3, transform::components::{Transform, GlobalTransform}, app::{Plugin, Update}};


#[derive(Component)]
pub struct IKChain(pub Vec<(Entity, f32)>,);

impl IKChain {
    pub fn new(entities: Vec<Entity>, transform_query: Query<&GlobalTransform>) -> Self {

        let mut chain = Vec::with_capacity(entities.len());

        let mut iter = entities.iter().enumerate().peekable();
        loop {
            let peek = match iter.peek() {
                Some(val)=> *val.1,
                None => break,
            };

            let (index, current) = iter.next().unwrap();

            chain[index] = (
                *current, 
                (transform_query.get(peek).unwrap().translation() -
                transform_query.get(*current).unwrap().translation()).length()
            );

        }

        IKChain(chain)
    }
}


pub enum IKConstraint {
    None
}

#[derive(Component)]
pub struct IKChainConstrains(pub Vec<IKConstraint>,);

#[derive(Component)]
pub struct IKGoal(pub Vec3);

fn ik(
    ik_query: Query<(&IKChain, Option<&IKChainConstrains>, &IKGoal)>,
    mut _transform_query: Query<&mut Transform, &GlobalTransform>
) {
    for (_chain, _constraints, _goal) in &ik_query {
        todo!();
        //probally gonna be jacobian or cmf impl of ik
    }
}

pub struct IKPlugin;

impl Plugin for IKPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, ik);
    }
}