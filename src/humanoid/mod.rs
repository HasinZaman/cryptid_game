use std::collections::{HashMap, VecDeque};

use bevy::{
    pbr::SkinnedMeshJoints,
    prelude::{
        App, AssetServer, Assets, BuildChildren, Commands, Component, ComputedVisibility, Entity,
        GlobalTransform, Handle, Mat4, Mesh, Plugin, Quat, Query, Res, ResMut, StandardMaterial,
        Transform, Update, Vec3, Visibility,
    },
    render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
};
use gltf::Node;

fn convert_transform((translation, rotation, scale): ([f32; 3], [f32; 4], [f32; 3])) -> Transform {
    let mut transform = Transform::default();

    transform.translation = Vec3 {
        x: translation[0],
        y: translation[1],
        z: translation[2],
    };

    transform.rotation = Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]);

    transform.scale = Vec3 {
        x: scale[0],
        y: scale[1],
        z: scale[2],
    };

    // println!("{translation:?}\t{rotation:?}\t{scale:?}\n{:?}", transform);

    transform
}

#[derive(Debug)]
pub struct Limb(pub Entity, pub Entity, pub Entity);
#[derive(Clone)]
pub struct LimbBuilder(Option<Entity>, Option<Entity>, Option<Entity>);

impl Default for LimbBuilder {
    fn default() -> Self {
        Self(None, None, None)
    }
}

impl From<LimbBuilder> for Limb {
    fn from(value: LimbBuilder) -> Self {
        Self(value.0.unwrap(), value.1.unwrap(), value.2.unwrap())
    }
}

#[derive(Component, Debug)]
pub struct Humanoid {
    pub head: Entity,
    pub body: Entity,
    pub left_arm: Limb,
    pub right_arm: Limb,
    pub left_leg: Limb,
    pub right_leg: Limb,
    pub meshes: HashMap<String, Entity>,
}

impl Humanoid {
    pub fn debug(&self, entity_query: &Query<&GlobalTransform>) -> String {
        let mut output = String::new();

        output += &format!(
            "head:{:?}",
            entity_query.get(self.head).unwrap().translation()
        );

        output += &format!(
            "\nbody:{:?}",
            entity_query.get(self.body).unwrap().translation()
        );
        {
            output += &format!(
                "\nleft arm 0:{:?}",
                entity_query.get(self.left_arm.0).unwrap().translation()
            );
            output += &format!(
                "\nleft arm 1:{:?}",
                entity_query.get(self.left_arm.1).unwrap().translation()
            );
            output += &format!(
                "\nleft hand:{:?}",
                entity_query.get(self.left_arm.2).unwrap().translation()
            );
        }
        {
            output += &format!(
                "\nright arm 0:{:?}",
                entity_query.get(self.right_arm.0).unwrap().translation()
            );
            output += &format!(
                "\nright arm 1:{:?}",
                entity_query.get(self.right_arm.1).unwrap().translation()
            );
            output += &format!(
                "\nright hand:{:?}",
                entity_query.get(self.right_arm.2).unwrap().translation()
            );
        }
        {
            output += &format!(
                "\nleft leg 0:{:?}",
                entity_query.get(self.left_leg.0).unwrap().translation()
            );
            output += &format!(
                "\nleft leg 1:{:?}",
                entity_query.get(self.left_leg.1).unwrap().translation()
            );
            output += &format!(
                "\nleft foot:{:?}",
                entity_query.get(self.left_leg.2).unwrap().translation()
            );
        }
        {
            output += &format!(
                "\nright leg 0:{:?}",
                entity_query.get(self.right_leg.0).unwrap().translation()
            );
            output += &format!(
                "\nright leg 1:{:?}",
                entity_query.get(self.right_leg.1).unwrap().translation()
            );
            output += &format!(
                "\nright foot:{:?}",
                entity_query.get(self.right_leg.2).unwrap().translation()
            );
        }
        for (name, entity) in &self.meshes {
            output += &format!(
                "\n{name}:{:?}",
                entity_query.get(*entity).unwrap().translation()
            );
        }

        output
    }
}

#[derive(Clone)]
struct HumanoidBuilder {
    head: Option<Entity>,
    body: Option<Entity>,
    left_arm: LimbBuilder,
    right_arm: LimbBuilder,
    left_leg: LimbBuilder,
    right_leg: LimbBuilder,
    meshes: HashMap<String, Entity>,
}

impl Default for HumanoidBuilder {
    fn default() -> Self {
        Self {
            head: None,
            body: None,
            left_arm: LimbBuilder::default(),
            right_arm: LimbBuilder::default(),
            left_leg: LimbBuilder::default(),
            right_leg: LimbBuilder::default(),
            meshes: HashMap::new(),
        }
    }
}

impl From<HumanoidBuilder> for Humanoid {
    fn from(value: HumanoidBuilder) -> Self {
        Humanoid {
            head: value.head.unwrap(),
            body: value.body.unwrap(),
            left_arm: value.left_arm.into(),
            right_arm: value.right_arm.into(),
            left_leg: value.left_leg.into(),
            right_leg: value.right_leg.into(),
            meshes: value.meshes,
        }
    }
}

