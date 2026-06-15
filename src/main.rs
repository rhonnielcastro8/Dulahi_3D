use bevy::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, (spawn_floor, spawn_camera))
    .run();
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>, 
) {
    let floor = (
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
    );
    commands.spawn(floor);
}

fn spawn_camera(mut commands: Commands) {
    let camera = (
        Camera3d::default(),
        Transform::from_xyz(-2.0,4.5,9.0).looking_at(Vec3::ZERO, Vec3::Y)
    );
    commands.spawn(camera);
}
