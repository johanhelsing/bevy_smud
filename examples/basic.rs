use bevy::{
    asset::AssetServerSettings,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_so_smooth::*;

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(SoSmoothPlugin)
        // .add_startup_system(setup)
        .add_startup_system(star)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let size = Vec2::new(300.0, 300.0);

    let material = materials.add(CustomMaterial {
        color: Vec4::new(0.05, 0.05, 0.1, 1.0),
    });

    commands.spawn().insert_bundle(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad::new(size)))),
        material,
        ..Default::default()
    });

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn star(
    mut commands: Commands,
    // We will add a new Mesh for the star being created
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Let's define the mesh for the object we want to draw: a nice star.
    // We will specify here what kind of topology is used to define the mesh,
    // that is, how triangles are built from the vertices. We will use a
    // triangle list, meaning that each vertex of the triangle has to be
    // specified.
    let mut star = Mesh::new(PrimitiveTopology::TriangleList);

    // Vertices need to have a position attribute. We will use the following
    // vertices (I hope you can spot the star in the schema).
    //
    //        1
    //
    //     10   2
    // 9      0      3
    //     8     4
    //        6
    //   7        5
    //
    let mut v_pos = vec![[0.0, 0.0]];
    for i in 0..10 {
        // Angle of each vertex is 1/10 of TAU, plus PI/2 for positioning vertex 0
        let a = std::f32::consts::FRAC_PI_2 - i as f32 * std::f32::consts::TAU / 10.0;
        // Radius of internal vertices (2, 4, 6, 8, 10) is 100, it's 200 for external
        let r = (1 - i % 2) as f32 * 100.0 + 100.0;
        // Add the vertex coordinates
        v_pos.push([r * a.cos(), r * a.sin()]);
    }
    // Set the position attribute
    star.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    // And a RGB color attribute as well
    let mut v_color = vec![[0.0, 0.0, 0.0, 1.0]];
    v_color.extend_from_slice(&[[1.0, 1.0, 0.0, 1.0]; 10]);
    star.set_attribute(Mesh::ATTRIBUTE_COLOR, v_color);

    // Now, we specify the indices of the vertex that are going to compose the
    // triangles in our star. Vertices in triangles have to be specified in CCW
    // winding (that will be the front face, colored). Since we are using
    // triangle list, we will specify each triangle as 3 vertices
    //   First triangle: 0, 2, 1
    //   Second triangle: 0, 3, 2
    //   Third triangle: 0, 4, 3
    //   etc
    //   Last triangle: 0, 1, 10
    let mut indices = vec![0, 1, 10];
    for i in 2..=10 {
        indices.extend_from_slice(&[0, i, i - 1]);
    }
    star.set_indices(Some(Indices::U32(indices)));

    // We can now spawn the entities for the star and the camera
    commands.spawn_bundle((
        // We use a marker component to identify the custom colored meshes
        SmudShape::default(),
        // The `Handle<Mesh>` needs to be wrapped in a `Mesh2dHandle` to use 2d rendering instead of 3d
        Mesh2dHandle(meshes.add(star)),
        // These other components are needed for 2d meshes to be rendered
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
    commands
        // And use an orthographic projection
        .spawn_bundle(OrthographicCameraBundle::new_2d());
}
