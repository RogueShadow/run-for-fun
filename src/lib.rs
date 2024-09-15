pub mod animation;
pub mod assets;
pub mod camera;
pub mod events_systems;
pub mod level_loader;
pub mod player_controls;
pub mod player_movement;
pub mod entities {
    pub mod crates;
    pub mod flags;
    pub mod message;
    pub mod player;
}
use crate::entities::player::PlayerMarker;
use animation::*;
use assets::*;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_asset_loader::prelude::*;
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
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Component)]
pub struct PlayerText;
#[derive(Component, Default)]
pub struct Start;
#[derive(Component, Default)]
pub struct Finish;
#[derive(Component, Deref, DerefMut)]
pub struct RaceTime(Time);
#[derive(Resource)]
pub struct BackgroundMusic;
#[derive(Resource)]
pub struct SoundEffects;

#[wasm_bindgen(start)]
pub fn run() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Run for Fun: With Physics!".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(RunGame)
        .run();
}

struct RunGame;

impl Plugin for RunGame {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app.add_plugins(LoadingPlugin);
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            Distance::PIXELS_PER_METER,
        ));
        app.add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        });
        app.add_plugins(RFFLevelPlugin);
        app.add_plugins(FrameTimeDiagnosticsPlugin);
        app.add_plugins(PerfUiPlugin);
        app.add_plugins(RustAnimationPlugin);
        app.add_plugins(WorldInspectorPlugin::default());
        app.add_plugins(AudioPlugin::default());
        app.add_plugins(CameraPlugin);
        app.add_plugins(EventsPlugin);
        app.add_plugins(PlayerControlPlugin);
        app.add_audio_channel::<BackgroundMusic>();
        app.add_audio_channel::<SoundEffects>();
        app.insert_resource(MousePosition(Vec2::ZERO));
        app.add_systems(OnEnter(GameState::LoadGame), setup);
        app.add_systems(PreUpdate, update_mouse_position);
        app.add_systems(
            Update,
            (menu_interaction, detect_flags, advance_race_timer)
                .run_if(in_state(GameState::LoadGame)),
        );
        app.add_systems(Update, log_transitions);
    }
}

pub fn log_transitions(mut transition_event: EventReader<StateTransitionEvent<GameState>>) {
    for event in transition_event.read() {
        info!("{:?}", event);
    }
}

#[derive(Default, States, Debug, Eq, PartialEq, Hash, Clone)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    LoadGame,
    InGame,
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
    player: Query<Entity, With<PlayerMarker>>,
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
    mut rapier_config: ResMut<RapierConfiguration>,
    level_query: Res<Levels>,
) {
    //Setup Physics
    rapier_config.gravity.y = -300.0;
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: 1. / 120.,
        substeps: 4,
    };

    cmds.spawn(LdtkWorldBundle {
        ldtk_handle: level_query.level1.clone(),
        ..default()
    });

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
                if let Ok((mut jump, mut run)) = movement_query.get_single_mut() {
                    match text.as_str() {
                        _ => {}
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
