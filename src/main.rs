use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;
use heron::prelude::*;
use bevy_tweening::*;
use rand::{rngs::StdRng, SeedableRng};
use bevy_kira_audio::{AudioPlugin, AudioSource};

mod player;
mod bullet;
mod orb;
mod menu;
mod controlls;
mod game;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    MainMenu,
    ControllsMenu,
    Game,
}

pub struct Winner(String);

pub struct Random(StdRng);

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy tagger".to_string(),
            width: 800.,
            height: 800.,
            vsync: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(30./255., 30./255., 30./255.)))
        .insert_resource(Random(StdRng::from_entropy()))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(menu::MenuPlugin)
        .add_plugin(controlls::ControllsMenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_state(GameState::MainMenu)
        .add_startup_system(setup)
        .run();
}

pub struct OrbSound(Handle<AudioSource>);
pub struct CatchSound(Handle<AudioSource>);
pub struct TimerSound(Handle<AudioSource>);
pub struct DashSound(Handle<AudioSource>);
pub struct ShootSound(Handle<AudioSource>);
pub struct FontHandle(Handle<Font>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let orb_audio: Handle<AudioSource> = asset_server.load("orb.wav");
    let catch_audio: Handle<AudioSource> = asset_server.load("catch.wav");
    let timer_audio: Handle<AudioSource> = asset_server.load("timer.wav");
    let dash_audio: Handle<AudioSource> = asset_server.load("dash.wav");
    let shoot_audio: Handle<AudioSource> = asset_server.load("shoot.wav");

    commands.insert_resource(OrbSound(orb_audio));
    commands.insert_resource(CatchSound(catch_audio));
    commands.insert_resource(TimerSound(timer_audio));
    commands.insert_resource(DashSound(dash_audio));
    commands.insert_resource(ShootSound(shoot_audio));
    
    let font = asset_server.load("zorque.otf");
    commands.insert_resource(FontHandle(font.clone()));

    commands.spawn_bundle(UiCameraBundle::default());
}
