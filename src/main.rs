use std::f32::consts::PI;

use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Colorace".to_string(),
            width: 800.,
            height: 1000.,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(9./255., 8./255., 28./255.)))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(setup)
        .add_system(handle_player_inputs.label("input"))
        .add_system(move_players.label("move").after("input"))
        .add_system(move_bullets.label("move").after("input"))
        .add_system(check_bullet_collisions_player_1.after("move"))
        .add_system(check_bullet_collisions_player_2.after("move"))
        .run();
}

#[derive(Component)]
struct Bullet {
    dir_vec: Vec3,
    radius: f32,
    speed: f32,
}

impl Bullet {
    pub fn new(dir_vec: Vec3, radius: Option<f32>, speed: Option<f32>) -> Self {
        Self { 
            dir_vec,
            radius: radius.unwrap_or(5.),
            speed: radius.unwrap_or(10.),
        }
    }
}

#[derive(Component)]
struct Player1();

#[derive(Component)]
struct Player2();

struct Keys {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    shoot: KeyCode,
}

#[derive(Component)]
struct Player {
    dir_vec: Vec3,
    velocity: f32,
    rotation: f32,
    keys: Keys,
}

impl Player {
    pub fn new(keys: Keys) -> Self {
        Self {
            dir_vec: Vec3::ZERO,
            velocity: 0.,
            rotation: 0.,
            keys,
        }
    }
}

fn setup(
    mut commands: Commands,
    windows: Res<Windows>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // DEFINING SHARED PLAYER PROPERTIES
    let shape = shapes::Polygon {
        points: vec![
            Vec2::new(-15., -15.),
            Vec2::new(15., -15.),
            Vec2::new(0., 25.),
        ],
        closed: true,
    };

    
    let draw_mode = DrawMode::Fill(FillMode::color(Color::rgb(209./255., 84./255., 255./255.)));
    let primary_window = windows.get_primary().unwrap();
    let y_pos = 30. - (primary_window.height() / 2.);
    let x_dist = primary_window.width() / 4.;

    // SPAWNING PLAYER 1
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        draw_mode,
        Transform {
            translation: Vec3::new(-x_dist, y_pos, 1.),
            ..Default::default()
        },
    ))
    .insert(Player::new(Keys {
        up: KeyCode::Up,
        down: KeyCode::Down,
        left: KeyCode::Left,
        right: KeyCode::Right,
        shoot: KeyCode::Return,
    }))
    .insert(Player1());

    // SPAWNING PLAYER 2
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        draw_mode,
        Transform {
            translation: Vec3::new(x_dist, y_pos, 1.),
            ..Default::default()
        },
    ))
    .insert(Player::new(Keys {
        up: KeyCode::W,
        down: KeyCode::S,
        left: KeyCode::A,
        right: KeyCode::D,
        shoot: KeyCode::Space,
    }))
    .insert(Player2());
}

fn handle_player_inputs(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Player, &Transform, Option<&Player1>)>,
) {
    for (mut player, transform, player1) in query.iter_mut() {
        // Handling forwards/backwards movement
        if keys.pressed(player.keys.up) {
            player.velocity += 30. * time.delta_seconds();
        }
        if keys.pressed(player.keys.down) {
            player.velocity += -30. * time.delta_seconds();
        }
    
        // Slowing the player down if not moving
        if !keys.pressed(player.keys.up) && !keys.pressed(player.keys.down) {
            player.velocity *= 1. - (8. * time.delta_seconds())
        }
    
        // Making sure it doesn't accelerate to high
        player.velocity = player.velocity.clamp(-5., 5.);
    
        // Handling right/left movement
        if keys.pressed(player.keys.right) {
            player.rotation -= 3. * time.delta_seconds();
        }
        if keys.pressed(player.keys.left) {
            player.rotation += 3. * time.delta_seconds();
        }

        // Handling shooting
        if keys.just_pressed(player.keys.shoot) {
            spawn_bullet(
                &mut commands,
                transform.translation.clone(),
                get_direction_vec(player.rotation),
                player1.is_some()
            )
        }
    }
}

