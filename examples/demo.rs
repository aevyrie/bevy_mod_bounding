use bevy::prelude::*;
use bevy_mod_bounding::{aabb, debug, obb, *};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(BoundingVolumePlugin::<sphere::BSphere>::default())
        .add_plugin(BoundingVolumePlugin::<aabb::AxisAlignedBB>::default())
        .add_plugin(BoundingVolumePlugin::<obb::OrientedBB>::default())
        .add_startup_system(setup.system())
        .add_system(rotation_system.system())
        .run();
}

struct Rotator;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh_path = "models/waterbottle/WaterBottle.gltf#Mesh0/Primitive0";
    let _scenes: Vec<HandleUntyped> = asset_server.load_folder("models").unwrap();
    let mut ortho_cam = OrthographicCameraBundle::new_3d();
    ortho_cam.transform = Transform::from_matrix(Mat4::face_toward(
        Vec3::new(-0.5, 0.5, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ));
    commands.spawn_bundle(ortho_cam);
    // AABB
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.get_handle(mesh_path),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Bounded::<aabb::AxisAlignedBB>::default())
        .insert(debug::DebugBounds)
        .insert(Rotator);
    // OBB
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.get_handle(mesh_path),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Bounded::<obb::OrientedBB>::default())
        .insert(debug::DebugBounds)
        .insert(Rotator);
    // Sphere
    commands
        .spawn_bundle(PbrBundle {
            mesh: asset_server.get_handle(mesh_path),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Bounded::<sphere::BSphere>::default())
        .insert(debug::DebugBounds)
        .insert(Rotator);
    // Light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}

/// Rotate the meshes to demonstrate how the bounding volumes update
fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in query.iter_mut() {
        let scale = Vec3::ONE * ((time.seconds_since_startup() as f32).sin() + 2.0);
        let rot_x = Quat::from_rotation_x((time.seconds_since_startup() as f32 / 5.0).sin() / 50.0);
        let rot_y = Quat::from_rotation_y((time.seconds_since_startup() as f32 / 3.0).sin() / 50.0);
        let rot_z = Quat::from_rotation_z((time.seconds_since_startup() as f32 / 4.0).sin() / 50.0);
        transform.scale = scale;
        transform.rotate(rot_x * rot_y * rot_z);
    }
}
