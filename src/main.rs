use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;
use heron::prelude::*;
use bevy_tweening::*;
use player::Tagger;
use rand::{rngs::StdRng, SeedableRng, prelude::SliceRandom};

mod player;
mod bullet;
mod orb;

pub struct ColorPalette(Vec<Color>);
pub struct Random(StdRng);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Colorace".to_string(),
            width: 1000.,
            height: 1000.,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(9./255., 8./255., 28./255.)))
        .insert_resource(ColorPalette(vec![
            Color::rgb(86./255., 176./255., 209./255.), //BLUE
            Color::rgb(230./255., 160./255., 80./255.), // ORANGE
            Color::rgb(227./255., 77./255., 77./255.), // RED
            Color::rgb(95./255., 194./255., 113./255.), // GREEN
            Color::rgb(181./255., 90./255., 214./255.), // PURPLE
        ]))
        .insert_resource(Random(StdRng::from_entropy()))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(bullet::BulletPlugin)
        .add_plugin(orb::OrbPlugin)
        .add_startup_system(setup)
        .add_system(detect_collisions.after("action"))
        .run();
}

#[derive(PhysicsLayer)]
enum Layer {
    Tagger,
    Runner,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    
    // MENU
    let primary_window = windows.get_primary().unwrap();
    let x_pos = primary_window.width() / 2.;
    let y_pos = primary_window.height() / 2.;

    let color = Color::BLACK;
    let shape = shapes::Rectangle {
        extents: Vec2::new(primary_window.width(), 50.),
        origin: RectangleOrigin::TopLeft
    };

    let font = asset_server.load("Roboto-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 30.,
        color: Color::WHITE,
        ..Default::default()
    };

    commands.spawn_bundle(GeometryBuilder::build_as(
        &shape,
        DrawMode::Fill(FillMode::color(color)),
        Transform {
            translation: Vec3::new(-x_pos, y_pos, 2.),
            ..Default::default()
        },
    ));
    
