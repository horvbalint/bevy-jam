use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;
use heron::prelude::*;
use crate::Layer;

const BULLET_RADIUS: f32 = 5.;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(move_bullets.label("move").after("input"));
    }
}

#[derive(Component)]
pub struct Bullet {
    pub shooter: Entity,
    dir_vec: Vec3,
    radius: f32,
    speed: f32,
}

impl Bullet {
    pub fn new(dir_vec: Vec3, shooter: Entity) -> Self {
        Self {
            dir_vec,
            radius: BULLET_RADIUS,
            speed: 10.,
            shooter,
        }
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

        if transform.translation.x.abs() >= x_dist + bullet.radius {
            commands.entity(entity).despawn();
        }
        else if transform.translation.y.abs() >= y_dist + bullet.radius {
            commands.entity(entity).despawn();
        }
    }
}

// HELPER FUNCTIONS
pub fn spawn_bullet(
    commands: &mut Commands,
    pos: Vec3,
    dir_vec: Vec3,
    shooter: Entity
) {
    let mut bullet = commands.spawn_bundle(GeometryBuilder::build_as(
        &shapes::Circle{
            radius: BULLET_RADIUS,
            center: Vec2::new(0., 0.),
        },
        DrawMode::Fill(FillMode::color(Color::rgb(181./255., 90./255., 214./255.))),
        Transform {
            translation: Vec3::new(pos.x, pos.y, 0.),
            ..Default::default()
        },
    ));
    bullet.insert(Bullet::new(dir_vec, shooter))
    .insert(RigidBody::KinematicPositionBased)
    .insert(CollisionShape::Sphere{radius: BULLET_RADIUS})
    .insert(CollisionLayers::new(Layer::Runner, Layer::Runner));
}