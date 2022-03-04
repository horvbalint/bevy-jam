use bevy::prelude::*;
use bevy_kira_audio::Audio;
use bevy_prototype_lyon::prelude::*;
use bevy_tweening::*;
use heron::prelude::*;

use crate::GameState;
use crate::{player, bullet, orb};

pub const GAME_DUR: f32 = 90.;
pub const TAGGER_COL: Color = Color::rgb(94./255., 165./255., 255./255.);
pub const RUNNER_COL: Color = Color::rgb(107./255., 186./255., 93./255.);
pub const ORB_FILL_COLOR: Color = Color::rgb(181./255., 90./255., 214./255.1);
pub const ORB_OUTLINE_COLOR: Color = Color::rgb(138./255., 30./255., 97./255.);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Game)
                .with_system(setup_game)
                .with_system(player::spawn_players),
        )
        .add_system_set(
            SystemSet::on_resume(GameState::Game)
                .with_system(setup_game),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Game)
                .with_system(player::handle_move_inputs.label("input"))
                .with_system(component_animator_system::<DrawMode>.label("animate").after("input"))
                .with_system(player::move_players.label("move").after("input"))
                .with_system(player::handle_action_button_for_runner.after("move").label("action"))
                .with_system(player::handle_action_button_for_tagger.after("move").label("action"))
                .with_system(player::handle_dash_timer_for_runner)
                .with_system(player::handle_cooldown_timer_for_runner)
                .with_system(bullet::move_bullets.label("move").after("input"))
                .with_system(orb::spawn_orb)
                .with_system(detect_collisions.after("action").after("animate"))
                .with_system(tick_count_down_timer)
        )
        .add_system_set(
            SystemSet::on_pause(GameState::Game)
                .with_system(teardown_game),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::Game)
                .with_system(teardown_game),
        );
    }
}

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
struct Countdown;

#[derive(Component)]
pub struct TopBar;

#[derive(PhysicsLayer)]
pub enum Layer {
    Tagger,
    Runner,
}

pub struct CountDownTimer(Timer);
impl CountDownTimer {
    pub fn new(seconds: f32) -> Self {Self(Timer::from_seconds(seconds, false))}
}

pub fn setup_game(
  mut commands: Commands,
  font: Res<crate::FontHandle>,
) {
    commands.insert_resource(CountDownTimer::new(GAME_DUR));
    commands.insert_resource(player::DashTimer::new(player::DASH_DUR));
    commands.insert_resource(player::CooldownTimer::new(player::COOLDOWN_DUR));

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
            },
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: Rect { 
                left: Val::Px(10.),
                right: Val::Px(10.),
                ..Default::default()
            },
            position: Rect {
                top: Val::Px(0.),
                ..Default::default()
            },
            ..Style::default()
        },
        color: UiColor(Color::rgb(23./255., 23./255., 23./255.)),
        visibility: Visibility {
            is_visible: true,
        },
        ..NodeBundle::default()
    })
    .insert(TopBar)
    .with_children(|parent| {
        // Player1
        let text_style = TextStyle {
            font: font.0.clone(),
            font_size: 30.,
            color: TAGGER_COL,
            ..Default::default()
        };
        
        parent.spawn_bundle(TextBundle {
            text: Text::with_section("Player_1", text_style, TextAlignment::default()),
            ..Default::default()
        })
        .insert(player::Player1);

        // Timer
        let text_style = TextStyle {
            font: font.0.clone(),
            font_size: 40.,
            color: Color::WHITE,
            ..Default::default()
        };

        parent.spawn_bundle(TextBundle {
            text: Text::with_section("01:00", text_style, TextAlignment::default()),
            ..Default::default()
        })
        .insert(Countdown);


        // Player2
        let text_style = TextStyle {
            font: font.0.clone(),
            font_size: 30.,
            color: RUNNER_COL,
            ..Default::default()
        };

        parent.spawn_bundle(TextBundle {
            text: Text::with_section("Player_2", text_style, TextAlignment::default()),
            ..Default::default()
        })
        .insert(player::Player2);
    });
}

pub fn teardown_game(
    mut commands: Commands,
    top_bar: Query<Entity, With<TopBar>>,
    game_entity_query: Query<Entity, With<GameEntity>>
) {
    let top_bar_entity = top_bar.single();
    commands.entity(top_bar_entity).despawn_recursive();

    for entity in game_entity_query.iter() {
        commands.entity(entity).despawn();
    }
}