    // Player1 point
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section("Player1: 0", text_style.clone(), TextAlignment::default()),
        transform: Transform {
            translation: Vec3::new(-x_pos+10., y_pos-10., 3.),
            ..Default::default()
        },
        ..Default::default()
    })
        .insert(player::Player1);

    // Player2 point
    commands.spawn_bundle(Text2dBundle {
        text: Text::with_section("Player2: 0", text_style, TextAlignment {
            horizontal: HorizontalAlign::Right, 
            ..Default::default()
        }),
        transform: Transform {
            translation: Vec3::new(x_pos-10., y_pos-10., 3.),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn detect_collisions(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut get_player: Query<&mut player::Player>,
    mut get_runner: Query<(&mut DrawMode, Option<&player::Dash>), (With<player::Player>, Without<Tagger>)>,
    mut get_tagger: Query<&mut DrawMode, (With<player::Player>, With<Tagger>)>,
    get_bullet: Query<&bullet::Bullet, With<bullet::Bullet>>,
    get_orb: Query<(), (With<orb::Orb>, Without<player::Player>)>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(data1, data2) = event {
            let entity1 = data1.rigid_body_entity();
            let entity2 = data2.rigid_body_entity();

            if let Ok(mut player) = get_player.get_mut(entity1) {
                handle_player_collision(&mut commands, entity1, &mut player, entity2, &mut get_runner, &mut get_tagger, &get_bullet, &get_orb);
            }
            else if let Ok(mut player) = get_player.get_mut(entity2) {
                handle_player_collision(&mut commands, entity2, &mut player, entity1, &mut get_runner, &mut get_tagger, &get_bullet, &get_orb);
            }
            else if let Ok(_) = get_orb.get(entity1) {
                commands.entity(entity2).despawn()
            }
            else if let Ok(_) = get_bullet.get(entity1) {
                commands.entity(entity2).despawn()
            }
        }
    }
}

fn handle_player_collision(
    commands: &mut Commands,
    player_entity: Entity, 
    player: &mut player::Player,
    other_entity: Entity,
    get_runner: &mut Query<(&mut DrawMode, Option<&player::Dash>), (With<player::Player>, Without<Tagger>)>,
    get_tagger: &mut Query<&mut DrawMode, (With<player::Player>, With<Tagger>)>,
    get_bullet: &Query<&bullet::Bullet, With<bullet::Bullet>>,
    get_orb: &Query<(), (With<orb::Orb>, Without<player::Player>)>,
) {
    if let Ok((mut runner_draw_mode, is_dashing)) = get_runner.get_mut(player_entity) {
        if is_dashing.is_none() {
            handle_runner_collision(commands, player_entity, player, other_entity, &mut runner_draw_mode, get_tagger, get_bullet, get_orb);
        }
    }
    else if let Ok(_) = get_tagger.get_mut(player_entity) {
        handle_tagger_collision(commands, player, other_entity, get_orb);
    }
}

fn handle_runner_collision(
    commands: &mut Commands,
    player_entity: Entity, 
    player: &mut player::Player,
    other_entity: Entity,
    draw_mode: &mut DrawMode,
    get_tagger: &mut Query<&mut DrawMode, (With<player::Player>, With<Tagger>)>,
    get_bullet: &Query<&bullet::Bullet, With<bullet::Bullet>>,
    get_orb: &Query<(), (With<orb::Orb>, Without<player::Player>)>,
) {
    if let Ok(bullet) = get_bullet.get(other_entity) {
        if let DrawMode::Fill(ref mut runner_fill_mode) = *draw_mode {
            if let Ok(mut tagger_draw_mode) = get_tagger.get_mut(bullet.shooter) {
                if let DrawMode::Fill(ref mut tagger_fill_mode) = *tagger_draw_mode {
                    runner_fill_mode.color = Color::rgb(181./255., 90./255., 214./255.);
                    tagger_fill_mode.color = Color::rgb(0., 0., 1.);
            
                    commands.entity(player_entity).remove::<player::Runner>();
                    commands.entity(player_entity).remove::<player::Dash>();
                    commands.entity(player_entity).remove::<player::Cooldown>();
                    commands.entity(player_entity).remove::<Animator<Transform>>();
                    commands.entity(player_entity).remove::<Animator<DrawMode>>();
                    commands.entity(player_entity).remove::<CollisionLayers>();
                    commands.entity(player_entity).insert(player::Tagger);
                    commands.entity(player_entity).insert(CollisionLayers::new(Layer::Tagger, Layer::Tagger));
                    
                    commands.entity(bullet.shooter).remove::<player::Tagger>();
                    commands.entity(bullet.shooter).remove::<CollisionLayers>();
                    commands.entity(bullet.shooter).insert(player::Runner);
                    commands.entity(bullet.shooter).insert(CollisionLayers::new(Layer::Runner, Layer::Runner));
            
                    commands.entity(other_entity).despawn();
                }
            }
        }
    }
    else if let Ok(_) = get_orb.get(other_entity) {
        player.velocity = -5.;
        commands.entity(other_entity).despawn();
    }
}

fn handle_tagger_collision(
    commands: &mut Commands,
    player: &mut player::Player,
    other_entity: Entity,
    get_orb: &Query<(), (With<orb::Orb>, Without<player::Player>)>,
) {
    if let Ok(_) = get_orb.get(other_entity) {
        player.velocity = -5.;
        commands.entity(other_entity).despawn();
    }
}

// fn change_player1_score(
//     mut text_query: Query<&mut Text, With<player::Player1>>,
//     player_query: Query<&player::Player, With<player::Player1>>,
// ) {
//     let mut text = text_query.single_mut();
//     let player = player_query.single();

//     // text.sections.first_mut().unwrap().value = format!("Player1: {}", player.score);
// }