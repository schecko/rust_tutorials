use bevy::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
    y : f32
} 

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);

struct HelloPlugin;

impl Plugin for HelloPlugin
{
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_startup_system(add_people)
            .add_system(greet_people); 
    }
}

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("hello".to_string())));
    commands.spawn((Person, Name("world".to_string())));
    commands.spawn((Person, Name("bruh".to_string()))); 
}

fn greet_people(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in query.iter() {
            println!("hello {}", name.0);
        }
    }
}

fn main() {
    println!("Hello, world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(HelloPlugin)
        .run();
}
