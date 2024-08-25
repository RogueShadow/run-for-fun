use crate::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_ldtk::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::Duration;

#[derive(Resource)]
pub struct LoadedPhysics(Vec<Entity>, Timer);
impl Default for LoadedPhysics {
    fn default() -> Self {
        Self {
            0: vec![],
            1: Timer::new(Duration::from_secs_f32(5.0), TimerMode::Once),
        }
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
            warn!("Not currently building");
            return;
        }
        self.size.x += self.tile_size.x;
    }
}

pub fn build_collision_boxes(
    mut loaded: Local<LoadedPhysics>,
    mut cmds: Commands,
    q_tile: Query<(Entity, &TileStorage, &TilemapGridSize, &LayerMetadata)>,
    data: Query<&TileMetadata>,
    query: Query<(Entity, &Handle<LdtkProject>)>,
    map_assets: Res<Assets<LdtkProject>>,
    asset_server: Res<AssetServer>,
    level_selection: Res<LevelSelection>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
) {
    if !loaded.1.finished() {
        loaded.1.tick(time.delta());
        return;
    }

    for (entity, handle) in query.iter() {
        if loaded.0.contains(&entity) {
            continue;
        }

        if let Some(map) = map_assets.get(handle) {
            let level = map
                .as_standalone()
                .find_raw_level_by_level_selection(&level_selection);
            if level.is_none() {
                info!("Failed to load level.");
                return;
            }
            let level = level.unwrap(); // We just checked, should be fine.
            let entities = level
                .layer_instances
                .as_ref()
                .unwrap()
                .iter()
                .filter(|d| d.identifier == "Entities")
                .next();
            if entities.is_none() {
                info!("Failed to load entities.");
                return;
            }
            let entities = entities.unwrap();
            let tiles = level
                .layer_instances
                .as_ref()
                .unwrap()
                .iter()
                .filter(|d| d.identifier == "Tiles")
                .next();
            if tiles.is_none() {
                info!("Failed to load tile layer");
                return;
            }

            // Get And Set Player Starting
            let player_start = entities
                .entity_instances
                .iter()
                .filter(|x| x.identifier == "Player_start")
                .next()
                .unwrap();

            let mut player_pos = player_start.px.as_vec2();
            player_pos.y *= -1.0;
            player_pos.y += level.px_hei as f32;

            let texture = asset_server.load("character.png");
            let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 3, 2, None, None);
            let texture_atlas_layout = texture_atlas_layouts.add(layout);
            cmds.spawn((
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
                info!("Finished Building Colliders for level.");

                // Get and Spawn Boxes
                let red_block = asset_server.load("red_block.png");
                let box_positions = entities
                    .entity_instances
                    .iter()
                    .filter(|x| x.identifier == "Physics_block")
                    .map(|p| {
                        let mut pos = p.px.as_vec2();
                        pos.y *= -1.0;
                        pos.y += level.px_hei as f32;
                        pos
                    })
                    .collect::<Vec<_>>();

                for p in box_positions {
                    cmds.spawn((
                        SpriteBundle {
                            texture: red_block.clone(),
                            sprite: Sprite { ..default() },
                            transform: Transform::from_xyz(p.x, p.y, 100.0),
                            ..default()
                        },
                        RigidBody::Dynamic,
                        Collider::cuboid(8.0, 8.0),
                        Friction::coefficient(0.5),
                    ));
                }

                //Get and spawn messages.
                let messages = entities
                    .entity_instances
                    .iter()
                    .filter(|m| m.identifier == "Text")
                    .map(|m| {
                        let mut pos = m.px.as_vec2();
                        pos.y *= -1.0;
                        pos.y += level.px_hei as f32;
                        (pos, m.get_string_field("message"))
                    })
                    .collect::<Vec<_>>();
                for (pos, msg) in messages {
                    let msg = match msg {
                        Err(e) => {
                            error!("{:?}", e);
                            "Invalid Message"
                        }
                        Ok(m) => m,
                    };

                    cmds.spawn(Text2dBundle {
                        text: Text {
                            sections: vec![TextSection::new(
                                msg,
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
                        transform: Transform::from_xyz(pos.x, pos.y, 200.0)
                            .with_scale(Vec3::splat(0.1)),
                        ..default()
                    });
                }

                //Get and Spawn start,stop locations
                let start = entities
                    .entity_instances
                    .iter()
                    .filter(|e| e.identifier == "Start")
                    .next();
                let end = entities
                    .entity_instances
                    .iter()
                    .filter(|e| e.identifier == "Finish")
                    .next();
                match (start, end) {
                    (Some(start), Some(end)) => {
                        let mut start_pos = start.px.as_vec2();
                        start_pos.y *= -1.0;
                        start_pos.y += level.px_hei as f32;
                        let start_size = vec2(start.width as f32, start.height as f32);
                        start_pos.x += start_size.x / 2.0;
                        start_pos.y -= start_size.y / 2.0;
                        let mut end_pos = end.px.as_vec2();
                        end_pos.y *= -1.0;
                        end_pos.y += level.px_hei as f32;
                        let end_size = vec2(end.width as f32, end.height as f32);
                        end_pos.x += end_size.x / 2.0;
                        end_pos.y -= end_size.y / 2.0;
                        let texture = asset_server.load("flag_red_green.png");
                        let layout =
                            TextureAtlasLayout::from_grid(UVec2::new(125, 250), 4, 2, None, None);
                        let texture_atlas_layout = texture_atlas_layouts.add(layout);
                        cmds.spawn((
                            Start,
                            Collider::cuboid(start_size.x / 2.0, start_size.y / 2.0),
                            Sensor,
                            ActiveCollisionTypes::KINEMATIC_STATIC,
                            ActiveEvents::COLLISION_EVENTS,
                            SpriteBundle {
                                texture: texture.clone(),
                                transform: Transform::from_xyz(start_pos.x, start_pos.y, 150.0),
                                sprite: Sprite {
                                    custom_size: Some(start_size),
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
                        cmds.spawn((
                            Finish,
                            Collider::cuboid(end_size.x / 2.0, end_size.y / 2.0),
                            Sensor,
                            ActiveCollisionTypes::KINEMATIC_STATIC,
                            ActiveEvents::COLLISION_EVENTS,
                            SpriteBundle {
                                texture: texture.clone(),
                                sprite: Sprite {
                                    custom_size: Some(end_size),
                                    ..default()
                                },
                                transform: Transform::from_xyz(end_pos.x, end_pos.y, 150.0),
                                ..default()
                            },
                            TextureAtlas {
                                layout: texture_atlas_layout.clone(),
                                index: 0,
                            },
                            AnimationIndices::new(0, 3),
                            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                        ));
                        info!("Loaded flags");
                    }
                    _ => {
                        warn!("Level didn't have start and end flags.");
                    }
                }
                loaded.0.push(entity); // Don't try to load again.
            } else {
                error!("Couldn't load level data.");
            }
            //////////
        }
    }
}
