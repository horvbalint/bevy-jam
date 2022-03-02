use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;
use heron::prelude::*;
use rand::Rng;

const ORB_COUNT: usize = 10;
const ORB_RADIUS: f32 = 20.;

pub struct OrbPlugin;

impl Plugin for OrbPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(spawn_orb);
    }
}

#[derive(Component)]
pub struct Orb;

fn spawn_orb(
    mut commands: Commands,
    mut random: ResMut<crate::Random>,
    windows: Res<Windows>,
    query: Query<&Transform, With<Orb>>,
) {
    let mut orbs: Vec<Transform> = query.iter().cloned().collect();
    let orb_count = query.iter().count();

    let num_of_orbs_needed = ORB_COUNT - orb_count; 
    if num_of_orbs_needed == 0 {return}

    let shape = shapes::Circle {
        radius: ORB_RADIUS,
        center: Vec2::new(0., 0.),
    };
    let primary_window = windows.get_primary().unwrap();
    let x_dist = primary_window.width() / 2.;
    let y_dist = primary_window.height() / 2. -100.;

    for _ in 0..num_of_orbs_needed {
        let mut x = random.0.gen_range((-x_dist + ORB_RADIUS*2.)..(x_dist - ORB_RADIUS*2.));
        let mut y = random.0.gen_range((-y_dist + ORB_RADIUS*2.)..(y_dist - ORB_RADIUS*2.));

        while orbs.iter().any(|orb| (orb.translation - Vec3::new(x, y, 1.)).length() < ORB_RADIUS*8.) {
            x = random.0.gen_range((-x_dist + ORB_RADIUS*2.)..(x_dist - ORB_RADIUS*2.));
            y = random.0.gen_range((-y_dist + ORB_RADIUS*2.)..(y_dist - ORB_RADIUS*2.));
        }

        let transform = Transform {
            translation: Vec3::new(x, y, 1.),
            ..Default::default()
        };

        let color = Color::GRAY;
        commands.spawn_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Fill(FillMode::color(color)),
            transform,
        ))
        .insert(RigidBody::Static)
        .insert(CollisionShape::Sphere{radius: ORB_RADIUS})
        .insert(Orb);

        orbs.push(transform);
    }
}
