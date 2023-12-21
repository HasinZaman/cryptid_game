use std::slice::Iter;

use bevy::{
    app::{Plugin, Update},
    ecs::{component::Component, entity::Entity, system::Query},
    math::Vec3,
    transform::components::{GlobalTransform, Transform},
};

#[derive(Component, Debug)]
pub struct IKChain(pub Vec<(Entity, f32)>);

impl IKChain {
    pub fn new(entities: Vec<Entity>, transform_query: Query<&GlobalTransform>) -> Self {
        entities
            .iter()
            .map(|entity| {
                (
                    *entity,
                    transform_query.get(*entity).unwrap().compute_transform(),
                )
            })
            .collect()
    }
    pub fn length(&self) -> f32 {
        self.0.iter().fold(0., |acc, (_, length)| acc + length)
    }

    pub fn iter(&self) -> Iter<'_, (Entity, f32)> {
        self.0.iter()
    }
}

impl FromIterator<(Entity, Transform)> for IKChain {
    fn from_iter<T: IntoIterator<Item = (Entity, Transform)>>(iter: T) -> Self {
        let mut chain = Vec::new();

        let mut iter = iter.into_iter().peekable();

        loop {
            let (_, next_transform) = match iter.peek() {
                Some(val) => *val,
                None => break,
            };

            let (entity, current_transform) = iter.next().unwrap();

            chain.push((
                entity,
                (next_transform.translation - current_transform.translation).length(),
            ));
        }

        match iter.next() {
            Some((entity, _)) => chain.push((entity, 0.)),
            None => {}
        };

        return IKChain(chain);
    }
}

pub enum IKConstraint {
    None,
}

#[derive(Component)]
pub struct IKChainConstrains(pub Vec<IKConstraint>);

#[derive(Component)]
pub struct IKGoal(pub Vec3);

fn ik(
    ik_query: Query<(&IKChain, Option<&IKChainConstrains>, &IKGoal)>,
    mut _transform_query: Query<&mut Transform, &GlobalTransform>,
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
