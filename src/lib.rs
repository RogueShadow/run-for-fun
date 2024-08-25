mod camera;
mod level_loader;
mod player_controls;
mod player_movement;
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

#[derive(Event)]
pub enum Play {
    Jump,
    Walk,
    Finish,
    Land,
    Start,
}

#[derive(Resource)]
pub struct Sounds {
    jump: Handle<AudioSource>,
    walk: Handle<AudioSource>,
    finish: Handle<AudioSource>,
    start: Handle<AudioSource>,
    land: Handle<AudioSource>,
}

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
                detect_race_status,
                update_character_position_from_velocity,
                update_movement_component.before(update_character_position_from_velocity),
                update_player_states,
                update_player_animation,
                move_camera,
            ),
        );
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}
#[derive(Component)]
pub struct AnimationAtlas {
    animations: Vec<Animation>,
    current: usize,
}
impl AnimationAtlas {
    pub fn new(animations: impl Into<Vec<Animation>>) -> Self {
        Self {
            animations: animations.into(),
            current: 0,
        }
    }
    pub fn next(&mut self) {
        if let Some(anim) = self.animations.get_mut(self.current) {
            anim.next();
        }
    }
    pub fn current(&self) -> usize {
        if let Some(anim) = self.animations.get(self.current) {
            anim.current()
        } else {
            0
        }
    }
}
impl Default for AnimationAtlas {
    fn default() -> Self {
        Self {
            animations: vec![],
            current: 0,
        }
    }
}

pub struct Animation {
    sprites: Vec<usize>,
    position: usize,
}
impl Animation {
    pub fn new(sprites: impl Into<Vec<usize>>) -> Self {
        Self {
            sprites: sprites.into(),
            position: 0,
        }
    }
    pub fn next(&mut self) {
        self.position += 1;
        if self.position >= self.sprites.len() {
            self.position = 0;
        }
    }
    pub fn current(&self) -> usize {
        self.sprites[self.position]
    }
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}

fn detect_race_status(
    player: Query<Entity, With<Player>>,
    mut collision_events: EventReader<CollisionEvent>,
    start: Query<Entity, With<Start>>,
    finish: Query<Entity, With<Finish>>,
    mut race_time: Query<&mut RaceTime>,
    mut commands: Commands,
    time: Res<Time>,
    mut text_q: Query<&mut Text, With<PlayerText>>,
) {
    if let Ok(mut race_timer) = race_time.get_single_mut() {
        race_timer.0.advance_by(time.delta());
    }
    for collision in collision_events.read() {
        if let Ok(player_entity) = player.get_single() {
            match collision {
                CollisionEvent::Started(e1, e2, _) => {
                    if [*e1, *e2].contains(&start.single()) && [*e1, *e2].contains(&player_entity) {
                        if let Ok(_) = race_time.get_single_mut() {
                            info!("You've already started, why you back here?!");
                            if let Ok(mut text) = text_q.get_single_mut() {
                                text.sections[0].value =
                                    "You've already started, why you back here?!".to_string();
                            } else {
                                error!("Couldn't get player's text.")
                            }
                        } else {
                            info!("Run to the finish!");
                            commands.trigger(Play::Start);
                            if let Ok(mut text) = text_q.get_single_mut() {
                                text.sections[0].value = "Run to the finish!".to_string();
                            } else {
                                error!("Couldn't get player's text.")
                            }
                            commands
                                .entity(player_entity)
                                .insert(RaceTime(Time::default()));
                        }
                    } else if [*e1, *e2].contains(&finish.single()) {
                        if let Ok(time) = race_time.get_single_mut() {
                            info!("You've finished! {:.3}", time.0.elapsed_seconds());
                            commands.trigger(Play::Finish);
                            if let Ok(mut text) = text_q.get_single_mut() {
                                text.sections[0].value =
                                    format!("You've finished! {:.3}", time.0.elapsed_seconds());
                            } else {
                                error!("Couldn't get player's text.")
                            }
                            commands.entity(player_entity).remove::<RaceTime>();
                        } else {
                            info!("Go back to the start, you haven't started!");
                        }
                    }
                }
                _ => {}
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
}

fn play_sounds(trigger: Trigger<Play>, mut commands: Commands, sounds: Res<Sounds>) {
    let source = match trigger.event() {
        Play::Jump => sounds.jump.clone_weak(),
        Play::Walk => sounds.walk.clone_weak(),
        Play::Finish => sounds.finish.clone_weak(),
        Play::Land => sounds.land.clone_weak(),
        Play::Start => sounds.start.clone_weak(),
    };
    commands.spawn(AudioSourceBundle {
        source,
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    });
}

fn update_character_position_from_velocity(
    mut player_query: Query<(&mut KinematicCharacterController, &PlayerMovement), With<Player>>,
) {
    if let Ok((mut controller, movement)) = player_query.get_single_mut() {
        controller.translation = Some(movement.velocity);
    }
}
