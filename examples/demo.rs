use bevy::prelude::*;
use bevy_mod_bounding::*;

// This example will show you how to use your mouse cursor as a ray casting source, cast into the
// scene, intersect a mesh, and mark the intersection with the built in debug cursor. If you are
// looking for a more fully-featured mouse picking plugin, try out bevy_mod_picking.

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        //.add_plugin(BoundingVolumePlugin::<BSphere>::default())
        .add_plugin(BoundingVolumePlugin::<AxisAlignedBB>::default())
        .add_plugin(BoundingVolumePlugin::<OrientedBB>::default())
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
    let mesh_path = "models/waterbottle/WaterBottle.gltf#Mesh0/Primitive0";
    let _scenes: Vec<HandleUntyped> = asset_server.load_folder("models").unwrap();
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
            mesh: asset_server.get_handle(mesh_path),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(Bounded::<AxisAlignedBB>::default())
        .with(DebugBounds)
        .with(Rotator)
        .with(Mesh::from(Mesh::from(shape::Icosphere {
            radius: 5.0,
            ..Default::default()
        })))
        .spawn(PbrBundle {
            mesh: asset_server.get_handle(mesh_path),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            ..Default::default()
        })
        .with(Bounded::<OrientedBB>::default())
        .with(DebugBounds)
        .with(Rotator)
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in query.iter_mut() {
        let scale = Transform::from_scale(
            Vec3::one() * ((time.seconds_since_startup() as f32 / 2.0).sin() * 0.01) + Vec3::one(),
        );
        let rot_x = Quat::from_rotation_x((time.seconds_since_startup() as f32 / 5.0).sin() / 20.0);
        let rot_y = Quat::from_rotation_y((time.seconds_since_startup() as f32 / 3.0).sin() / 20.0);
        let rot_z = Quat::from_rotation_z((time.seconds_since_startup() as f32 / 4.0).sin() / 20.0);
        *transform = *transform * scale * Transform::from_rotation(rot_x * rot_y * rot_z);
    }
}
