use bevy::{
    ecs::{entity::Entity, component::Component},
    prelude::{
        GlobalTransform, Query, Transform, Vec3, Without,
    },
};


#[derive(Clone, Copy, Debug)]
pub enum Coord {
    Spherical { theta: f32, phi: f32, r: f32 },
    Cartesian { x: f32, y: f32, z: f32 },
}

impl Into<Vec3> for Coord {
    fn into(self) -> Vec3 {
        match self {
            Coord::Spherical { theta, phi, r } => Vec3 {
                x: r * phi.sin() * theta.cos(),
                y: r * phi.cos(),
                z: r * phi.sin() * theta.sin(),
            },
            Coord::Cartesian { x, y, z } => Vec3 { x: x, y: y, z: z },
        }
    }
}

#[derive(Debug)]
pub struct FollowTarget {
    pub target: Entity,
    pub offset: Coord,
}

#[derive(Component)]
pub struct Follow(pub Option<FollowTarget>);

pub fn follow(
    //mut controllable_query: Query<&Transform, With<Controllable>>,
    follow_target_query: Query<(Entity, &GlobalTransform), Without<Follow>>,
    mut follow_query: Query<(&mut Transform, &Follow)>,
) {
    for (mut transform, follow) in &mut follow_query {
        let transform: &mut Transform = transform.as_mut();

        let Some(FollowTarget { target, offset }) = &follow.0 else {
            continue;
        };

        let target_transform = match follow_target_query.get(*target) {
            Ok(val) => val,
            Err(_err) => todo!(),
        }
        .1
        .clone();

        let target_pos = target_transform.translation();

        //target_transform.translation = Vec3::ZERO;

        // let target_transform_matrix = transform.compute_matrix();

        //println!("{}", target);

        let offset: Vec3 = offset.clone().into();
        /*{
            let p = Vec4::new(
                offset.x,
                offset.y,
                offset.z,
                0.
            );

            let q_1 = target_rot;

            let q_2 = q_1 * (-1. / q_1.length());

            let tmp = target_rot.mul_vec3(*offset);

            let offset_new = q_1 * p * q_2;

            Vec3 {
                x: offset_new.x,
                y: offset_new.y,
                z: offset_new.z,
            }
        };*/

        transform.translation = target_pos + offset;

        // Vec3 {
        //     x: target.x + offset.x,// - 5.0,
        //     y: target.y + offset.y,// - 5.0,
        //     z: target.z+ offset.z,
        // };

        // let target = Query::get::<((Entity, Transform))>(target);
    }
    //let controllable = controllable_query.
    //todo!()
}