fn move_players(
    mut query: Query<(&mut Transform, &mut Player)>,
    windows: Res<Windows>,
) {
    let primary_window = windows.get_primary().unwrap();
    let x_dist = primary_window.width() / 2.;
    let y_dist = primary_window.height() / 2.;

    for (mut transform, mut player) in query.iter_mut() {
        let dir = get_direction_vec(player.rotation);
        
        if transform.translation.x.abs() >= x_dist {
            transform.translation.x = transform.translation.x.signum() * (x_dist-5.);
            player.dir_vec = Vec3::new(-dir.x, dir.y, dir.z);
            player.rotation *= -1.;
        }
        else if transform.translation.y.abs() >= y_dist {
            transform.translation.y = transform.translation.y.signum() * (y_dist-5.);
            player.dir_vec = Vec3::new(dir.y, -dir.y, dir.z);
            player.rotation = ((player.rotation - PI / 2.) * -1.) + PI / 2.;
        }
        else {
            player.dir_vec = (player.dir_vec + dir).normalize();
        }

        transform.translation += player.dir_vec * player.velocity;
        transform.rotation = Quat::from_rotation_z(player.rotation);
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    pos: Vec3,
    dir_vec: Vec3,
    is_player_1: bool,
) {
    let mut bullet = commands.spawn_bundle(GeometryBuilder::build_as(
        &shapes::Circle{
            radius: 5.,
            center: Vec2::new(0., 0.),
        },
        DrawMode::Fill(FillMode::color(Color::rgb(116./255., 247./255., 118./255.))),
        Transform {
            translation: Vec3::new(pos.x, pos.y, 0.),
            ..Default::default()
        },
    ));
    bullet.insert(Bullet::new(dir_vec, None, None));
    
    if is_player_1 {
        bullet.insert(Player1());
    } else {
        bullet.insert(Player2());
    }
}

fn move_bullets(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Bullet)>,
    windows: Res<Windows>,
) {
    let primary_window = windows.get_primary().unwrap();
    let x_dist = primary_window.width() / 2.;
    let y_dist = primary_window.height() / 2.;

    for (entity, mut transform, bullet) in query.iter_mut() {
        transform.translation += bullet.dir_vec * bullet.speed;

        if transform.translation.x.abs() + bullet.radius >= x_dist {
            commands.entity(entity).despawn();
        }
        else if transform.translation.y.abs() + bullet.radius >= y_dist {
            commands.entity(entity).despawn();
        }
    }
}

fn check_bullet_collisions_player_1(
    mut commands: Commands,
    player_query: Query<(&Transform, &Player), With<Player1>>,
    bullet_query: Query<(Entity, &Transform, &Bullet), With<Player2>>,
) {
    let (player_tf, player) = player_query.single();
    let vec = get_direction_vec(player.rotation) * -1. * (15. - 6.8181);

    let player_trinalge_circle_pos = player_tf.translation + vec;

    for (entity, transform, bullet) in bullet_query.iter() {
        if (transform.translation - player_trinalge_circle_pos).length() <= (6.8181 + bullet.radius) {
            commands.entity(entity).despawn();
        } 
    }
}

fn check_bullet_collisions_player_2(
    mut commands: Commands,
    player_query: Query<(&Transform, &Player), With<Player2>>,
    bullet_query: Query<(Entity, &Transform, &Bullet), With<Player1>>,
) {
    let (player_tf, player) = player_query.single();
    let vec = get_direction_vec(player.rotation) * -1. * (15. - 6.8181);

    let player_trinalge_circle_pos = player_tf.translation + vec;

    for (entity, transform, bullet) in bullet_query.iter() {
        if (transform.translation - player_trinalge_circle_pos).length() <= (6.8181 + bullet.radius) {
            commands.entity(entity).despawn();
        } 
    }
}

fn get_direction_vec(angle: f32) -> Vec3 {
    let x = -angle.sin();
    let y = angle.cos();

    Vec3::new(x, y, 0.)
}