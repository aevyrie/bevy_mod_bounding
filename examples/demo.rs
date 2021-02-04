use bevy::prelude::*;
use bevy_mod_bounding::*;

// This example will show you how to use your mouse cursor as a ray casting source, cast into the
// scene, intersect a mesh, and mark the intersection with the built in debug cursor. If you are
// looking for a more fully-featured mouse picking plugin, try out bevy_mod_picking.

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(BoundingVolumePlugin)
        .add_startup_system(setup.system())
        .add_system(rotation_system.system())
        .run();
}

struct Rotator;

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let _scenes: Vec<HandleUntyped> = asset_server.load_folder("models").unwrap();
    let mesh_handle1 = asset_server.get_handle("models/waterbottle/WaterBottle.gltf#Mesh0/Primitive0");
    let mesh_handle2 = asset_server.get_handle("models/waterbottle/WaterBottle.gltf#Mesh0/Primitive0");
    let mesh_handle3 = asset_server.get_handle("models/waterbottle/WaterBottle.gltf#Mesh0/Primitive0");
    let mesh_handle4 = asset_server.get_handle("models/waterbottle/WaterBottle.gltf#Mesh0/Primitive0");
    commands
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(0.0, 1.0, 3.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .spawn(PbrBundle {
            mesh: mesh_handle1,
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(-1.5, 0.0, 0.0)),
            ..Default::default()
        })
        .with(BoundingVolume::<BoundingSphere>::default())
        .with(BoundingVolumeDebug)
        .with(Rotator)
        .spawn(PbrBundle {
            mesh: mesh_handle2,
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(-0.5, 0.0, 0.0)),
            ..Default::default()
        })
        .with(BoundingVolume::<AxisAlignedBoundingBox>::default())
        .with(BoundingVolumeDebug)
        .with(Rotator)
        .spawn(PbrBundle {
            mesh: mesh_handle3,
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.5, 0.0, 0.0)),
            ..Default::default()
        })
        .with(BoundingVolume::<OrientedBoundingBox>::default())
        .with(BoundingVolumeDebug)
        .with(Rotator)
        .spawn(PbrBundle {
            mesh: mesh_handle4,
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(1.5, 0.0, 0.0)),
            ..Default::default()
        })
        //.with(BoundingVolume::<BoundingSphere>::default())
        //.with(BoundingVolumeDebug)
        .with(Rotator)
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in query.iter_mut(){
        let rotx = Quat::from_rotation_x((time.seconds_since_startup() as f32/5.0).sin()/10.0);
        let roty = Quat::from_rotation_y((time.seconds_since_startup() as f32/3.0).sin()/10.0);
        let rotz = Quat::from_rotation_z((time.seconds_since_startup() as f32/4.0).sin()/10.0);
        *transform = *transform * Transform::from_rotation(rotx*roty*rotz);
    }
}