use bevy::prelude::*;
use crate::{GameState, game};

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct ControllsButton;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::MainMenu)
                .with_system(setup_main_menu),
        )
        .add_system_set(
            SystemSet::on_resume(GameState::MainMenu)
                .with_system(setup_main_menu),
        )
        .add_system_set(
            SystemSet::on_update(GameState::MainMenu)
                .with_system(handle_menu_interactions),
        )
        .add_system_set(
            SystemSet::on_pause(GameState::MainMenu)
                .with_system(teardown_menu_items),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::MainMenu)
                .with_system(teardown_menu_items),
        );
    }
}

pub fn setup_main_menu(
  mut commands: Commands,
  winner: Option<Res<crate::Winner>>,
  font: Res<crate::FontHandle>,
) {
  commands
    .spawn_bundle(NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            ..Style::default()
        },
        visibility: Visibility {
            is_visible: false,
        },
        ..NodeBundle::default()
    })
    .insert(MainMenu)
    .with_children(|parent| {
        let text = Text {
            sections: vec![
                TextSection {
                    value: "Bevy ".to_string(),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: 70.0,
                        color: game::TAGGER_COL,
                    },
                },
                TextSection {
                    value: "Tag".to_string(),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: 70.0,
                        color: game::RUNNER_COL,
                    },
                }
            ],
            alignment: TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            },
        };

        parent.spawn_bundle(TextBundle {
            text,
            ..TextBundle::default()
        });

        if let Some(winner) = &winner {
            let style = TextStyle {
                font: font.0.clone(),
                font_size: 30.0,
                color: Color::WHITE,
            };

            let alignment = TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            };

            parent.spawn_bundle(TextBundle {
                text: Text::with_section(format!("{}  won!", winner.0), style, alignment),
                ..TextBundle::default()
            });
        }

        parent.spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..Style::default()
            },
            visibility: Visibility {
                is_visible: false,
            },
            ..NodeBundle::default()
        })
        .with_children(|parent| {
            let btn_style = Style {
                size: Size {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                },
                margin: Rect {
                      bottom: Val::Px(20.),
                      ..Default::default()
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..Style::default()
            };

            let btn_text_style = TextStyle {
                font: font.0.clone(),
                font_size: 30.0,
                color: Color::WHITE,
            };

            let btn_text_alignment = TextAlignment {
                vertical: VerticalAlign::Center,
                horizontal: HorizontalAlign::Center,
            };

            parent.spawn_bundle(ButtonBundle {
              style: btn_style.clone(),
              ..ButtonBundle::default()
            })
            .insert(PlayButton)
            .with_children(|parent| {   
                let btn_text = if winner.is_some() {"Play again"} else {"Play"};

                parent.spawn_bundle(TextBundle {
                    style: Style::default(),
                    text: Text::with_section(btn_text, btn_text_style.clone(), btn_text_alignment.clone()),
                    ..TextBundle::default()
                });
            });
    
            parent.spawn_bundle(ButtonBundle {
                style: btn_style.clone(),
                ..ButtonBundle::default()
              })
              .insert(ControllsButton)
              .with_children(|parent| {
                  parent.spawn_bundle(TextBundle {
                      style: Style::default(),
                      text: Text::with_section("Controlls", btn_text_style.clone(), btn_text_alignment.clone()),
                      ..TextBundle::default()
                  });
              });
        });
    });
}

pub fn handle_menu_interactions(
    mut app_state: ResMut<State<GameState>>,
    mut play_btn_query: Query<(&Interaction, &mut UiColor), (With<PlayButton>, Without<ControllsButton>)>,
    mut controlls_btn_query: Query<(&Interaction, &mut UiColor), (With<ControllsButton>, Without<PlayButton>)>,
) {
    for (interaction, mut button) in play_btn_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                app_state.push(GameState::Game).unwrap();
            },
            Interaction::Hovered => {
                button.0 = game::ORB_OUTLINE_COLOR;
            },
            Interaction::None => {
                button.0 = Color::rgb(149./255., 53./255., 184./255.);
            }
        }
    }

    for (interaction, mut button) in controlls_btn_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                app_state.push(GameState::ControllsMenu).unwrap();
            },
            Interaction::Hovered => {
                button.0 = game::ORB_OUTLINE_COLOR;
            },
            Interaction::None => {
                button.0 = Color::rgb(149./255., 53./255., 184./255.);
            }
        }
    }
}

pub fn teardown_menu_items(
    mut commands: Commands,
    query: Query<Entity, With<MainMenu>>
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
