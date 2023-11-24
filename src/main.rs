use std::f32::consts::PI;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::{PrimaryWindow, WindowMode, WindowResolution};

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
    turn_factor: f32
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
        turn_factor: f32
    ) -> SimulationSettings
    {
        SimulationSettings { boid_size: boid_size, seperation_factor: seperation_factor, alignment_factor: alignment_factor, 
            cohesion_factor: cohesion_factor, perception_range: perception_range, protected_range: protected_range, min_speed: min_speed, max_speed: max_speed, turn_factor: turn_factor}
    }

    fn default() -> SimulationSettings
    {
        SimulationSettings {boid_size: 1000, seperation_factor: 0.3, alignment_factor: 0.075, 
            cohesion_factor: 0.055, perception_range: 60.0, protected_range: 12.0, min_speed: 5.0, max_speed: 5.1, turn_factor: 0.4}
    }
}

#[derive(Resource)]
struct ScreenMargins
{
    left_margin: f32,
    right_margin: f32,
    top_margin: f32,
    bottom_margin: f32
}

#[derive(Component)]
struct Movement
{
    velocity: Vec2,
    position: Vec2
}

#[derive(Component)]
struct Boid{id: u32}

fn main() 
{
    App::new()
    .insert_resource(SimulationSettings::default())
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
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
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
    simulation_settings: Res<SimulationSettings>
) 
{
    let window = window_query.get_single().unwrap();

    let mut rng = thread_rng();
    let mut x: f32;
    let mut y: f32;

    for i in 0..simulation_settings.boid_size as u32 {
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
            Movement {velocity: Vec2::new(x, y), position: Vec2::new(x - (window.width() / 2.0), y - (window.height() / 2.0))}
        ));
    }   

    commands.insert_resource(ScreenMargins{ left_margin: window.width() / 6.0, right_margin: window.width() - (window.width() / 6.0), 
        top_margin: window.height() / 5.0 , bottom_margin: window.height() - (window.height() / 5.0)});

}
 
// Calculate the velocity of the boids
fn simulate(mut boid_query: Query<(&mut Movement, &Boid)>, window_query: Query<&Window, With<PrimaryWindow>>,simulation_settings: Res<SimulationSettings>, screen_margins: Res<ScreenMargins>) 
{
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

        // Iterate over every other boid
        for other_boid in boid_query.iter() 
        {
            if other_boid.1.id == boid.1.id
            {
                continue;
            }
            let distance = boid.0.position.distance(other_boid.0.position);

            // Checks if the other boid is visible to the current boid
            if distance < simulation_settings.perception_range
            {
                // Seperation
                if distance < simulation_settings.protected_range
                {
                    let offset = boid.0.position - other_boid.0.position;
                    seperation_dv += offset.normalize() * (simulation_settings.protected_range - offset.length()).abs();
                }

                // Alignment
                alignment_dv += other_boid.0.velocity;
                cohesion_position += other_boid.0.position;
                neighboring_boids += 1;
            }
        }
        
        alignment_dv /= neighboring_boids as f32;
        cohesion_position /= neighboring_boids as f32;

        // Turns the boid a away from the edges of the window
        if boid.0.position.x < 100.0
        {
            turn_dv.x += simulation_settings.turn_factor;
        }
        if boid.0.position.x > (window.width() - 100.0)
        {
            turn_dv.x -= simulation_settings.turn_factor;
        }
        if boid.0.position.y < 100.0
        {
            turn_dv.y += simulation_settings.turn_factor;
        }
        if boid.0.position.y > (window.height() - 100.0)
        {
            turn_dv.y -= simulation_settings.turn_factor;
        }

        changes.push(seperation_dv.normalize_or_zero() * simulation_settings.seperation_factor
         + turn_dv.normalize_or_zero() * simulation_settings.turn_factor
         + (alignment_dv - boid.0.velocity).normalize_or_zero() * simulation_settings.alignment_factor
         + (cohesion_position - boid.0.position).normalize_or_zero() * simulation_settings.cohesion_factor);
    }
    
    for (mut boid, change) in boid_query.iter_mut().zip(changes.into_iter()) 
    {
        boid.0.velocity += change;
    }
}


// Calculate the new position of the boids
fn apply_velocity(mut boid_query: Query<&mut Movement>, time: Res<Time>, simulation_settings: Res<SimulationSettings>) 
{
    for mut boid in &mut boid_query
    {
        boid.velocity = boid.velocity.clamp_length(simulation_settings.min_speed, simulation_settings.max_speed);
        boid.position.x += boid.velocity.x;
        boid.position.y += boid.velocity.y;
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