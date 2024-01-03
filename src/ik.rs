use std::{
    ops::{Index, IndexMut},
    slice::Iter,
};

use bevy::{
    app::{Plugin, Update},
    ecs::{component::Component, entity::Entity, system::Query},
    math::Vec3,
    transform::components::{GlobalTransform, Transform},
};
use rand::distributions::uniform::UniformSampler;

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

    pub fn get(&self, index: usize) -> Option<&(Entity, f32)> {
        if index > self.0.len() {
            return None;
        }

        Some(&self[index])
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&(Entity, f32)> {
        if index > self.0.len() {
            return None;
        }

        Some(&mut self[index])
    }
}

impl Index<usize> for IKChain {
    type Output = (Entity, f32);

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl IndexMut<usize> for IKChain {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
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
    Pivot,
    Hinge,
    Saddle,
    BallAndSocket,
    CondyLoid,
    Planar,
}

#[derive(Component)]
pub struct IKChainConstrains(pub Vec<IKConstraint>);

#[derive(Component)]
pub struct IKGoal(pub Vec3);

// fn ik(
//     ik_query: Query<(&IKChain, Option<&IKChainConstrains>, &IKGoal)>,
//     mut _transform_query: Query<&mut Transform, &GlobalTransform>,
// ) {
//     for (_chain, _constraints, _goal) in &ik_query {
//         todo!();
//         //probally gonna be jacobian or cmf impl of ik
//     }
// }

pub fn ik<const MAX_ITERATION: usize>(
    chain: &IKChain,
    _constraints: Option<&IKChainConstrains>,
    transform_query: &Query<&GlobalTransform>,
    goal: &IKGoal,
    tolerance: f32,
) -> Vec<Vec3> {
    let start = transform_query
        .get(chain.iter().next().unwrap().0)
        .unwrap()
        .translation();

    let mut final_pos: Vec<Vec3> = chain
        .iter()
        .map(|(entity, _)| transform_query.get(*entity).unwrap().translation())
        .collect();

    let goal_dist = (start - goal.0).length();

    // check target is within range
    if goal_dist > chain.length() {
        //target is unreachable
        let mut joint_dist = chain.iter().map(|(_, dist)| dist);
        for i in 1..(final_pos.len() - 1) {
            let joint_dist_from_goal = (goal.0 - final_pos[i]).length();
            let ratio = joint_dist.next().unwrap() / joint_dist_from_goal;

            final_pos[i + 1] = (1. - ratio) * final_pos[i] + ratio * goal.0;
        }
    } else {
        //target is reachable
        for _iteration in 0..MAX_ITERATION {
            //if distance from final_pos & goal is less than tolerance then break
            if (goal.0 - *final_pos.last().unwrap()).length() < tolerance {
                break;
            }

            //forward reaching
            *final_pos.last_mut().unwrap() = goal.0.clone();
            for i in (1..(final_pos.len() - 1)).rev() {
                let joint_dist = (final_pos[i + 1] - final_pos[i]).length();
                let ratio = chain[i - 1].1 / joint_dist;

                final_pos[i] = (1. - ratio) * final_pos[i + 1] + ratio * final_pos[i];
            }

            //backward reaching
            *final_pos.get_mut(0).unwrap() = start.clone();
            for i in 1..final_pos.len() {
                let joint_dist = (final_pos[i + 1] - final_pos[i]).length();
                let ratio = chain[i - 1].1 / joint_dist;

                final_pos[i + 1] = (1. - ratio) * final_pos[i] + ratio * final_pos[i + 1];
            }
        }
    }

    final_pos
}

// pub struct IKPlugin;

// impl Plugin for IKPlugin {
//     fn build(&self, app: &mut bevy::prelude::App) {
//         app.add_systems(Update, ik);
//     }
// }