pub fn load_humanoid(
    humanoid_asset_path: &str,
    commands: &mut Commands,

    asset_server: &Res<AssetServer>,

    standard_material: &mut ResMut<Assets<StandardMaterial>>,
    inverse_bindposes: &mut ResMut<Assets<SkinnedMeshInverseBindposes>>,
) -> Option<(Entity, Humanoid)> {
    //start from start en
    let (gltf, buffers, _) = gltf::import(&format!("assets/{humanoid_asset_path}")).ok()?;

    let mut humanoid_builder = HumanoidBuilder::default();

    let entities: Vec<(Entity, Transform)> = gltf
        .nodes()
        .map(|node| {
            let transform = convert_transform(node.transform().decomposed());
            let entity = commands
                .spawn((
                    transform.clone(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    ComputedVisibility::default(),
                ))
                .id();

            if let Some(mesh) = node.mesh() {
                let mesh: Handle<Mesh> = asset_server.load(&format!(
                    "{humanoid_asset_path}#Mesh{}/Primitive0",
                    mesh.index()
                ));
                commands
                    .entity(entity)
                    .insert((mesh, standard_material.add(StandardMaterial::default())));
            }

            if let Some(name) = node.name() {
                match name {
                    "head" => {
                        humanoid_builder.head = Some(entity);
                    }

                    "Spine" => {
                        humanoid_builder.body = Some(entity);
                    }

                    "right.arm.0" => {
                        humanoid_builder.right_arm.0 = Some(entity);
                    }
                    "right.arm.1" => {
                        humanoid_builder.right_arm.1 = Some(entity);
                    }
                    "right.hand" => {
                        humanoid_builder.right_arm.2 = Some(entity);
                    }

                    "left.arm.0" => {
                        humanoid_builder.left_arm.0 = Some(entity);
                    }
                    "left.arm.1" => {
                        humanoid_builder.left_arm.1 = Some(entity);
                    }
                    "left.hand" => {
                        humanoid_builder.left_arm.2 = Some(entity);
                    }

                    "right.leg.upper" => {
                        humanoid_builder.right_leg.0 = Some(entity);
                    }
                    "right.leg.lower" => {
                        humanoid_builder.right_leg.1 = Some(entity);
                    }
                    "right.foot" => {
                        humanoid_builder.right_leg.2 = Some(entity);
                    }

                    "left.leg.upper" => {
                        humanoid_builder.left_leg.0 = Some(entity);
                    }
                    "left.leg.lower" => {
                        humanoid_builder.left_leg.1 = Some(entity);
                    }
                    "left.foot" => {
                        humanoid_builder.left_leg.2 = Some(entity);
                    }

                    name => {
                        if node.mesh().is_some() {
                            humanoid_builder.meshes.insert(name.into(), entity);
                        }
                    }
                }
            }
            //println!("{:#?}",humanoid_builder.meshes);
            (entity, transform)
        })
        .collect();

    //generate skinned mesh

    let skins: Vec<SkinnedMesh> = gltf
        .skins()
        .map(|skin| {
            let joints: Vec<Entity> = skin.joints()
                .map(|node| {
                    commands
                        .entity(entities[node.index()].0)
                        .insert(SkinnedMeshJoints {
                            index: skin.index() as u32,
                        });

                    entities[node.index()].0
                })
                .collect();

            let inverse_bind_matrices = SkinnedMeshInverseBindposes::from(
                match skin.inverse_bind_matrices() {
                    Some(accessor) => {
                        let get_buffer_data = |buffer: gltf::Buffer| buffers.get(buffer.index()).map(|x| &*x.0);
    
                        gltf::accessor::Iter::<[[f32; 4]; 4]>::new(accessor, get_buffer_data)
                            .unwrap()
                            .map(|matrix| {
                                Mat4::from_cols(
                                    matrix[0].into(),
                                    matrix[1].into(),
                                    matrix[2].into(),
                                    matrix[3].into(),
                                )
                            })
                            .collect::<Vec<Mat4>>()   
                    }
                    None => (0..joints.len())
                        .map(|_| Mat4::IDENTITY)
                        .collect::<Vec<Mat4>>(),
                }
            );

            SkinnedMesh {
                inverse_bindposes: inverse_bindposes.add(inverse_bind_matrices),
                joints,
            }
        })
        .collect();

    let mut queue: VecDeque<usize> = VecDeque::new();

    let start_index = gltf
        .scenes()
        .next()?
        .nodes()
        .next()?
        .index();

    queue.push_back(start_index);

    let nodes: Vec<Node<'_>> = gltf.nodes().collect();

    while !queue.is_empty() {
        let index = queue.pop_front().unwrap();
        let current_node = &entities[index];

        if let Some(skin) = nodes[index].skin() {
            commands
                .entity(current_node.0)
                .insert(skins[skin.index()].clone());
        }

        let _ = &nodes[index].children().for_each(|node| {
            commands
                .entity(current_node.0)
                .push_children(&[entities[node.index()].0]);

            queue.push_back(node.index());
        });
    }

    let start_entity = entities[start_index].0;

    let component: Humanoid = humanoid_builder.clone().into();

    commands.entity(start_entity).insert(component);
    return Some((start_entity, humanoid_builder.into()));
}

fn rotate_head(mut entities: Query<&mut Transform>, query: Query<&Humanoid>) {
    for humanoid in &query {
        if let Ok(mut arm) = entities.get_mut(humanoid.left_arm.0) {
            let arm = arm.as_mut();

            arm.rotate_z(0.001);
        }
        if let Ok(mut arm) = entities.get_mut(humanoid.right_arm.0) {
            let arm = arm.as_mut();

            arm.rotate_x(0.001);
        }
        if let Ok(mut arm) = entities.get_mut(humanoid.head) {
            let arm = arm.as_mut();

            arm.rotate_y(0.001);
        }
    }
}

pub struct HumanoidPlugin;

impl Plugin for HumanoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (rotate_head,));
        // let plastic_props = Props::<PlasticMaterial>(HashMap::new());

        // app.insert_resource(plastic_props)
        //     .add_plugins((MaterialsPlugin,))
        //     .add_systems(Startup, setup)
        //     .add_systems(PreStartup, load_plastic_props)
        //     .add_systems(Update, update_prop_visibility)
        //     .add_systems(Update, prop_visibility_system![PlasticMaterial]);
    }
}