fn detect_collisions(
  mut commands: Commands,
  mut events: EventReader<CollisionEvent>,
  mut dash_timer: ResMut<player::DashTimer>,
  mut cooldown_timer: ResMut<player::CooldownTimer>,
  mut get_player: Query<&mut player::Player>,
  mut get_runner: Query<(&mut DrawMode, Option<&player::Dash>), (With<player::Player>, Without<player::Tagger>)>,
  mut get_tagger: Query<&mut DrawMode, (With<player::Player>, With<player::Tagger>)>,
  mut get_player1_text: Query<&mut Text, (With<player::Player1>, Without<player::Player2>)>,
  mut get_player2_text: Query<&mut Text, (With<player::Player2>, Without<player::Player1>)>,
  get_bullet: Query<&bullet::Bullet, With<bullet::Bullet>>,
  get_orb: Query<(), (With<orb::Orb>, Without<player::Player>)>,
  audio: Res<Audio>,
  orb_sound: Res<crate::OrbSound>,
  catch_sound: Res<crate::CatchSound>,
) {
  for event in events.iter() {
      if let CollisionEvent::Started(data1, data2) = event {
          let entity1 = data1.rigid_body_entity();
          let entity2 = data2.rigid_body_entity();

          if let Ok(mut player) = get_player.get_mut(entity1) {
              handle_player_collision(&mut commands, entity1, &mut player, entity2, &mut get_runner, &mut get_tagger, &mut get_player1_text, &mut get_player2_text, &get_bullet, &get_orb, &audio, &orb_sound, &catch_sound, &mut dash_timer, &mut cooldown_timer);
          }
          else if let Ok(mut player) = get_player.get_mut(entity2) {
              handle_player_collision(&mut commands, entity2, &mut player, entity1, &mut get_runner, &mut get_tagger, &mut get_player1_text, &mut get_player2_text, &get_bullet, &get_orb, &audio, &orb_sound, &catch_sound, &mut dash_timer, &mut cooldown_timer);
          }
          else if let Ok(_) = get_orb.get(entity1) {
              commands.entity(entity2).despawn()
          }
          else if let Ok(_) = get_bullet.get(entity1) {
              commands.entity(entity1).despawn()
          }
      }
  }
}

fn handle_player_collision(
  commands: &mut Commands,
  player_entity: Entity, 
  player: &mut player::Player,
  other_entity: Entity,
  get_runner: &mut Query<(&mut DrawMode, Option<&player::Dash>), (With<player::Player>, Without<player::Tagger>)>,
  get_tagger: &mut Query<&mut DrawMode, (With<player::Player>, With<player::Tagger>)>,
  get_player1_text: &mut Query<&mut Text, (With<player::Player1>, Without<player::Player2>)>,
  get_player2_text: &mut Query<&mut Text, (With<player::Player2>, Without<player::Player1>)>,
  get_bullet: &Query<&bullet::Bullet, With<bullet::Bullet>>,
  get_orb: &Query<(), (With<orb::Orb>, Without<player::Player>)>,
  audio: &Res<Audio>,
  orb_sound: &Res<crate::OrbSound>,
  catch_sound: &Res<crate::CatchSound>,
  dash_timer: &mut player::DashTimer,
  cooldown_timer: &mut player::CooldownTimer,
) {
  if let Ok((mut runner_draw_mode, is_dashing)) = get_runner.get_mut(player_entity) {
      if is_dashing.is_none() {
          handle_runner_collision(commands, player_entity, player, other_entity, &mut runner_draw_mode, get_tagger, get_player1_text, get_player2_text, get_bullet, get_orb, audio, orb_sound, catch_sound, dash_timer, cooldown_timer);
      }
  }
  else if let Ok(_) = get_tagger.get_mut(player_entity) {
      handle_tagger_collision(commands, player, other_entity, get_orb, audio, orb_sound);
  }
}

