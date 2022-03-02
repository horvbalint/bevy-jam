use bevy::{prelude::*, utils::Duration};
use bevy_prototype_lyon::prelude::*;
use heron::prelude::*;
use bevy_tweening::*;
use crate::{bullet, Layer};

const LIN_VEL: f32 = 20.;
const ANG_VEL: f32 = 3.5;
const DUMP_VEL: f32 = 8.;
const DASH_VEL: f32 = 20.;
const DASH_DUR: f32 = 0.1;
const COOLDOWN_DUR: f32 = 1.0;

struct DashTimer(Timer);
impl DashTimer {
    pub fn new(seconds: f32) -> Self {Self(Timer::from_seconds(seconds, false))}
}

struct CooldownTimer(Timer);
impl CooldownTimer {
    pub fn new(seconds: f32) -> Self {Self(Timer::from_seconds(seconds, false))}
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(DashTimer::new(DASH_DUR))
            .insert_resource(CooldownTimer::new(COOLDOWN_DUR))
            .add_system(component_animator_system::<DrawMode>)
            .add_startup_system(spawn_players)
            .add_system(handle_move_inputs.label("input"))
            .add_system(move_players.label("move").after("input"))
            .add_system(handle_action_button_for_runner.after("move").label("action"))
            .add_system(handle_action_button_for_tagger.after("move").label("action"))
            .add_system(handle_dash_timer_for_runner)
            .add_system(handle_cooldown_timer_for_runner);
    }
}

#[derive(Component)]
pub struct Dash;

#[derive(Component)]
pub struct Cooldown;

#[derive(Component)]
pub struct Tagger;

#[derive(Component)]
pub struct Runner;

#[derive(Component)]
pub struct Player1;

pub struct Keys {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    action: KeyCode,
}

#[derive(Component)]
pub struct Player {
    pub velocity: f32,
    dir_vec: Vec3,
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

struct DrawColorLens {
    start: Vec3, // Vec3 because it can be subtracted and mulitplied
    end: Vec3, // Vec3 because it can be subtracted and mulitplied
}

impl Lens<DrawMode> for DrawColorLens {
    fn lerp(&mut self, target: &mut DrawMode, ratio: f32) -> () {
        if let DrawMode::Fill(ref mut fill_mode) = *target {
            let diff = (self.end - self.start) * ratio;
            let curr = self.start + diff;

            fill_mode.color = Color::rgb(curr.x, curr.y, curr.z);
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    windows: Res<Windows>,
) {
    // DEFINING SHARED PLAYER PROPERTIES
    let shape = shapes::Polygon {
        points: vec![
            Vec2::new(-15., -15.),
            Vec2::new(15., -15.),
            Vec2::new(0., 25.),
        ],
        closed: true,
    };

    let primary_window = windows.get_primary().unwrap();
    let y_pos = 30. - (primary_window.height() / 2.);
    let x_dist = primary_window.width() / 4.;
    
    // SPAWNING PLAYER 1
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(FillMode::color(Color::rgb(181./255., 90./255., 214./255.))),
        Transform {
            translation: Vec3::new(-x_dist, y_pos, 1.),
            ..Default::default()
        },
    ))
    .insert(RigidBody::KinematicPositionBased)
    .insert(CollisionShape::Sphere{radius: 20.})
    .insert(CollisionLayers::new(Layer::Tagger, Layer::Tagger))
    .insert(Player::new(Keys {
        up: KeyCode::W,
        down: KeyCode::S,
        left: KeyCode::A,
        right: KeyCode::D,
        action: KeyCode::Space,
    }))
    .insert(Tagger);

    // SPAWNING PLAYER 2
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(FillMode::color(Color::BLUE)),
        Transform {
            translation: Vec3::new(x_dist, y_pos, 1.),
            ..Default::default()
        },
    ))
    .insert(RigidBody::KinematicPositionBased)
    .insert(CollisionShape::Sphere{radius: 20.})
    .insert(CollisionLayers::new(Layer::Runner, Layer::Runner))
    .insert(Player::new(Keys {
        up: KeyCode::Up,
        down: KeyCode::Down,
        left: KeyCode::Left,
        right: KeyCode::Right,
        action: KeyCode::Return,
    }))
    .insert(Runner);
}

