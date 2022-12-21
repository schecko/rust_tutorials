use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component, Default, Copy, Clone, PartialEq, Debug)]
struct Player {
    velocity: Vec2,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // player
    let size = Vec2::new(25.0, 25.0);
    commands.spawn((
        Player::default(),
        Collider::cuboid(size.x / 2.0, size.y / 2.0),
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController {
            snap_to_ground: None, //Some(CharacterLength::Absolute(0.5)),
            apply_impulse_to_dynamic_bodies: true,
            up: Vec2::Y,
            offset: CharacterLength::Relative(0.1),
            slide: true,
            max_slope_climb_angle: std::f32::consts::PI / 2.0,
            min_slope_slide_angle: 0.0,
            ..default()
        },
        TransformBundle::from(Transform::from_xyz(0.0, 100.0, 0.0)),
    ));

    // ground
    commands.spawn((
        Collider::cuboid(500.0, 25.0),
        RigidBody::Fixed,
        TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)),
    ));

    // side
    commands.spawn((
        Collider::cuboid(25.0, 500.0),
        RigidBody::Fixed,
        TransformBundle::from(Transform::from_xyz(500.0, 0.0, 0.0)),
    ));
}

fn player_kinematics(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    rapier_config: Res<RapierConfiguration>,
    mut controller_query: Query<(
        &mut Player,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
) {
    if controller_query.is_empty() {
        return;
    }
    let (mut player, mut controller, controller_output) = controller_query.single_mut();
    let grounded = match controller_output {
        Some(output) => output.grounded,
        None => false,
    };

    let dt = time.delta_seconds();
    let speed = 40.0;
    let jump_impulse = 100.0;
    let mut instant_acceleration = Vec2::ZERO;
    let mut instant_velocity = player.velocity;

    // physics simulation
    if grounded {
        // friction
        instant_velocity.x *= 0.9;
    } else {
        // gravity
        instant_acceleration += Vec2::Y * rapier_config.gravity;
    }

    // input
    if keyboard.pressed(KeyCode::Left) {
        instant_velocity.x -= speed;
    }
    if keyboard.pressed(KeyCode::Right) {
        instant_velocity.x += speed;
    }

    if keyboard.pressed(KeyCode::Up) {
        instant_velocity.y += jump_impulse;
    }
    if keyboard.pressed(KeyCode::Down) {
        instant_velocity.y -= speed;
    }
    instant_velocity = instant_velocity.clamp(Vec2::splat(-1000.0), Vec2::splat(1000.0));

    player.velocity = (instant_acceleration * dt) + instant_velocity;
    controller.translation = Some(player.velocity * dt);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -2000.0),
            ..default()
        })
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_system(player_kinematics)
        .run();
}