fn handle_runner_collision(
  commands: &mut Commands,
  player_entity: Entity, 
  player: &mut player::Player,
  other_entity: Entity,
  draw_mode: &mut DrawMode,
  get_tagger: &mut Query<&mut DrawMode, (With<player::Player>, With<player::Tagger>)>,
  get_player1_text: &mut Query<&mut Text, (With<player::Player1>, Without<player::Player2>)>,
  get_player2_text: &mut Query<&mut Text, (With<player::Player2>, Without<player::Player1>)>,
  get_bullet: &Query<&bullet::Bullet, With<bullet::Bullet>>,
  get_orb: &Query<(), (With<orb::Orb>, Without<player::Player>)>,
  audio: &Res<Audio>,
  orb_sound: &Res<crate::OrbSound>,
  catch_sound: &Res<crate::CatchSound>,
  dash_timer: &mut player::DashTimer,
  cooldown_timer: &mut player::CooldownTimer,
) {
    if let Ok(bullet) = get_bullet.get(other_entity) {
        if let DrawMode::Fill(ref mut runner_fill_mode) = *draw_mode {
            if let Ok(mut tagger_draw_mode) = get_tagger.get_mut(bullet.shooter) {
                if let DrawMode::Fill(ref mut tagger_fill_mode) = *tagger_draw_mode {
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

                    runner_fill_mode.color = TAGGER_COL;
                    tagger_fill_mode.color = RUNNER_COL;
                    
                    let (mut player1_text, mut player2_text) = (get_player1_text.single_mut(), get_player2_text.single_mut());

                    if player1_text.sections.first_mut().unwrap().style.color == TAGGER_COL {
                        player1_text.sections.first_mut().unwrap().style.color = RUNNER_COL;
                    } else {
                        player1_text.sections.first_mut().unwrap().style.color = TAGGER_COL;
                    }

                    if player2_text.sections.first_mut().unwrap().style.color == TAGGER_COL {
                        player2_text.sections.first_mut().unwrap().style.color = RUNNER_COL;
                    } else {
                        player2_text.sections.first_mut().unwrap().style.color = TAGGER_COL;
                    }
            
                    audio.play(catch_sound.0.clone());

                    dash_timer.0.reset();
                    cooldown_timer.0.reset();
            
                    commands.entity(other_entity).despawn();
                }
            }
        }
    }
    else if let Ok(_) = get_orb.get(other_entity) {
        audio.play(orb_sound.0.clone());

        player.velocity = player.velocity.signum() * -1. * player::MAX_SPEED;
        commands.entity(other_entity).despawn();
    }
}

fn handle_tagger_collision(
  commands: &mut Commands,
  player: &mut player::Player,
  other_entity: Entity,
  get_orb: &Query<(), (With<orb::Orb>, Without<player::Player>)>,
  audio: &Res<Audio>,
  orb_sound: &Res<crate::OrbSound>,
) {
  if let Ok(_) = get_orb.get(other_entity) {
      audio.play(orb_sound.0.clone());

      player.velocity = player.velocity.signum() * -1. * player::MAX_SPEED;
      commands.entity(other_entity).despawn();
  }
}

fn tick_count_down_timer(
    mut commands: Commands,
    time: Res<Time>,
    timer_sound: Res<crate::TimerSound>,
    audio: Res<Audio>,
    mut app_state: ResMut<State<GameState>>,
    mut timer: ResMut<CountDownTimer>,
    mut text_query: Query<&mut Text, With<Countdown>>,
    player1_query: Query<(), (With<player::Player1>, With<player::Tagger>)>,
) { 
    timer.0.tick(time.delta());

    let elapsed = timer.0.elapsed_secs();
    let remaining_time = GAME_DUR - elapsed;
    let remaining_miutes = (remaining_time / 60.).floor();
    let remaining_seconds = (remaining_time - remaining_miutes * 60.).floor();

    let mut text = text_query.single_mut();
    let text = text.sections.first_mut().unwrap();

    let prev_value = text.value.clone();
    text.value = format!("{:0>2}:{:0>2}", remaining_miutes, remaining_seconds);

    if remaining_time <= 16. {
        if prev_value != text.value {
            audio.play(timer_sound.0.clone());
        }

        text.style.color = Color::RED;
    }

    if timer.0.just_finished() {
        if player1_query.is_empty() {
            commands.insert_resource(crate::Winner("Player_1".to_string()));
        } else {
            commands.insert_resource(crate::Winner("Player_2".to_string()));
        }
        
        app_state.push(GameState::MainMenu).unwrap();
    }
}