fn handle_move_inputs(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Player, Option<&Dash>)>,
) {
    for (mut player, is_dashing) in query.iter_mut() {
        if is_dashing.is_none() {
            // Handling forwards/backwards movement
            if keys.pressed(player.keys.up) {
                player.velocity += LIN_VEL * time.delta_seconds();
            }
            if keys.pressed(player.keys.down) {
                player.velocity += -LIN_VEL * time.delta_seconds();
            }
    
            // Slowing the player down if not moving
            if !keys.pressed(player.keys.up) && !keys.pressed(player.keys.down) {
                player.velocity *= 1. - (DUMP_VEL * time.delta_seconds())
            }

            // Making sure it doesn't accelerate to high
            player.velocity = player.velocity.clamp(-5., 5.);
        }

        // Handling right/left movement
        if keys.pressed(player.keys.right) {
            player.rotation -= ANG_VEL * time.delta_seconds();
        }
        if keys.pressed(player.keys.left) {
            player.rotation += ANG_VEL * time.delta_seconds();
        }
    }
}


fn handle_action_button_for_tagger(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    query: Query<(Entity, &Player, &Transform), With<Tagger>>,
) {
    let (entity, player, transform) = query.single();

    if keys.just_pressed(player.keys.action) {
        bullet::spawn_bullet(
            &mut commands,
            transform.translation.clone(),
            get_direction_vec(player.rotation),
            entity
        );
    }
}

fn handle_action_button_for_runner(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut query: Query<(Entity, &mut Player), (With<Runner>, Without<Dash>, Without<Cooldown>)>,
) {
    if let Ok((entity, mut player)) = query.get_single_mut() {        
        if keys.just_pressed(player.keys.action) {
            commands.entity(entity).insert(Dash);
            player.velocity = DASH_VEL;


            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::PingPong,
                Duration::from_millis((DASH_DUR*500.) as u64),
                lens::TransformScaleLens {
                    start: Vec3::new(1., 1., 1.),
                    end: Vec3::new(0.2, 1., 1.),
                },
            );
            commands.entity(entity).insert(Animator::new(tween));

            let tween = Tween::new(
                EaseFunction::CircularIn,
                TweeningType::Once,
                Duration::from_secs((COOLDOWN_DUR + DASH_DUR) as u64),
                DrawColorLens {
                    start: Vec3::new(1., 1., 1.),
                    end: Vec3::new(0., 0., 1.,)
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
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
        player.dir_vec = get_direction_vec(player.rotation);

        transform.translation += player.dir_vec * player.velocity;
        
        if transform.translation.x.abs() > x_dist {
            transform.translation.x = transform.translation.x.signum() * x_dist;
        }
        if transform.translation.y < -y_dist {
            transform.translation.y = -y_dist;
        }
        else if transform.translation.y > y_dist-50. {
            transform.translation.y = y_dist-50.;
        }

        transform.rotation = Quat::from_rotation_z(player.rotation);
    }
}

fn handle_dash_timer_for_runner(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<DashTimer>,
    query: Query<Entity, With<Dash>>
) { 
    if let Ok(entity) = query.get_single() {
        timer.0.tick(time.delta());
    
        if timer.0.just_finished() {
            commands.entity(entity).insert(Cooldown);
            commands.entity(entity).remove::<Dash>();
            commands.entity(entity).remove::<Animator<Transform>>();
            timer.0.reset();
        }
    }
}

fn handle_cooldown_timer_for_runner(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<CooldownTimer>,
    query: Query<Entity, With<Cooldown>>
) { 
    if let Ok(entity) = query.get_single() {
        timer.0.tick(time.delta());
    
        if timer.0.just_finished() {
            commands.entity(entity).remove::<Cooldown>();
            commands.entity(entity).remove::<Animator<DrawMode>>();
            timer.0.reset();
        }
    }
}

fn get_direction_vec(angle: f32) -> Vec3 {
    let x = -angle.sin();
    let y = angle.cos();

    Vec3::new(x, y, 0.)
}