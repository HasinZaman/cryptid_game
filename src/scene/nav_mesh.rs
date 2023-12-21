use bevy::{
    asset::Handle,
    ecs::{bundle::Bundle, component::Component},
    render::{
        mesh::Mesh,
        view::{ComputedVisibility, Visibility},
    },
    transform::components::{GlobalTransform, Transform},
};

#[derive(Component)]
pub struct NavMesh;

#[derive(Bundle)]
pub struct NavMeshBundle {
    nav_mesh: NavMesh,
    mesh: Handle<Mesh>,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

impl NavMeshBundle {
    pub fn new(mesh: Handle<Mesh>, transform: Transform) -> Self {
        return NavMeshBundle {
            nav_mesh: NavMesh,
            mesh,
            transform,
            global_transform: GlobalTransform::default(),
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
        };
    }
}
