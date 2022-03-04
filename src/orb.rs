use std::time::Duration;

use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::*;
use heron::prelude::*;
use rand::Rng;

use crate::game;

const ORB_RADIUS: f32 = 38.;


#[derive(Component)]
pub struct Orb;

pub fn spawn_orb(
    mut commands: Commands,
    mut random: ResMut<crate::Random>,
    windows: Res<Windows>,
    query: Query<&Transform, With<Orb>>,
) {
    let primary_window = windows.get_primary().unwrap();
    let window_width = primary_window.width();
    let window_height = primary_window.height();

    let mut orbs: Vec<Transform> = query.iter().cloned().collect();
    let orb_count = query.iter().count();

    let desired_orb_count = ((window_width * (window_height - 50.)) / (ORB_RADIUS * 7.).powf(2.)).floor() as usize;
    let num_of_orbs_needed =  desired_orb_count - orb_count; 
    if num_of_orbs_needed == 0 {return}

    let shape = shapes::SvgPathShape {
        svg_doc_size_in_px: Vec2::new(ORB_RADIUS * 2., ORB_RADIUS * 2.),
        svg_path_string: "M 15 11 L 25 13 Q 30 14 31 12 L 36 2 Q 38 -2 40 2 L 45 12 Q 46 14 51 13 L 62 11 Q 66 10 65 14 L 63 25 Q 62 30 64 31 L 74 36 Q 78 38 74 40 L 64 45 Q 62 46 63 50 L 65 61 Q 66 66 61 65 L 51 63 Q 46 62 45 64 L 40 74 Q 38 78 36 74 L 31 64 Q 30 62 25 63 L 15 65 Q 10 66 11 61 L 13 51 Q 14 46 12 45 L 2 40 Q -2 38 2 36 L 12 31 Q 14 30 13 25 L 11 15 Q 10 10 15 11 Z".to_owned(),
    };
    let x_dist = window_width / 2.;
    let y_dist = window_height / 2. - 50.;

    for _ in 0..num_of_orbs_needed {
        let mut x = random.0.gen_range((-x_dist + ORB_RADIUS*2.)..(x_dist - ORB_RADIUS*2.));
        let mut y = random.0.gen_range((-y_dist + ORB_RADIUS*2.)..(y_dist - ORB_RADIUS*2.));

        while orbs.iter().any(|orb| (orb.translation - Vec3::new(x, y, 1.)).length() < ORB_RADIUS*4.) {
            x = random.0.gen_range((-x_dist + ORB_RADIUS*2.)..(x_dist - ORB_RADIUS*2.));
            y = random.0.gen_range((-y_dist + ORB_RADIUS*2.)..(y_dist - ORB_RADIUS*2.));
        }

        let transform = Transform {
            translation: Vec3::new(x, y, 1.),
            scale: Vec3::new(0., 0., 1.),
            ..Default::default()
        };

        let rotation_dir = random.0.gen_range((-1. as f32)..(1. as f32)).signum();

        let rotation_tween = Tween::new(
            EaseMethod::Linear,
            TweeningType::Loop,
            Duration::from_millis(2000),
            lens::TransformRotationLens {
                start: Quat::from_rotation_z(0.),
                end: Quat::from_rotation_z(rotation_dir * 6.28),
            },
        );
        let scale_tween = Tween::new(
            EaseFunction::QuadraticOut,
            TweeningType::Once,
            Duration::from_millis(300),
            lens::TransformScaleLens {
                start: Vec3::new(0., 0., 1.),
                end: Vec3::new(1., 1., 1.),
            },
        );

        commands.spawn_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Outlined {
                outline_mode: StrokeMode::new(game::ORB_OUTLINE_COLOR, 5.),
                fill_mode: FillMode::color(game::ORB_FILL_COLOR)
            },
            transform,
        ))
        .insert(Animator::new(scale_tween.then(rotation_tween)))
        .insert(RigidBody::Static)
        .insert(CollisionShape::Sphere{radius: ORB_RADIUS})
        .insert(Orb)
        .insert(game::GameEntity);

        orbs.push(transform);
    }
}
