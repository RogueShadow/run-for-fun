mod animation;
mod camera;
mod events_systems;
mod level_loader;
mod player_controls;
mod player_movement;
mod sound;
use animation::*;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::{AudioApp, AudioPlugin};
use bevy_rapier2d::prelude::*;
use camera::*;
use events_systems::*;
use itertools::Itertools;
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
#[derive(Resource)]
pub struct BackgroundMusic;

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
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            Distance::PIXELS_PER_METER,
        ));
        app.add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        });
        app.add_plugins(LdtkPlugin);
        app.add_plugins(FrameTimeDiagnosticsPlugin);
        app.add_plugins(PerfUiPlugin);
        app.add_plugins(RustAnimationPlugin);
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_plugins(AudioPlugin::default());
        app.add_audio_channel::<BackgroundMusic>();
        app.insert_resource(LevelSelection::Uid(0));
        app.insert_resource(MousePosition(Vec2::ZERO));
        app.register_type::<Jump>();
        app.add_systems(Startup, setup_camera);
        app.add_systems(Startup, setup);
        app.add_systems(FixedPreUpdate, build_collision_boxes);
        app.add_systems(
            PreUpdate,
            (
                update_player_controls,
                update_mouse_position,
                player_wall_ceiling_checks,
            ),
        );
        app.add_systems(
            Update,
            (
                update_speedometer,
                menu_interaction,
                detect_flags,
                advance_race_timer,
                update_character_position_from_velocity,
                update_jump_component.before(update_character_position_from_velocity),
                update_run_component.before(update_character_position_from_velocity),
                update_player_states,
                update_player_animation,
                update_platform_position,
                move_camera.after(update_character_position_from_velocity),
            ),
        );
    }
}

pub fn update_mouse_position(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_position: ResMut<MousePosition>,
) {
    let (camera, camera_transform) = camera.single();
    if let Ok(window) = window.get_single() {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            mouse_position.0 = world_position;
        }
    }
}

pub fn advance_race_timer(mut race_timer: Query<&mut RaceTime>, time: Res<Time>) {
    if let Ok(mut race_timer) = race_timer.get_single_mut() {
        race_timer.0.advance_by(time.delta());
    }
}

pub fn detect_flags(
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
    rapier_config.gravity.y = -300.0;
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: 1. / 120.,
        substeps: 4,
    };

    //Load some Sounds
    let sounds = Sounds {
        bgm: assets.load("Caketown 1.mp3"),
        jump: assets.load("jump_01.wav"),
        walk: assets.load("03_Step_grass_03.wav"),
        finish: assets.load("Won!.wav"),
        start: assets.load("Start_Sounds_003.wav"),
        land: assets.load("45_Landing_01.wav"),
    };
    cmds.insert_resource(sounds);

    //Observe all the things.
    cmds.observe(play_sounds);
    cmds.observe(player_touched_flags);
    cmds.observe(spawn_player);
    cmds.observe(spawn_box);
    cmds.observe(spawn_message);
    cmds.observe(spawn_flags);
    cmds.observe(start_background_music);
    cmds.observe(spawn_platform);

    //UI Attempts
    let root_row = cmds
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                left: Val::Px(4.),
                top: Val::Px(4.),
                ..default()
            },
            ..default()
        })
        .id();
    let column1 = cmds
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                left: Val::Px(4.),
                top: Val::Px(4.),
                ..default()
            },
            ..default()
        })
        .id();
    let column2 = cmds
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                left: Val::Px(4.),
                top: Val::Px(4.),
                ..default()
            },
            ..default()
        })
        .id();
    cmds.entity(root_row).add_child(column1);
    cmds.entity(root_row).add_child(column2);
    let button_labels1 = ["Jump+", "FDrag+"];
    let button_labels2 = ["Jump-", "FDrag-"];
    cmds.entity(column1).with_children(|parent| {
        for label in button_labels1 {
            parent.spawn(button()).with_children(|parent| {
                parent.spawn(text(label));
            });
        }
    });
    cmds.entity(column2).with_children(|parent| {
        for label in button_labels2 {
            parent.spawn(button()).with_children(|parent| {
                parent.spawn(text(label));
            });
        }
    });
}

pub fn button() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(90.0),
            height: Val::Px(30.0),
            border: UiRect::all(Val::Px(2.0)),
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        background_color: BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        border_color: BorderColor(Color::srgb(0., 0., 0.)),
        border_radius: BorderRadius::all(Val::Px(8.)),
        ..default()
    }
}
pub fn text(label: impl Into<String>) -> TextBundle {
    TextBundle {
        text: Text::from_section(label, TextStyle::default()),
        ..default()
    }
}

pub fn menu_interaction(
    interaction_query: Query<(&Interaction, &Children), Changed<Interaction>>,
    text_query: Query<&Text>,
    mut movement_query: Query<(&mut Jump, &mut Run)>,
) {
    for (interaction, children) in interaction_query.iter() {
        let text = &text_query.get(children[0]).unwrap().sections[0].value;
        match interaction {
            Interaction::Pressed => {
                let (mut jump, mut run) = movement_query.single_mut();
                match text.as_str() {
                    _ => {}
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
