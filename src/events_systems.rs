use crate::animation::{circle_spline, RustAnimation, RustAnimationAtlas};
use crate::assets::Sounds;
use crate::camera::Follow;
use crate::player_controls::PlayerState;
use crate::player_movement::{Jump, Run, SideChecks, Speedometer};
use crate::BackgroundMusic;
use crate::RaceTime;
use crate::{Finish, Player, PlayerText, Start};
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::{EntityInstance, LdtkEntity};
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::control::{CharacterLength, KinematicCharacterController};
use bevy_rapier2d::dynamics::{LockedAxes, RigidBody};
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Friction, Sensor};
use bevy_rapier2d::prelude::{AdditionalMassProperties, Velocity};

pub struct EventsPlugin;
impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.observe(play_sounds);
        app.observe(player_touched_flags);
        app.observe(spawn_player);
        app.observe(spawn_box);
        app.observe(spawn_message);
        app.observe(spawn_flags);
        app.observe(start_background_music);
        app.observe(spawn_platform);
    }
}

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
#[derive(Event)]
pub struct SpawnPlatformEvent {
    pub(crate) spline: CubicCardinalSpline<Vec2>,
    pub(crate) speed: f32,
}
#[derive(Component)]
pub struct MovingPlatform(CubicCardinalSpline<Vec2>, f32);

pub fn spawn_platform(
    trigger: Trigger<SpawnPlatformEvent>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    let spawn_event = trigger.event();

    if let Some(position) = spawn_event.spline.control_points.first() {
        let texture = assets.load("red_block.png");
        commands.spawn((
            MovingPlatform(spawn_event.spline.to_owned(), spawn_event.speed),
            RigidBody::KinematicPositionBased,
            LockedAxes::ROTATION_LOCKED,
            Collider::cuboid(32.0, 8.0),
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(64.0, 16.0)),
                    rect: None,
                    ..default()
                },
                transform: Transform::from_xyz(position.x, position.y, 120.0),
                texture,
                ..default()
            },
        ));
        info!("Spawned platform");
    }
}
pub fn update_platform_position(
    mut query_platforms: Query<(&MovingPlatform, &mut Transform)>,
    time: Res<Time>,
) {
    let mut elapsed = time.elapsed_seconds() / 4.0;
    for (platform, mut transform) in query_platforms.iter_mut() {
        let curve = platform.0.to_curve();
        let t_len = platform.0.control_points.len() as f32 - 1.0;
        let pos = curve.position(elapsed % t_len);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
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
            SideChecks::default(),
            //Speedometer::default(),
            Jump::default(),
            Run::default(),
            PlayerState::default(),
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
            Collider::cuboid(7.75, 7.75),
            LockedAxes::ROTATION_LOCKED,
            KinematicCharacterController {
                translation: None,
                offset: CharacterLength::Relative(0.01),
                normal_nudge_factor: 0.001,
                slide: true,
                snap_to_ground: Some(CharacterLength::Relative(0.03)),
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
    audio: Res<AudioChannel<BackgroundMusic>>,
) {
    audio
        .play(sounds.bgm.clone_weak())
        .looped()
        .with_volume(0.25);
}

#[derive(Event)]
pub enum Play {
    Jump,
    Walk,
    Finish,
    Land,
    Start,
}

pub fn play_sounds(trigger: Trigger<Play>, sfx: Res<AudioChannel<MainTrack>>, sounds: Res<Sounds>) {
    let source = match trigger.event() {
        Play::Jump => sounds.jump.clone_weak(),
        Play::Walk => sounds.walk.clone_weak(),
        Play::Finish => sounds.finish.clone_weak(),
        Play::Land => sounds.land.clone_weak(),
        Play::Start => sounds.start.clone_weak(),
    };

    sfx.play(source);
}
