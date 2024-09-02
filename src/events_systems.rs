use crate::animation::{RustAnimation, RustAnimationAtlas, Spline};
use crate::camera::Follow;
use crate::player_controls::{PlayerControls, PlayerState};
use crate::player_movement::PlayerMovement;
use crate::sound::Sounds;
use crate::Play;
use crate::RaceTime;
use crate::{Finish, Player, PlayerText, Start};
use bevy::audio::{PlaybackMode, Volume};
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_rapier2d::control::{CharacterLength, KinematicCharacterController};
use bevy_rapier2d::dynamics::{LockedAxes, RigidBody};
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Friction, Sensor};
use bevy_rapier2d::prelude::AdditionalMassProperties;

#[derive(Event)]
pub struct SpawnPlayerEvent {
    pub position: Vec2,
}
#[derive(Event)]
pub struct SpawnBoxEvent {
    pub position: Vec2,
}
#[derive(Event)]
pub struct SpawnMessageEvent {
    pub message: String,
    pub position: Vec2,
}
pub enum FlagType {
    Start,
    Finish,
}
#[derive(Event)]
pub struct SpawnFlagEvent {
    pub flag: FlagType,
    pub position: Vec2,
    pub size: Vec2,
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
                    transform: Transform::from_xyz(position.x, position.y, 50.0),
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
                RustAnimation::range(4, 7, 0.1),
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
                    transform: Transform::from_xyz(position.x, position.y, 50.0),
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
                RustAnimation::range(0, 3, 0.1),
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
        transform: Transform::from_xyz(position.x, position.y, 60.0).with_scale(Vec3::splat(0.1)),
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
    let red_block = assets.load("box.png");
    commands.spawn((
        SpriteBundle {
            texture: red_block.clone(),
            sprite: Sprite {
                custom_size: Some(vec2(16.0, 16.0)),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 100.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(8.0, 8.0),
        Friction::coefficient(0.5),
        AdditionalMassProperties::Mass(3000.0),
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
    let points = (0..360)
        .step_by(20)
        .map(|x| {
            let len = 16.0;
            let x = (x as f32).to_radians();
            let xpos = x.cos() * len;
            let ypos = x.sin() * len;
            Vec2::new(xpos, ypos)
        })
        .collect::<Vec<_>>();
    let mut spline = Spline::new(points, true);
    commands
        .spawn((
            Follow,
            Player,
            PlayerMovement::default(),
            PlayerControls::default(),
            PlayerState::default(),
            spline,
            RustAnimationAtlas::new([
                RustAnimation::list([0], 0.1),
                RustAnimation::list([0, 1, 2, 3], 0.1),
                RustAnimation::list([0, 1, 2, 3, 4], 0.1),
                RustAnimation::list([0], 0.1),
                RustAnimation::list([5], 0.1),
                RustAnimation::list([0, 1, 2, 3], 0.1),
            ]),
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

#[derive(Event)]
pub enum TouchedFlag {
    Start,
    Finish,
}

pub fn player_touched_flags(
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

#[derive(Event)]
pub struct StartBackgroundMusic;

pub fn start_background_music(
    _: Trigger<StartBackgroundMusic>,
    sounds: Res<Sounds>,
    mut commands: Commands,
) {
    commands.spawn(AudioBundle {
        source: sounds.bgm.clone_weak(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.25),
            ..default()
        },
    });
}
