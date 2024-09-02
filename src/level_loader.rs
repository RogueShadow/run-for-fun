use crate::*;
use bevy::prelude::*;
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
            1: Timer::new(Duration::from_secs_f32(1.0), TimerMode::Once),
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
    level_selection: Res<LevelSelection>,
    time: Res<Time>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if !loaded.1.finished() {
        loaded.1.tick(time.delta());
        return;
    }

    let (camera, camera_transform) = camera.single();

    for (entity, handle) in query.iter() {
        if loaded.0.contains(&entity) {
            continue;
        }

        if let Some(map) = map_assets.get(handle) {
            let level = map
                .as_standalone()
                .find_loaded_level_by_level_selection(&level_selection);
            if level.is_none() {
                info!("Failed to load level.");
                return;
            }
            let level = level.unwrap(); // We just checked, should be fine.

            let entities = level
                .layer_instances()
                .iter()
                .filter(|d| d.identifier == "Entities")
                .next();
            if entities.is_none() {
                info!("Failed to load entities.");
                return;
            }
            let entities = entities.unwrap();
            let tiles = level
                .layer_instances()
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
            player_pos.y += *level.px_hei() as f32;

            cmds.trigger(SpawnPlayerEvent {
                position: player_pos,
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

                entities
                    .entity_instances
                    .iter()
                    .filter(|x| x.identifier == "Physics_block")
                    .for_each(|p| {
                        let mut pos = p.px.as_vec2();
                        pos.y *= -1.0;
                        pos.y += *level.px_hei() as f32;
                        cmds.trigger(SpawnBoxEvent { position: pos });
                    });

                //Get and spawn messages.
                let messages = entities
                    .entity_instances
                    .iter()
                    .filter(|m| m.identifier == "Text")
                    .map(|m| {
                        let mut pos = m.px.as_vec2();
                        pos.y *= -1.0;
                        pos.y += *level.px_hei() as f32;
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
                    cmds.trigger(SpawnMessageEvent {
                        position: pos,
                        message: msg.to_string(),
                    })
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
                        start_pos.y += *level.px_hei() as f32;
                        let start_size = vec2(start.width as f32, start.height as f32);
                        start_pos.x += start_size.x / 2.0;
                        start_pos.y -= start_size.y / 2.0;
                        let mut end_pos = end.px.as_vec2();
                        end_pos.y *= -1.0;
                        end_pos.y += *level.px_hei() as f32;
                        let end_size = vec2(end.width as f32, end.height as f32);
                        end_pos.x += end_size.x / 2.0;
                        end_pos.y -= end_size.y / 2.0;

                        cmds.trigger(SpawnFlagEvent {
                            flag: FlagType::Start,
                            position: start_pos,
                            size: start_size,
                        });
                        cmds.trigger(SpawnFlagEvent {
                            flag: FlagType::Finish,
                            position: end_pos,
                            size: end_size,
                        });
                        info!("Loaded flags");
                    }
                    _ => {
                        warn!("Level didn't have start and end flags.");
                    }
                }
                loaded.0.push(entity); // Don't try to load again.

                cmds.trigger(StartBackgroundMusic);
            } else {
                error!("Couldn't load level data.");
            }
            //////////
        }
    }
}
