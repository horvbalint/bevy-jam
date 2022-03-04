use bevy::{prelude::*, utils::Duration};
use bevy_kira_audio::Audio;
use bevy_prototype_lyon::prelude::*;
use heron::prelude::*;
use bevy_tweening::*;
use crate::bullet;
use crate::game;

pub const MAX_SPEED: f32 = 400.;
pub const LIN_VEL: f32 = 1600.;
pub const ANG_VEL: f32 = 3.5;
pub const DUMP_VEL: f32 = 8.;
pub const DASH_SPEED: f32 = 1600.;
pub const DASH_DUR: f32 = 0.1;
pub const COOLDOWN_DUR: f32 = 1.0;

pub struct DashTimer(pub Timer);
impl DashTimer {
    pub fn new(seconds: f32) -> Self {Self(Timer::from_seconds(seconds, false))}
}

pub struct CooldownTimer(pub Timer);
impl CooldownTimer {
    pub fn new(seconds: f32) -> Self {Self(Timer::from_seconds(seconds, false))}
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

#[derive(Component)]
pub struct Player2;

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

pub fn spawn_players(
    mut commands: Commands,
    windows: Res<Windows>,
) {
    // DEFINING SHARED PLAYER PROPERTIES
    let shape = shapes::SvgPathShape {
        svg_doc_size_in_px: Vec2::new(31.7, 44.),
        svg_path_string: "M 1.2681 34.1401 Q -1.6782 43.8592 4.2145 43.8592 L 27.7855 43.8592 Q 33.6782 43.8592 30.7319 34.1401 L 21.8927 8.2224 Q 16 -7.9761 10.1073 8.2224".to_owned()
    };

    let primary_window = windows.get_primary().unwrap();
    let y_pos = 30. - (primary_window.height() / 2.);
    let x_dist = primary_window.width() / 4.;
    
    // SPAWNING PLAYER 1
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(FillMode::color(game::TAGGER_COL)),
        Transform {
            translation: Vec3::new(-x_dist, y_pos, 1.),
            ..Default::default()
        },
    ))
    .insert(RigidBody::KinematicPositionBased)
    .insert(CollisionShape::Capsule{radius: 10., half_segment: 12.})
    .insert(CollisionLayers::new(game::Layer::Tagger, game::Layer::Tagger))
    .insert(Player::new(Keys {
        up: KeyCode::W,
        down: KeyCode::S,
        left: KeyCode::A,
        right: KeyCode::D,
        action: KeyCode::Space,
    }))
    .insert(Tagger)
    .insert(Player1)
    .insert(game::GameEntity);

    // SPAWNING PLAYER 2
    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(FillMode::color(game::RUNNER_COL)),
        Transform {
            translation: Vec3::new(x_dist, y_pos, 1.),
            ..Default::default()
        },
    ))
    .insert(RigidBody::KinematicPositionBased)
    .insert(CollisionShape::Capsule{radius: 10., half_segment: 10.})
    .insert(CollisionLayers::new(game::Layer::Runner, game::Layer::Runner))
    .insert(Player::new(Keys {
        up: KeyCode::Up,
        down: KeyCode::Down,
        left: KeyCode::Left,
        right: KeyCode::Right,
        action: KeyCode::Return,
    }))
    .insert(Runner)
    .insert(Player2)
    .insert(game::GameEntity);
}

pub fn handle_move_inputs(
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
            player.velocity = player.velocity.clamp(-MAX_SPEED, MAX_SPEED);
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


pub fn handle_action_button_for_tagger(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    audio: Res<Audio>,
    shoot_sound: Res<crate::ShootSound>,
    query: Query<(Entity, &Player, &Transform), With<Tagger>>,
) {
    let (entity, player, transform) = query.single();

    if keys.just_pressed(player.keys.action) {
        audio.play(shoot_sound.0.clone());
        
        bullet::spawn_bullet(
            &mut commands,
            transform.translation.clone(),
            get_direction_vec(player.rotation),
            entity
        );
    }
}

pub fn handle_action_button_for_runner(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    audio: Res<Audio>,
    dash_sound: Res<crate::DashSound>,
    mut query: Query<(Entity, &mut Player), (With<Runner>, Without<Dash>, Without<Cooldown>)>,
) {
    if let Ok((entity, mut player)) = query.get_single_mut() {        
        if keys.just_pressed(player.keys.action) {
            commands.entity(entity).insert(Dash);
            player.velocity = DASH_SPEED;
            audio.play(dash_sound.0.clone());

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
                    end: Vec3::new(107./255., 186./255., 93./255.)
                },
            );
            commands.entity(entity).insert(Animator::new(tween));
        }
    }
}

pub fn move_players(
    mut query: Query<(&mut Transform, &mut Player)>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    let primary_window = windows.get_primary().unwrap();
    let x_dist = primary_window.width() / 2.;
    let y_dist = primary_window.height() / 2.;

    for (mut transform, mut player) in query.iter_mut() {
        player.dir_vec = get_direction_vec(player.rotation);

        transform.translation += player.dir_vec * player.velocity * time.delta_seconds();
        
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

pub fn handle_dash_timer_for_runner(
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

pub fn handle_cooldown_timer_for_runner(
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

pub fn get_direction_vec(angle: f32) -> Vec3 {
    let x = -angle.sin();
    let y = angle.cos();

    Vec3::new(x, y, 0.)
}