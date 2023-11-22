use bevy::{
    asset::Handle,
    ecs::{bundle::Bundle, component::Component},
    render::mesh::Mesh,
};

#[derive(Component)]
pub struct NavMesh;

#[derive(Bundle)]
pub struct NavMeshBundle {
    nav_mesh: NavMesh,
    mesh: Handle<Mesh>,
}

impl NavMeshBundle {
    pub fn new(mesh: Handle<Mesh>) -> Self {
        return NavMeshBundle {
            nav_mesh: NavMesh,
            mesh: mesh,
        };
    }
}
