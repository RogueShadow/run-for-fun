use crate::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::ldtk::loaded_level::LoadedLevel;
use bevy_ecs_ldtk::ldtk::TileInstance;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

pub struct RFFLevelPlugin;
impl Plugin for RFFLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin);
        app.insert_resource(LevelSelection::Uid(0));
        app.register_ldtk_entity::<PlayerBundle>("Player_start");
        app.register_ldtk_entity::<PhysicsBlock>("Physics_block");
        app.register_ldtk_entity::<StartFlag>("Start");
        app.register_ldtk_entity::<FinishFlag>("Finish");
        app.register_ldtk_entity::<LevelTextBundle>("Text");
        app.add_systems(
            Update,
            (
                dynamic_collision_layer_building,
                spawn_player,
                spawn_flags,
                spawn_world_message,
            )
                .run_if(in_state(GameState::LoadGame)),
        );
    }
}
#[derive(Component, Default)]
pub struct PlayerMarker {
    position: Vec2,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct PlayerBundle {
    #[sprite_bundle]
    sprite_bundle: SpriteBundle,
    player: PlayerMarker,
}

#[derive(Bundle, LdtkEntity)]
pub struct PhysicsBlock {
    #[sprite_bundle]
    sprite_bundle: SpriteBundle,
    rigid_body: RigidBody,
    collider: Collider,
}
impl Default for PhysicsBlock {
    fn default() -> Self {
        Self {
            collider: Collider::cuboid(8.0, 8.0),
            rigid_body: RigidBody::Dynamic,
            sprite_bundle: Default::default(),
        }
    }
}
#[derive(Bundle, LdtkEntity)]
pub struct StartFlag {
    #[sprite_bundle("flag_red_green.png")]
    sprite_bundle: SpriteBundle,
    start: Start,
    collider: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
    active_events: ActiveEvents,
    rust_animation: RustAnimation,
}
impl Default for StartFlag {
    fn default() -> Self {
        Self {
            sprite_bundle: Default::default(),
            start: Start,
            collider: Collider::cuboid(8.0, 16.0),
            active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
            active_events: ActiveEvents::COLLISION_EVENTS,
            sensor: Sensor,
            rust_animation: RustAnimation::range(4, 7, 0.1),
        }
    }
}
#[derive(Bundle, LdtkEntity)]
pub struct FinishFlag {
    #[sprite_bundle("flag_red_green.png")]
    sprite_bundle: SpriteBundle,
    finish: Finish,
    collider: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
    active_events: ActiveEvents,
    rust_animation: RustAnimation,
}
impl Default for FinishFlag {
    fn default() -> Self {
        Self {
            sprite_bundle: Default::default(),
            finish: Finish,
            collider: Collider::cuboid(8.0, 16.0),
            active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
            active_events: ActiveEvents::COLLISION_EVENTS,
            sensor: Sensor,
            rust_animation: RustAnimation::range(0, 3, 0.1),
        }
    }
}

#[derive(Component, Default)]
pub struct WorldMessage;
#[derive(LdtkEntity, Bundle, Default)]
pub struct LevelTextBundle {
    world_message: WorldMessage,
    #[with(world_text)]
    text: Text2dBundle,
}
fn spawn_world_message(mut messages: Query<&mut Transform, Added<WorldMessage>>) {
    for mut transform in messages.iter_mut() {
        transform.scale = Vec3::splat(0.1);
    }
}
fn world_text(entity_instance: &EntityInstance) -> Text2dBundle {
    let msg = entity_instance
        .get_string_field("message")
        .expect("There should be a message field");
    Text2dBundle {
        text: Text::from_section(
            msg,
            TextStyle {
                color: Color::srgb(0.0, 0.0, 0.0),
                font_size: 64.0,
                ..default()
            },
        ),
        ..default()
    }
}

pub struct TiledCollisionBuilder {
    building: bool,
    position: Vec2,
    size: Vec2,
    height: f32,
    tile_size: Vec2,
}
impl Default for TiledCollisionBuilder {
    fn default() -> Self {
        Self {
            size: Vec2::ZERO,
            building: false,
            height: 32.0,
            position: Vec2::ZERO,
            tile_size: Vec2::new(16.0, 16.0),
        }
    }
}
impl TiledCollisionBuilder {
    pub fn build(&mut self) -> (SpatialBundle, Collider, RigidBody) {
        let components = (
            SpatialBundle {
                transform: Transform::from_xyz(
                    self.position.x + (self.size.x / 2.0) - self.tile_size.x / 2.0,
                    self.position.y - (self.height - self.tile_size.y) / 2.0,
                    0.0,
                ),
                ..default()
            },
            Collider::cuboid(self.size.x / 2.0, self.height / 2.0),
            RigidBody::Fixed,
        );
        self.reset();
        components
    }
    pub fn begin(&mut self, pos: Vec2, tile_size: Vec2, height: f32) {
        self.size = tile_size;
        self.position = pos;
        self.tile_size = tile_size;
        self.height = height;
        self.building = true;
    }
    pub fn reset(&mut self) {
        let b = Self::default();
        self.building = b.building;
        self.tile_size = b.tile_size;
        self.size = b.size;
        self.height = b.height;
        self.position = b.position;
    }
    pub fn extend(&mut self) {
        if !self.building {
            return;
        }
        self.size.x += self.tile_size.x;
    }
}

pub fn dynamic_collision_layer_building(
    mut cmds: Commands,
    q_tile: Query<(Entity, &TileStorage, &TilemapGridSize, &LayerMetadata), Added<TileStorage>>,
    data: Query<&TileMetadata>,
    map_assets: Res<Assets<LdtkProject>>,
    level_data: Res<Levels>,
) {
    if let Some(map) = map_assets.get(&level_data.level1) {
        if let Some((level_entity, tile_storage, tilemap_gridsize, _)) = q_tile
            .iter()
            .filter(|(_, _, _, l)| l.identifier == "Tiles")
            .next()
        {
            let tile_size = vec2(tilemap_gridsize.x, tilemap_gridsize.y);
            let level_size = (tile_storage.size.x, tile_storage.size.y);
            let mut cells: HashMap<(i32, i32), f32> = HashMap::new();
            for y in 0..level_size.1 {
                for x in 0..level_size.0 {
                    let tile_position = TilePos::new(x, y);
                    if let Some(tile) = tile_storage.checked_get(&tile_position) {
                        if let Ok(tile_metadata) = data.get(tile) {
                            match tile_metadata.data.as_str() {
                                "collider" => {
                                    cells.insert((x as i32, y as i32), tile_size.y);
                                }
                                "half_collider" => {
                                    cells.insert((x as i32, y as i32), tile_size.y * 0.5);
                                }
                                "quarter_collider" => {
                                    cells.insert((x as i32, y as i32), tile_size.y * 0.25);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }

            let mut builder = TiledCollisionBuilder::default();

            for y in 0..level_size.1 as i32 {
                for x in 0..level_size.0 as i32 {
                    let pos = vec2(x as f32 * tile_size.x, y as f32 * tile_size.y);
                    let cell = cells.get(&(x, y));
                    match (cell.is_some(), builder.building) {
                        (true, false) => {
                            builder.begin(pos, tile_size, *cell.unwrap()); //We checked if it's some first.
                        }
                        (true, true) => {
                            builder.extend();
                        }
                        (false, false) => {}
                        (false, true) => {
                            cmds.spawn(builder.build()).set_parent(level_entity);
                        }
                    }
                }
                if builder.building {
                    cmds.spawn(builder.build()).set_parent(level_entity);
                }
            }
            cmds.trigger(StartBackgroundMusic);
            let data = level_data.level1.path();
            info!("Finished Building Colliders for level. {:?}", data);
        }
    }
}

pub fn spawn_flags(
    mut start: Query<(Entity, &mut Handle<Image>, &mut Sprite), (Added<Start>, Without<Finish>)>,
    mut finish: Query<(Entity, &mut Handle<Image>, &mut Sprite), (Added<Finish>, Without<Start>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("flag_red_green.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(125, 250), 4, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    for (entity, mut image, mut sprite) in start.iter_mut() {
        *image = texture.clone();
        sprite.custom_size = Some(vec2(16.0, 32.0));
        commands.entity(entity).insert(TextureAtlas {
            index: 4,
            layout: texture_atlas_layout.clone(),
        });
    }
    for (entity, mut image, mut sprite) in finish.iter_mut() {
        *image = texture.clone();
        sprite.custom_size = Some(vec2(16.0, 32.0));
        commands.entity(entity).insert(TextureAtlas {
            index: 0,
            layout: texture_atlas_layout.clone(),
        });
    }
}
pub fn spawn_player(
    mut new_player: Query<(Entity, &mut Sprite, &mut Handle<Image>), Added<PlayerMarker>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (player, mut sprite, mut image) in new_player.iter_mut() {
        let texture = assets.load("character.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 3, 2, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        *image = texture;
        sprite.anchor = Anchor::Custom(vec2(0.0, -0.25));
        commands
            .entity(player)
            .insert((
                Follow,
                SideChecks::default(),
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
}
