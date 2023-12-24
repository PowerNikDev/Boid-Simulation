use std::f32::consts::PI;
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::{PrimaryWindow, WindowMode, WindowResolution};
use crate::quadtree::QuadTree;
use crate::quadtree::Rectangle;

mod quadtree;

use rand::{thread_rng, Rng};

#[derive(Resource)]
struct SimulationSettings
{
    boid_size: u32,
    seperation_factor: f32,
    alignment_factor: f32,
    cohesion_factor: f32,
    perception_range: f32,
    protected_range: f32,
    min_speed: f32,
    max_speed: f32,
    turn_factor: f32,
    attraction_point_factor: f32
}

impl SimulationSettings
{
    fn new (
        boid_size: u32,
        seperation_factor: f32,
        alignment_factor: f32,
        cohesion_factor: f32,
        perception_range: f32,
        protected_range: f32,
        min_speed: f32,
        max_speed: f32,
        turn_factor: f32,
        attraction_point_factor: f32
    ) -> SimulationSettings
    {
        SimulationSettings { boid_size: boid_size, seperation_factor: seperation_factor, alignment_factor: alignment_factor, 
            cohesion_factor: cohesion_factor, perception_range: perception_range, protected_range: protected_range, min_speed: min_speed, max_speed: max_speed, 
            turn_factor: turn_factor, attraction_point_factor: attraction_point_factor}
    }

    fn default() -> SimulationSettings
    {
        SimulationSettings {boid_size: 500, seperation_factor: 0.3, alignment_factor: 0.075, 
            cohesion_factor: 0.055, perception_range: 60.0, protected_range: 12.0, min_speed: 5.0, max_speed: 5.1, turn_factor: 0.4, attraction_point_factor: 0.1}
    }
}


#[derive(Component)]
struct AttractionPoint
{
    position: Vec2
}


#[derive(Component)]
struct Movement
{
    velocity: Vec2,
    position: Vec2
}

#[derive(Component)]
struct Boid{id: u32}

#[derive(Component)]
struct DebugPoint;


fn main() 
{
    App::new()
    .insert_resource(SimulationSettings::default())
    .insert_resource(QuadTree::new(Rectangle {position: Vec2::ZERO, size: Vec2::new(1300.0, 1300.0)}, 6))
    .add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
        mode: WindowMode::Windowed,
        resizable: false,
        resolution: WindowResolution::new(920.0, 920.0),
        ..default()
        }),
        ..default()
    }), LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
    .add_systems(Startup, (load_simulation, spawn_camera))
    .add_systems(Update, mouse_button_input)
    .add_systems(Update, simulate)
    .add_systems(Update, apply_velocity.after(simulate))
    .add_systems(Update, draw_boids.after(apply_velocity))
    .run();
}


fn spawn_camera(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.get_single().unwrap();

    commands.spawn(
        Camera2dBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0).with_scale(Vec3::new(1.0, 1.0, 1.0)),
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::WHITE)
            },
            ..default()
        }
    );
}


// Create triangle to represent a boid
fn create_triangle() -> Mesh 
{
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0], [1.0, -2.0, 0.0], [-1.0, -2.0, 0.0]],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 1.0]; 3]);
    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));
    mesh
}

