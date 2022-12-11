use bevy::prelude::*;
use bevy::sprite::collide_aabb::{ collide, Collision };
use bevy::sprite::MaterialMesh2dBundle;
use bevy::time::FixedTimestep;

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PADDLE_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
// These values are exact
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: f32 = 5.0;
// These values are lower bounds, as the number of bricks is computed
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Brick;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

fn setup
(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
)
{
    commands.spawn(Camera2dBundle::default());

    // paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;
    commands.spawn
    ((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new( 0.0, paddle_y, 0.0 ),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default() 
            },
            ..default()
        },
        Paddle
        //Collider,
    ));

    // ball
    commands.spawn
    ((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED)
    ));

    // bricks
    {
        let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
        let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
        let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

        let n_col = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
        let n_row = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
        let n_vert_gaps = n_col - 1;

        let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
        let left_edge_of_bricks = center_of_bricks
            - (n_col as f32 / 2.0 * BRICK_SIZE.x)
            - (n_vert_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS);
        
        let offset = Vec2::new(left_edge_of_bricks, bottom_edge_of_bricks) + BRICK_SIZE / Vec2::splat(2.0);

        for row in 0..n_row {
            for col in 0..n_col {
                let pos = offset + Vec2::new(col as f32, row as f32) * ( BRICK_SIZE + Vec2::splat(GAP_BETWEEN_BRICKS) );

                commands.spawn
                ((
                    SpriteBundle{
                        sprite: Sprite {
                            color: BRICK_COLOR,
                            ..default()
                        },
                        transform: Transform {
                            translation: pos.extend(0.0),
                            scale: BRICK_SIZE.extend(1.0),
                            ..default()
                        },
                        ..default()
                    },
                    Brick,
                    //Collider
                ));
            }
        }
        
    }
}

fn move_paddle
(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>
)
{
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard.pressed(KeyCode::Left) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    let new_paddle_pos = paddle_transform.translation.x + direction * PADDLE_SPEED * TIME_STEP;
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;
    paddle_transform.translation.x = new_paddle_pos.clamp(left_bound, right_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for ( mut trans, vel ) in &mut query {
        trans.translation += vel.0.extend(0.0) * Vec3::splat(TIME_STEP);
    }
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(move_paddle)
        .add_system(apply_velocity)
        .run();
}

