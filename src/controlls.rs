use bevy::prelude::*;
use crate::{GameState, game};

#[derive(Component)]
pub struct ControllsMenu;

#[derive(Component)]
pub struct BackButton;

pub struct ControllsMenuPlugin;

impl Plugin for ControllsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::ControllsMenu)
                .with_system(setup_controlls_menu),
        )
        .add_system_set(
            SystemSet::on_resume(GameState::ControllsMenu)
                .with_system(setup_controlls_menu),
        )
        .add_system_set(
            SystemSet::on_update(GameState::ControllsMenu)
                .with_system(handle_controlls_menu_interactions),
        )
        .add_system_set(
            SystemSet::on_pause(GameState::ControllsMenu)
                .with_system(teardown_controlls_menu_items),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::ControllsMenu)
                .with_system(teardown_controlls_menu_items),
        );
    }
}

pub fn setup_controlls_menu(
  mut commands: Commands,
  font: Res<crate::FontHandle>,
) {
    commands.spawn_bundle(NodeBundle {
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
    .insert(ControllsMenu)
    .with_children(|parent| {
        // PLAYER_1 CONTROLLS
        let player_1_controlls = [
            ("Move forward", "W"),
            ("Move backwards", "S"),
            ("Turn right", "D"),
            ("Turn left", "A"),
            ("Action", "Space"),
        ];

        create_player_controlls(parent, "Player_1", player_1_controlls, font.0.clone());

        // PLAYER_2 CONTROLLS
        let player_2_controlls = [
            ("Move forward", "Up"),
            ("Move backwards", "Down"),
            ("Turn right", "Right"),
            ("Turn left", "Left"),
            ("Action", "Enter"),
        ];

        create_player_controlls(parent, "Player_2", player_2_controlls, font.0.clone());

        // BACK BUTTON
        parent.spawn_bundle(ButtonBundle {
            style: Style {
                size: Size {
                    width: Val::Px(200.0),
                    height: Val::Px(50.0),
                },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..Style::default()
            },
            ..ButtonBundle::default()
        })
        .insert(BackButton)
        .with_children(|parent| {
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
                style: Style::default(),
                text: Text::with_section("Back", style, alignment),
                ..TextBundle::default()
            });
        });
    });
}

fn create_player_controlls(
    parent: &mut ChildBuilder,
    title: &str,
    controlls: [(&str, &str); 5],
    font: Handle<Font>,
) {
    let title_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: Color::WHITE,
    };

    let title_style = Style {
        margin: Rect {
            top: Val::Px(40.),
            bottom: Val::Px(10.),
            ..Default::default()
        },
        ..Default::default()
    };

    let controll_text_style = TextStyle {
        font: font.clone(),
        font_size: 25.0,
        color: Color::WHITE,
    };

    let alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

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
        parent.spawn_bundle(TextBundle {
            style: title_style.clone(),
            text: Text::with_section(title, title_text_style.clone(), alignment),
            ..TextBundle::default()
        });

        for (movement, button) in controlls {
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Px(400.0),
                        ..Default::default()
                    },
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Style::default()
                },
                visibility: Visibility {
                    is_visible: false,
                },
                ..NodeBundle::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    style: Style::default(),
                    text: Text::with_section(movement, controll_text_style.clone(), alignment),
                    ..TextBundle::default()
                });

                parent.spawn_bundle(TextBundle {
                    style: Style::default(),
                    text: Text::with_section(button, controll_text_style.clone(), alignment),
                    ..TextBundle::default()
                });
            });
        }
    });
}

pub fn handle_controlls_menu_interactions(
    mut app_state: ResMut<State<GameState>>,
    mut back_btn_query: Query<(&Interaction, &mut UiColor), With<BackButton>>,
) {
    for (interaction, mut button) in back_btn_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                app_state.push(GameState::MainMenu).unwrap();
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

pub fn teardown_controlls_menu_items(
    mut commands: Commands,
    query: Query<Entity, With<ControllsMenu>>
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
