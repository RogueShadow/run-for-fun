mod animation;
mod camera;
mod level_loader;
mod player_controls;
mod player_movement;
mod sound;
use animation::*;
use bevy::asset::AssetMetaCheck;
use bevy::audio::{PlaybackMode, Volume};
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use camera::*;
use iyes_perf_ui::prelude::*;
use level_loader::*;
use player_controls::*;
use player_movement::*;
use sound::*;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Component)]
pub struct PlayerText;
#[derive(Component)]
pub struct Start;
#[derive(Component)]
pub struct Finish;
#[derive(Component, Deref, DerefMut)]
pub struct RaceTime(Time);
#[derive(Component)]
pub struct Player;

#[wasm_bindgen(start)]
pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(RunGame)
        .run();
}

struct RunGame;

impl Plugin for RunGame {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(16.0).in_fixed_schedule(),
        );
        app.add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        });
        app.add_plugins(LdtkPlugin);
        app.add_plugins(FrameTimeDiagnosticsPlugin);
        app.add_plugins(PerfUiPlugin);
        app.insert_resource(LevelSelection::Uid(0));
        app.add_systems(Startup, setup_camera);
        app.add_systems(Startup, setup);
        app.add_systems(FixedPreUpdate, build_collision_boxes);
        app.add_systems(PreUpdate, update_player_controls);
        app.add_systems(
            Update,
            (
                animate_sprite,
                detect_flags,
                advance_race_timer,
                update_character_position_from_velocity,
                update_movement_component.before(update_character_position_from_velocity),
                update_player_states,
                update_player_animation,
                move_camera,
            ),
        );
    }
}

fn advance_race_timer(mut race_timer: Query<&mut RaceTime>, time: Res<Time>) {
    if let Ok(mut race_timer) = race_timer.get_single_mut() {
        race_timer.0.advance_by(time.delta());
    }
}

#[derive(Event)]
pub enum TouchedFlag {
    Start,
    Finish,
}

fn player_touched_flags(
    trigger: Trigger<TouchedFlag>,
    mut text_query: Query<&mut Text, With<PlayerText>>,
    mut commands: Commands,
    mut race_time_query: Query<&mut RaceTime>,
    player: Query<Entity, With<Player>>,
) {
    let player_entity = player.single();
    let mut msg = |msg: &str| {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = msg.into();
        }
    };
    match trigger.event() {
        TouchedFlag::Start => {
            if let Ok(_) = race_time_query.get_single_mut() {
                msg("You've already started, why you back here?!");
            } else {
                commands.trigger(Play::Start);
                msg("Run to the finish!");
                commands
                    .entity(player_entity)
                    .insert(RaceTime(Time::default()));
            }
        }
        TouchedFlag::Finish => {
            if let Ok(time) = race_time_query.get_single_mut() {
                commands.trigger(Play::Finish);
                msg(&format!("You've finished! {:.3}", time.0.elapsed_seconds()));
                commands.entity(player_entity).remove::<RaceTime>();
            }
        }
    }
}

fn detect_flags(
    player: Query<Entity, With<Player>>,
    mut collision_events: EventReader<CollisionEvent>,
    start: Query<Entity, With<Start>>,
    finish: Query<Entity, With<Finish>>,
    mut commands: Commands,
) {
    let player_entity = if let Ok(entity) = player.get_single() {
        entity
    } else {
        return;
    };
    for collision in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = collision {
            if ![*e1, *e2].contains(&player_entity) {
                return;
            }
            if [*e1, *e2].contains(&start.single()) {
                commands.trigger(TouchedFlag::Start);
            } else if [*e1, *e2].contains(&finish.single()) {
                commands.trigger(TouchedFlag::Finish);
            }
        }
    }
}

fn setup(
    mut cmds: Commands,
    assets: Res<AssetServer>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    //Load Level
    let level_handle: Handle<LdtkProject> = assets.load("run_level.ldtk");
    cmds.spawn(LdtkWorldBundle {
        ldtk_handle: level_handle,
        ..default()
    });

    //Setup Physics
    rapier_config.gravity.y = -200.0;

    // Play some music
    cmds.spawn(AudioBundle {
        source: assets.load("Caketown 1.mp3"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.5),
            ..default()
        },
    });

    //Load some Sounds
    let sounds = Sounds {
        jump: assets.load("jump_01.wav"),
        walk: assets.load("03_Step_grass_03.wav"),
        finish: assets.load("Won!.wav"),
        start: assets.load("Start_Sounds_003.wav"),
        land: assets.load("45_Landing_01.wav"),
    };
    cmds.insert_resource(sounds);
    cmds.observe(play_sounds);
    cmds.observe(player_touched_flags);
}