// Creates all boids at the beginning of the simulation and calculates the margins for the boid to turn around at the edges of the screen 
fn load_simulation (
    mut commands: Commands, 
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    simulation_settings: Res<SimulationSettings>,
    mut quadtree: ResMut<QuadTree>
) 
{
    let window = window_query.get_single().unwrap();

    let mut rng = thread_rng();
    let mut x: f32;
    let mut y: f32;

    let mut a: usize = 0;

    commands.spawn((
        MaterialMesh2dBundle {
            transform: Transform::from_xyz(100.0, 100.0, 0.0).with_scale(Vec3::new(10.0, 10.0, 10.0)).with_rotation(Quat::from_rotation_z(0.0)),
            mesh: meshes.add(create_triangle()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            ..default()
        },
        Boid {id: 0},
        Movement {velocity: Vec2::new(10.0, 10.0), position: Vec2::new(110.0, 110.0)}
    ));
    quadtree.insert((Vec2::new(100.0, 100.0), Vec2::new(10.0, 10.0)));

    for i in 1..simulation_settings.boid_size as u32 {
        a += 1;
        x = rng.gen_range(-(window.width() / 2.0) .. window.width() / 2.0) as f32; 
        y = rng.gen_range(-(window.height() / 2.0) .. (window.height() / 2.0)) as f32;

        commands.spawn((
            MaterialMesh2dBundle {
                transform: Transform::from_xyz(x, y, 0.0).with_scale(Vec3::new(3.0, 3.0, 3.0)).with_rotation(Quat::from_rotation_z(0.0)),
                mesh: meshes.add(create_triangle()).into(),
                material: materials.add(ColorMaterial::from(Color::BLACK)),
                ..default()
            },
            Boid {id: i},
            Movement {velocity: Vec2::new(x, y), position: Vec2::new(x - (window.width() / 2.0) + i as f32 * 10.0, y - (window.height() / 2.0))}
        ));
        quadtree.insert((Vec2::new(x - (window.width() / 2.0) + i as f32 * 10.0, y - (window.height() / 2.0)), Vec2::new(x, y)));
    }

    println!("{}", a);
}
 
// Calculate the velocity of the boids
fn simulate(mut boid_query: Query<(&mut Movement, &Boid)>, 
  window_query: Query<&Window, With<PrimaryWindow>>, 
  simulation_settings: Res<SimulationSettings>,
  attraction_points_query: Query<&AttractionPoint>,
  quadtree: Res<QuadTree>,
  mut commands: Commands,
  debug_points: Query<(&DebugPoint, Entity)>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>)
{
    debug_points.for_each(|point| commands.entity(point.1).despawn());
    let window = window_query.get_single().unwrap();
    let mut changes = Vec::new();

    for boid in boid_query.iter() 
    {
        // Velocity change for ...
        // ... seperation
        let mut seperation_dv = Vec2::ZERO;
        // ... alignment
        let mut alignment_dv = Vec2::ZERO;
        let mut neighboring_boids: u32 = 0;
        // ... cohesion
        let mut cohesion_position = Vec2::ZERO;
        // ... turning at the edge of the screen
        let mut turn_dv = Vec2::ZERO;
        

        // Computes the average position of every attraction points so the boids can fly towards it
        let mut attraction_position = Vec2::ZERO;
        // Using this variable instead of attraction_points_query.iter().count() because the method performs with O(n) time complexity because it has to iterate over the whole iterator, to get
        // the average position this must be done necessarily why we can just do it in there. This may be a small improvement, but not doing it knowing it is not perfect bothered me
        let mut attraction_count = 0;
        for attraction_point in attraction_points_query.iter()
        {
            attraction_position += attraction_point.position;
            attraction_count += 1;
        }
        attraction_position /= attraction_count as f32;

        // Iterate over every other boid
        for other_boid in quadtree.query(&Rectangle{position: boid.0.position, size: Vec2::new(simulation_settings.perception_range, simulation_settings.perception_range)}).iter()
        {
            let distance = boid.0.position.distance(other_boid.0);

            // Checks if the other boid is visible to the current boid
            if distance < simulation_settings.perception_range
            {
                if boid.1.id == 0
                {
                    commands.spawn((
                        MaterialMesh2dBundle {
                            transform: Transform::from_xyz(other_boid.0.x, other_boid.0.y, 1.0).with_rotation(Quat::from_rotation_z(0.0)),
                            mesh: meshes.add(shape::Circle::new(3.0).into()).into(),
                            material: materials.add(ColorMaterial::from(Color::GREEN)),
                            ..default()
                        },
                        DebugPoint {}
                    ));
                }
                // Seperation
                if distance < simulation_settings.protected_range
                {
                    let offset = boid.0.position - other_boid.0;
                    seperation_dv += offset.normalize() * (simulation_settings.protected_range - offset.length()).abs();
                }

                // Alignment
                alignment_dv += other_boid.1;

                // Cohesion
                cohesion_position += other_boid.0;
                neighboring_boids += 1;
            }
        }
        
        alignment_dv /= neighboring_boids as f32;
        cohesion_position /= neighboring_boids as f32;

        // Turns the boid a away from the edges of the window
        if boid.0.position.x < 150.0
        {
            turn_dv.x += simulation_settings.turn_factor;
        }
        if boid.0.position.x > (window.width() - 150.0)
        {
            turn_dv.x -= simulation_settings.turn_factor;
        }
        if boid.0.position.y < 150.0
        {
            turn_dv.y += simulation_settings.turn_factor;
        }
        if boid.0.position.y > (window.height() - 150.0)
        {
            turn_dv.y -= simulation_settings.turn_factor;
        }

        changes.push(seperation_dv.normalize_or_zero() * simulation_settings.seperation_factor
         + turn_dv.normalize_or_zero() * simulation_settings.turn_factor
         + (alignment_dv - boid.0.velocity).normalize_or_zero() * simulation_settings.alignment_factor
         + (cohesion_position - boid.0.position).normalize_or_zero() * simulation_settings.cohesion_factor
         + (if boid.1.id % 2 == 0 {(attraction_position - boid.0.position).normalize_or_zero() * simulation_settings.attraction_point_factor} else {Vec2::ZERO}));
    }

    for (mut boid, change) in boid_query.iter_mut().zip(changes.into_iter()) 
    {
        boid.0.velocity += change;
    }
}


// Calculate the new position of the boids and updates the quadtree
fn apply_velocity(mut boid_query: Query<&mut Movement>, simulation_settings: Res<SimulationSettings>, mut quadtree: ResMut<QuadTree>) 
{
    for mut boid in &mut boid_query
    {
        boid.velocity = boid.velocity.clamp_length(simulation_settings.min_speed, simulation_settings.max_speed);
        let old = (boid.position, boid.velocity);
        boid.position.x += boid.velocity.x;
        boid.position.y += boid.velocity.y;
        quadtree.move_point(old, (boid.position, boid.velocity))
    }
}


// Moves the sprites of the boids
fn draw_boids(mut boid_query: Query<(&mut Movement, &mut Transform)>) 
{
    for (movement, mut transfrom) in &mut boid_query
    {
        transfrom.translation = Vec3::new(movement.position.x, movement.position.y, 0.0);
        transfrom.rotation = Quat::from_rotation_z(movement.velocity.y.atan2(movement.velocity.x) - (90.0 * PI/180.0));
        //transfrom.look_at(Vec3::new(movement.velocity.x, movement.velocity.y, 0.0), Vec3::new(0.0, 1.0, 0.0));
    }
}


// Handle mouse input and place points to which the boids are attracted to
fn mouse_button_input(buttons: Res<Input<MouseButton>>, 
    window_query: Query<&Window, With<PrimaryWindow>>, 
    camera_query: Query<(&Camera, &GlobalTransform)>, 
    mut attraction_points_query: Query<(&mut AttractionPoint, Entity)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>)
{
    let window = window_query.single();
    let (camera, camera_transform) = camera_query.single();
    
    // Try to get the position of the cursor
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        // Add attraction point at the position of the cursor if the left mouse button was pressed
        if buttons.just_pressed(MouseButton::Left)
        {
            commands.spawn((
                MaterialMesh2dBundle {
                    transform: Transform::from_xyz(world_position.x, world_position.y, 1.0).with_rotation(Quat::from_rotation_z(0.0)),
                    mesh: meshes.add(shape::Circle::new(3.0).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::RED)),
                    ..default()
                },
                AttractionPoint {
                    position: world_position
                }
            ));
        }

        // Despawns attraction point at the position of the cursor if the right mouse button was pressed
        if buttons.pressed(MouseButton::Right)
        {
            attraction_points_query.iter_mut()
            .filter(|point| (point.0.position - world_position).length() < 7.0)
            .for_each(|point| commands.entity(point.1).despawn());
        }
    }
}