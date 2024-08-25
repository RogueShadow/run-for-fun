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
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::na::DimAdd;
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
    cmds.observe(spawn_player);
    cmds.observe(spawn_box);
    cmds.observe(spawn_message);
    cmds.observe(spawn_flags);
}

#[derive(Event)]
pub struct SpawnPlayerEvent {
    position: Vec2,
}
#[derive(Event)]
pub struct SpawnBoxEvent {
    position: Vec2,
}
#[derive(Event)]
pub struct SpawnMessageEvent {
    message: String,
    position: Vec2,
}
pub enum FlagType {
    Start,
    Finish,
}
#[derive(Event)]
pub struct SpawnFlagEvent {
    flag: FlagType,
    position: Vec2,
    size: Vec2,
}

pub fn spawn_flags(
    trigger: Trigger<SpawnFlagEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let spawn_event = trigger.event();
    let position = spawn_event.position;
    let size = spawn_event.size;
    let texture = asset_server.load("flag_red_green.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(125, 250), 4, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    match spawn_event.flag {
        FlagType::Start => {
            commands.spawn((
                Start,
                Collider::cuboid(size.x / 2.0, size.y / 2.0),
                Sensor,
                ActiveCollisionTypes::KINEMATIC_STATIC,
                ActiveEvents::COLLISION_EVENTS,
                SpriteBundle {
                    texture: texture.clone(),
                    transform: Transform::from_xyz(position.x, position.y, 150.0),
                    sprite: Sprite {
                        custom_size: Some(size),
                        ..default()
                    },
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 4,
                },
                AnimationIndices::new(4, 7),
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ));
        }
        FlagType::Finish => {
            commands.spawn((
                Finish,
                Collider::cuboid(size.x / 2.0, size.y / 2.0),
                Sensor,
                ActiveCollisionTypes::KINEMATIC_STATIC,
                ActiveEvents::COLLISION_EVENTS,
                SpriteBundle {
                    texture: texture.clone(),
                    transform: Transform::from_xyz(position.x, position.y, 150.0),
                    sprite: Sprite {
                        custom_size: Some(size),
                        ..default()
                    },
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: 0,
                },
                AnimationIndices::new(0, 3),
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ));
        }
    }
}

pub fn spawn_message(trigger: Trigger<SpawnMessageEvent>, mut commands: Commands) {
    let spawn_event = trigger.event();
    let message = spawn_event.message.as_str();
    let position = spawn_event.position;
    commands.spawn(Text2dBundle {
        text: Text {
            sections: vec![TextSection::new(
                message,
                TextStyle {
                    color: Color::srgb(0.0, 0.0, 0.0),
                    font_size: 64.0,
                    ..default()
                },
            )],
            justify: JustifyText::Center,
            ..default()
        },
        text_anchor: Anchor::Center,
        transform: Transform::from_xyz(position.x, position.y, 200.0).with_scale(Vec3::splat(0.1)),
        ..default()
    });
}
pub fn spawn_box(
    trigger: Trigger<SpawnBoxEvent>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    let spawn_event = trigger.event();
    let position = spawn_event.position;
    let red_block = assets.load("red_block.png");
    commands.spawn((
        SpriteBundle {
            texture: red_block.clone(),
            sprite: Sprite { ..default() },
            transform: Transform::from_xyz(position.x, position.y, 100.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 8.0),
        Friction::coefficient(0.5),
    ));
}
pub fn spawn_player(
    trigger: Trigger<SpawnPlayerEvent>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let spawn_event = trigger.event();
    let player_pos = spawn_event.position;
    let texture = assets.load("character.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 3, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands
        .spawn((
            Follow,
            Player,
            PlayerMovement::default(),
            PlayerControls::default(),
            PlayerState::default(),
            AnimationAtlas::new([
                Animation::new([0]),
                Animation::new([0, 1, 2, 3]),
                Animation::new([0, 1, 2, 3, 4]),
                Animation::new([0]),
                Animation::new([5]),
                Animation::new([0, 1, 2, 3]),
            ]),
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            SpriteBundle {
                sprite: Sprite {
                    flip_x: false,
                    custom_size: None,
                    rect: None,
                    anchor: Anchor::Custom(vec2(0.0, -0.25)),
                    ..default()
                },
                texture,
                transform: Transform::from_xyz(player_pos.x, player_pos.y, 100.0),
                ..default()
            },
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 2,
            },
            RigidBody::KinematicVelocityBased,
            Collider::cuboid(8.0, 7.75),
            LockedAxes::ROTATION_LOCKED,
            KinematicCharacterController {
                translation: None,
                offset: CharacterLength::Relative(0.01),
                normal_nudge_factor: 0.001,
                slide: true,
                snap_to_ground: Some(CharacterLength::Relative(0.05)),
                apply_impulse_to_dynamic_bodies: true,
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn((
                PlayerText,
                Text2dBundle {
                    text: Text::from_section(
                        "Hello World",
                        TextStyle {
                            font: Default::default(),
                            font_size: 40.0,
                            color: Color::srgb(0.0, 0.0, 0.0),
                        },
                    )
                    .with_justify(JustifyText::Center),
                    transform: Transform::from_xyz(0.0, 20.0, 0.0).with_scale(Vec3::splat(0.1)),
                    ..default()
                },
            ));
        });
}
