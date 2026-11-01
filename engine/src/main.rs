use std::{collections::HashMap, fmt::Write};

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

mod world;
use world::{BlockId, BlockPos, ChunkPos, World, CHUNK_HEIGHT, CHUNK_WIDTH, START_CHUNK_RADIUS};

const WORLD_HALF_EDGE: f32 = (CHUNK_WIDTH as i32 * START_CHUNK_RADIUS) as f32;
const WORLD_MIN: f32 = -WORLD_HALF_EDGE;
const WORLD_MAX: f32 = WORLD_HALF_EDGE;
const DRAW_CALL_MAX_VERTICES: usize = 10_000;
const DRAW_CALL_MAX_INDICES: usize = 5_000;
const PLAYER_EYE_HEIGHT: f32 = 1.62;
const PLAYER_HEIGHT: f32 = 1.8;
const PLAYER_RADIUS: f32 = 0.3;
const MOVE_SPEED: f32 = 5.0;
const GRAVITY: f32 = 22.0;
const JUMP_SPEED: f32 = 8.0;
const LOOK_SPEED: f32 = 0.0025;
const REACH: f32 = 6.0;
const HOTBAR_SLOTS: usize = 9;
const STACK_LIMIT: u16 = 64;
const ATLAS_TILE_SIZE: u16 = 16;
const ATLAS_COLUMNS: u16 = 8;

const TILE_GRASS_TOP: usize = 0;
const TILE_GRASS_SIDE: usize = 1;
const TILE_DIRT: usize = 2;
const TILE_STONE: usize = 3;
const TILE_WATER: usize = 4;
const TILE_SAND: usize = 5;
const TILE_WOOD: usize = 6;
const TILE_LEAVES: usize = 7;
const DIGIT_LABELS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
const SLOT_LABELS: [&str; HOTBAR_SLOTS] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

#[derive(Clone, Copy, PartialEq)]
enum AppScreen {
    MainMenu,
    Settings,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy)]
struct Settings {
    render_distance: i32,
    mouse_sensitivity: f32,
    fullscreen: bool,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
struct ItemStack {
    block: BlockId,
    count: u16,
}

impl ItemStack {
    const EMPTY: Self = Self {
        block: BlockId::Air,
        count: 0,
    };
}

#[derive(Clone, Deserialize, Serialize)]
struct Inventory {
    slots: [ItemStack; HOTBAR_SLOTS],
    selected: usize,
}

impl Inventory {
    fn new() -> Self {
        let mut inventory = Self {
            slots: [ItemStack::EMPTY; HOTBAR_SLOTS],
            selected: 0,
        };
        for (slot, block) in [
            BlockId::Grass,
            BlockId::Dirt,
            BlockId::Stone,
            BlockId::Sand,
            BlockId::Wood,
        ]
        .into_iter()
        .enumerate()
        {
            inventory.slots[slot] = ItemStack { block, count: 32 };
        }
        inventory
    }

    fn add(&mut self, block: BlockId) {
        if block == BlockId::Air || block == BlockId::Water {
            return;
        }
        if let Some(stack) = self
            .slots
            .iter_mut()
            .find(|stack| stack.block == block && stack.count < STACK_LIMIT)
        {
            stack.count += 1;
        } else if let Some(stack) = self.slots.iter_mut().find(|stack| stack.count == 0) {
            *stack = ItemStack { block, count: 1 };
        }
    }

    fn selected_block(&self) -> Option<BlockId> {
        let stack = self.slots[self.selected];
        (stack.count > 0).then_some(stack.block)
    }

    fn consume_selected(&mut self) -> Option<BlockId> {
        let stack = &mut self.slots[self.selected];
        if stack.count == 0 {
            return None;
        }
        stack.count -= 1;
        let block = stack.block;
        if stack.count == 0 {
            stack.block = BlockId::Air;
        }
        Some(block)
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct Survival {
    health: u8,
    hunger: u8,
    exhaustion: f32,
    starvation_time: f32,
}

#[derive(Deserialize, Serialize)]
struct SavedPlayer {
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    vertical_speed: f32,
    on_ground: bool,
}

#[derive(Deserialize, Serialize)]
struct SavedGame {
    version: u32,
    world: World,
    player: SavedPlayer,
    inventory: Inventory,
    survival: Survival,
}

impl SavedPlayer {
    fn capture(player: &Player) -> Self {
        Self {
            position: [player.position.x, player.position.y, player.position.z],
            yaw: player.yaw,
            pitch: player.pitch,
            vertical_speed: player.vertical_speed,
            on_ground: player.on_ground,
        }
    }

    fn restore(self) -> Player {
        Player {
            position: vec3(self.position[0], self.position[1], self.position[2]),
            yaw: self.yaw,
            pitch: self.pitch,
            vertical_speed: self.vertical_speed,
            on_ground: self.on_ground,
        }
    }
}

impl Survival {
    fn new() -> Self {
        Self {
            health: 20,
            hunger: 20,
            exhaustion: 0.0,
            starvation_time: 0.0,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_distance: 1,
            mouse_sensitivity: LOOK_SPEED,
            fullscreen: false,
        }
    }
}

impl Settings {
    fn load() -> Self {
        let Ok(contents) = std::fs::read_to_string(settings_path()) else {
            return Self::default();
        };
        let mut settings = Self::default();
        for line in contents.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key {
                "render_distance" => {
                    if let Ok(value) = value.parse::<i32>() {
                        settings.render_distance = value.clamp(1, START_CHUNK_RADIUS * 2);
                    }
                }
                "mouse_sensitivity" => {
                    if let Ok(value) = value.parse::<f32>() {
                        settings.mouse_sensitivity = value.clamp(0.001, 0.006);
                    }
                }
                "fullscreen" => settings.fullscreen = value == "true",
                _ => {}
            }
        }
        settings
    }

    fn save(self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let contents = format!(
            "render_distance={}\nmouse_sensitivity={}\nfullscreen={}\n",
            self.render_distance, self.mouse_sensitivity, self.fullscreen
        );
        let _ = std::fs::write(path, contents);
    }
}

fn data_directory() -> std::path::PathBuf {
    let data_root = std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config"))
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    data_root.join("MaazX")
}

fn settings_path() -> std::path::PathBuf {
    data_directory().join("settings.ini")
}

fn save_path() -> std::path::PathBuf {
    data_directory().join("worlds").join("main.bin")
}

fn save_game(
    world: &World,
    player: &Player,
    inventory: &Inventory,
    survival: &Survival,
) -> Result<(), String> {
    let path = save_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let save = SavedGame {
        version: 1,
        world: world.clone(),
        player: SavedPlayer::capture(player),
        inventory: inventory.clone(),
        survival: survival.clone(),
    };
    let bytes = bincode::serialize(&save).map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

fn load_game() -> Option<SavedGame> {
    let bytes = std::fs::read(save_path()).ok()?;
    let save: SavedGame = bincode::deserialize(&bytes).ok()?;
    (save.version == 1).then_some(save)
}

fn menu_button(label: &str, bounds: Rect) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let hovered = bounds.contains(vec2(mouse_x, mouse_y));
    draw_rectangle(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        if hovered {
            Color::from_rgba(74, 124, 89, 255)
        } else {
            Color::from_rgba(32, 54, 39, 255)
        },
    );
    draw_rectangle_lines(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        2.0,
        Color::from_rgba(126, 200, 80, 255),
    );
    let text = measure_text(label, None, 24, 1.0);
    draw_text(
        label,
        bounds.x + (bounds.w - text.width) * 0.5,
        bounds.y + (bounds.h + text.height) * 0.5,
        24.0,
        WHITE,
    );
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_menu_background(title: &str, subtitle: &str) {
    clear_background(Color::from_rgba(13, 24, 17, 255));
    let title_size = measure_text(title, None, 42, 1.0);
    draw_text(
        title,
        (screen_width() - title_size.width) * 0.5,
        150.0,
        42.0,
        Color::from_rgba(205, 230, 194, 255),
    );
    let subtitle_size = measure_text(subtitle, None, 18, 1.0);
    draw_text(
        subtitle,
        (screen_width() - subtitle_size.width) * 0.5,
        185.0,
        18.0,
        Color::from_rgba(170, 190, 170, 255),
    );
}

fn daylight_at(time_seconds: f64) -> f32 {
    (((time_seconds / 180.0 * std::f64::consts::TAU).sin() + 1.0) * 0.5) as f32
}

fn sky_color(time_seconds: f64) -> Color {
    let daylight = daylight_at(time_seconds);
    Color::new(
        0.025 + daylight * 0.38,
        0.035 + daylight * 0.56,
        0.10 + daylight * 0.78,
        1.0,
    )
}

fn block_icon_color(block: BlockId) -> Color {
    match block {
        BlockId::Grass => Color::from_rgba(80, 155, 55, 255),
        BlockId::Dirt => Color::from_rgba(134, 96, 67, 255),
        BlockId::Stone => Color::from_rgba(130, 130, 130, 255),
        BlockId::Water => Color::from_rgba(55, 116, 190, 255),
        BlockId::Sand => Color::from_rgba(205, 190, 130, 255),
        BlockId::Wood => Color::from_rgba(110, 75, 43, 255),
        BlockId::Leaves => Color::from_rgba(52, 126, 48, 255),
        BlockId::Air => Color::from_rgba(20, 25, 22, 255),
    }
}

fn draw_hotbar(inventory: &Inventory) {
    let slot_size = 44.0;
    let gap = 4.0;
    let total_width = HOTBAR_SLOTS as f32 * (slot_size + gap) - gap;
    let start_x = (screen_width() - total_width) * 0.5;
    let y = screen_height() - slot_size - 28.0;

    for (index, stack) in inventory.slots.iter().enumerate() {
        let x = start_x + index as f32 * (slot_size + gap);
        let selected = index == inventory.selected;
        draw_rectangle(
            x,
            y,
            slot_size,
            slot_size,
            Color::from_rgba(20, 24, 21, 220),
        );
        draw_rectangle_lines(
            x,
            y,
            slot_size,
            slot_size,
            if selected { 3.0 } else { 1.0 },
            if selected {
                Color::from_rgba(238, 230, 190, 255)
            } else {
                Color::from_rgba(105, 115, 105, 230)
            },
        );
        if stack.count > 0 {
            draw_rectangle(x + 10.0, y + 8.0, 24.0, 24.0, block_icon_color(stack.block));
            let tens = stack.count / 10;
            if tens > 0 {
                draw_text(DIGIT_LABELS[tens as usize], x + 25.0, y + 41.0, 14.0, WHITE);
            }
            draw_text(
                DIGIT_LABELS[(stack.count % 10) as usize],
                x + 32.0,
                y + 41.0,
                14.0,
                WHITE,
            );
        }
        draw_text(SLOT_LABELS[index], x + 3.0, y + 13.0, 11.0, WHITE);
    }
}

fn draw_survival_bars(survival: &Survival) {
    let bar_width = 12.0;
    let gap = 3.0;
    let top = screen_height() - 112.0;
    for index in 0..10 {
        let x = 20.0 + index as f32 * (bar_width + gap);
        let health_value = survival.health as i32 - index * 2;
        let hunger_value = survival.hunger as i32 - index * 2;
        draw_rectangle(x, top, bar_width, 12.0, Color::from_rgba(50, 35, 35, 220));
        if health_value > 0 {
            draw_rectangle(
                x,
                top,
                if health_value == 1 {
                    bar_width * 0.5
                } else {
                    bar_width
                },
                12.0,
                Color::from_rgba(205, 55, 48, 255),
            );
        }

        let hunger_x = screen_width() - 20.0 - (index as f32 + 1.0) * (bar_width + gap);
        draw_rectangle(
            hunger_x,
            top,
            bar_width,
            12.0,
            Color::from_rgba(55, 48, 32, 220),
        );
        if hunger_value > 0 {
            draw_rectangle(
                hunger_x,
                top,
                if hunger_value == 1 {
                    bar_width * 0.5
                } else {
                    bar_width
                },
                12.0,
                Color::from_rgba(215, 164, 55, 255),
            );
        }
    }
}

fn update_survival(survival: &mut Survival, moving: bool, sprinting: bool, delta_time: f32) {
    if moving {
        survival.exhaustion += if sprinting { 0.8 } else { 0.15 } * delta_time;
    }
    while survival.exhaustion >= 6.0 && survival.hunger > 0 {
        survival.exhaustion -= 6.0;
        survival.hunger -= 1;
    }

    if survival.hunger == 0 {
        survival.starvation_time += delta_time;
        if survival.starvation_time >= 4.0 {
            survival.starvation_time = 0.0;
            survival.health = survival.health.saturating_sub(1);
        }
    } else {
        survival.starvation_time = 0.0;
    }
}

struct Player {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    vertical_speed: f32,
    on_ground: bool,
}

impl Player {
    fn new(x: f32, surface_height: f32, z: f32) -> Self {
        Self {
            position: vec3(x, surface_height + PLAYER_EYE_HEIGHT, z),
            yaw: 0.0,
            pitch: -0.35,
            vertical_speed: 0.0,
            on_ground: true,
        }
    }

    fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "MaazX Potato Edition v0.1".to_owned(),
        window_width: 1366,
        window_height: 768,
        ..Default::default()
    }
}

fn push_quad(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    points: [Vec3; 4],
    color: Color,
    tile: usize,
) {
    let first_vertex = vertices.len() as u16;
    let atlas_width = (ATLAS_TILE_SIZE * ATLAS_COLUMNS) as f32;
    let u0 = tile as f32 / ATLAS_COLUMNS as f32 + 0.5 / atlas_width;
    let u1 = (tile + 1) as f32 / ATLAS_COLUMNS as f32 - 0.5 / atlas_width;
    let v0 = 0.5 / ATLAS_TILE_SIZE as f32;
    let v1 = 1.0 - v0;
    vertices.extend([
        Vertex::new(points[0].x, points[0].y, points[0].z, u0, v0, color),
        Vertex::new(points[1].x, points[1].y, points[1].z, u1, v0, color),
        Vertex::new(points[2].x, points[2].y, points[2].z, u1, v1, color),
        Vertex::new(points[3].x, points[3].y, points[3].z, u0, v1, color),
    ]);
    indices.extend([
        first_vertex,
        first_vertex + 2,
        first_vertex + 1,
        first_vertex,
        first_vertex + 3,
        first_vertex + 2,
    ]);
}

fn make_atlas_image() -> Image {
    let width = ATLAS_TILE_SIZE * ATLAS_COLUMNS;
    let height = ATLAS_TILE_SIZE;
    let mut image = Image::gen_image_color(width, height, WHITE);
    let base_colors = [
        [78, 154, 52],
        [134, 96, 67],
        [120, 84, 58],
        [125, 125, 125],
        [55, 116, 190],
        [205, 190, 130],
        [110, 75, 43],
        [52, 126, 48],
    ];

    for tile in 0..ATLAS_COLUMNS as usize {
        for y in 0..ATLAS_TILE_SIZE as u32 {
            for x in 0..ATLAS_TILE_SIZE as u32 {
                let selected_tile = if tile == TILE_GRASS_SIDE && y < 4 {
                    TILE_GRASS_TOP
                } else if tile == TILE_GRASS_SIDE {
                    TILE_DIRT
                } else {
                    tile
                };
                let base = base_colors[selected_tile];
                let mut variation = ((x * 7 + y * 11 + tile as u32 * 13) % 17) as i32 - 8;
                if tile == TILE_WOOD && x % 4 == 0 {
                    variation -= 10;
                }
                if tile == TILE_LEAVES && (x + y * 3) % 11 == 0 {
                    variation += 18;
                }
                let channel = |value: u8| (value as i32 + variation).clamp(0, 255) as u8;
                image.set_pixel(
                    tile as u32 * ATLAS_TILE_SIZE as u32 + x,
                    y,
                    Color::from_rgba(channel(base[0]), channel(base[1]), channel(base[2]), 255),
                );
            }
        }
    }
    image
}

fn make_block_atlas() -> Texture2D {
    let texture = Texture2D::from_image(&make_atlas_image());
    texture.set_filter(FilterMode::Nearest);
    texture
}

struct MeshBuilder {
    meshes: Vec<Mesh>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    texture: Option<Texture2D>,
}

struct ChunkMeshes {
    position: ChunkPos,
    meshes: Vec<Mesh>,
}

impl MeshBuilder {
    fn new(texture: Option<Texture2D>) -> Self {
        Self {
            meshes: Vec::new(),
            vertices: Vec::with_capacity(4096),
            indices: Vec::with_capacity(4096),
            texture,
        }
    }

    fn push_face(&mut self, points: [Vec3; 4], color: Color, tile: usize) {
        if self.vertices.len() + 4 > DRAW_CALL_MAX_VERTICES
            || self.indices.len() + 6 > DRAW_CALL_MAX_INDICES
        {
            self.flush();
        }
        push_quad(&mut self.vertices, &mut self.indices, points, color, tile);
    }

    fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        self.meshes.push(Mesh {
            vertices: std::mem::replace(&mut self.vertices, Vec::with_capacity(4096)),
            indices: std::mem::replace(&mut self.indices, Vec::with_capacity(4096)),
            texture: self.texture.clone(),
        });
    }

    fn finish(mut self) -> Vec<Mesh> {
        self.flush();
        self.meshes
    }
}

fn make_world_meshes(world: &World, atlas: Option<&Texture2D>) -> Vec<ChunkMeshes> {
    let mut builders = HashMap::<ChunkPos, MeshBuilder>::new();
    world.for_each_non_air(|block_x, block_y, block_z, block_id| {
        let block = BlockPos {
            x: block_x,
            y: block_y,
            z: block_z,
        };
        let chunk_position = ChunkPos {
            x: block_x.div_euclid(CHUNK_WIDTH as i32),
            z: block_z.div_euclid(CHUNK_WIDTH as i32),
        };
        let builder = builders
            .entry(chunk_position)
            .or_insert_with(|| MeshBuilder::new(atlas.map(Texture2D::weak_clone)));

        let x = block.x as f32;
        let y = block.y as f32;
        let z = block.z as f32;
        let (top_color, side_color, bottom_color, top_tile, side_tile, bottom_tile) = match block_id
        {
            BlockId::Grass => (
                WHITE,
                WHITE,
                WHITE,
                TILE_GRASS_TOP,
                TILE_GRASS_SIDE,
                TILE_DIRT,
            ),
            BlockId::Dirt => (WHITE, WHITE, WHITE, TILE_DIRT, TILE_DIRT, TILE_DIRT),
            BlockId::Stone => (WHITE, WHITE, WHITE, TILE_STONE, TILE_STONE, TILE_STONE),
            BlockId::Water => (WHITE, WHITE, WHITE, TILE_WATER, TILE_WATER, TILE_WATER),
            BlockId::Sand => (WHITE, WHITE, WHITE, TILE_SAND, TILE_SAND, TILE_SAND),
            BlockId::Wood => (WHITE, WHITE, WHITE, TILE_WOOD, TILE_WOOD, TILE_WOOD),
            BlockId::Leaves => (WHITE, WHITE, WHITE, TILE_LEAVES, TILE_LEAVES, TILE_LEAVES),
            BlockId::Air => return,
        };

        let faces = [
            (
                BlockPos {
                    y: block.y + 1,
                    ..block
                },
                [
                    vec3(x, y + 1.0, z),
                    vec3(x + 1.0, y + 1.0, z),
                    vec3(x + 1.0, y + 1.0, z + 1.0),
                    vec3(x, y + 1.0, z + 1.0),
                ],
                top_color,
                top_tile,
            ),
            (
                BlockPos {
                    y: block.y - 1,
                    ..block
                },
                [
                    vec3(x, y, z + 1.0),
                    vec3(x + 1.0, y, z + 1.0),
                    vec3(x + 1.0, y, z),
                    vec3(x, y, z),
                ],
                bottom_color,
                bottom_tile,
            ),
            (
                BlockPos {
                    x: block.x - 1,
                    ..block
                },
                [
                    vec3(x, y, z),
                    vec3(x, y + 1.0, z),
                    vec3(x, y + 1.0, z + 1.0),
                    vec3(x, y, z + 1.0),
                ],
                side_color,
                side_tile,
            ),
            (
                BlockPos {
                    x: block.x + 1,
                    ..block
                },
                [
                    vec3(x + 1.0, y, z + 1.0),
                    vec3(x + 1.0, y + 1.0, z + 1.0),
                    vec3(x + 1.0, y + 1.0, z),
                    vec3(x + 1.0, y, z),
                ],
                side_color,
                side_tile,
            ),
            (
                BlockPos {
                    z: block.z - 1,
                    ..block
                },
                [
                    vec3(x + 1.0, y, z),
                    vec3(x + 1.0, y + 1.0, z),
                    vec3(x, y + 1.0, z),
                    vec3(x, y, z),
                ],
                side_color,
                side_tile,
            ),
            (
                BlockPos {
                    z: block.z + 1,
                    ..block
                },
                [
                    vec3(x, y, z + 1.0),
                    vec3(x, y + 1.0, z + 1.0),
                    vec3(x + 1.0, y + 1.0, z + 1.0),
                    vec3(x + 1.0, y, z + 1.0),
                ],
                side_color,
                side_tile,
            ),
        ];

        for (neighbor, points, color, tile) in faces {
            if world.get_block(neighbor.x, neighbor.y, neighbor.z) == BlockId::Air {
                builder.push_face(points, color, tile);
            }
        }
    });

    builders
        .into_iter()
        .map(|(position, builder)| ChunkMeshes {
            position,
            meshes: builder.finish(),
        })
        .collect()
}

fn chunk_is_visible(chunk: ChunkPos, player: ChunkPos, render_distance: i32) -> bool {
    (chunk.x - player.x).abs() <= render_distance && (chunk.z - player.z).abs() <= render_distance
}

fn raycast_blocks(world: &World, origin: Vec3, direction: Vec3) -> Option<(BlockPos, BlockPos)> {
    let mut previous = BlockPos {
        x: origin.x.floor() as i32,
        y: origin.y.floor() as i32,
        z: origin.z.floor() as i32,
    };

    for step in 1..=(REACH * 40.0) as usize {
        let point = origin + direction * (step as f32 / 40.0);
        let current = BlockPos {
            x: point.x.floor() as i32,
            y: point.y.floor() as i32,
            z: point.z.floor() as i32,
        };
        if current == previous {
            continue;
        }
        if world.get_block(current.x, current.y, current.z) != BlockId::Air {
            return Some((current, previous));
        }
        previous = current;
    }

    None
}

fn block_overlaps_player(block: BlockPos, player: Vec3) -> bool {
    let player_feet = player.y - PLAYER_EYE_HEIGHT;
    (block.x as f32) < player.x + PLAYER_RADIUS
        && (block.x as f32) + 1.0 > player.x - PLAYER_RADIUS
        && (block.z as f32) < player.z + PLAYER_RADIUS
        && (block.z as f32) + 1.0 > player.z - PLAYER_RADIUS
        && (block.y as f32) < player_feet + PLAYER_HEIGHT
        && (block.y as f32) + 1.0 > player_feet
}

fn collides_at(world: &World, position: Vec3) -> bool {
    let feet = position.y - PLAYER_EYE_HEIGHT;
    let head = feet + PLAYER_HEIGHT;
    let min_x = (position.x - PLAYER_RADIUS).floor() as i32;
    let max_x = (position.x + PLAYER_RADIUS - 0.001).floor() as i32;
    let min_y = (feet + 0.001).floor() as i32;
    let max_y = (head - 0.001).floor() as i32;
    let min_z = (position.z - PLAYER_RADIUS).floor() as i32;
    let max_z = (position.z + PLAYER_RADIUS - 0.001).floor() as i32;

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.get_block(x, y, z) != BlockId::Air {
                    return true;
                }
            }
        }
    }

    false
}

fn find_landing_height(
    world: &World,
    player: Vec3,
    previous_feet: f32,
    next_feet: f32,
) -> Option<f32> {
    let min_x = (player.x - PLAYER_RADIUS).floor() as i32;
    let max_x = (player.x + PLAYER_RADIUS - 0.001).floor() as i32;
    let min_z = (player.z - PLAYER_RADIUS).floor() as i32;
    let max_z = (player.z + PLAYER_RADIUS - 0.001).floor() as i32;
    let min_y = (next_feet - 1.0).floor() as i32;
    let max_y = previous_feet.floor() as i32;
    let mut landing_height = None;

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            for y in min_y..=max_y {
                if world.get_block(x, y, z) != BlockId::Air {
                    let top = y as f32 + 1.0;
                    if top <= previous_feet + 0.001
                        && top >= next_feet - 0.001
                        && landing_height.is_none_or(|current| top > current)
                    {
                        landing_height = Some(top);
                    }
                }
            }
        }
    }

    landing_height
}

fn update_player_motion(
    player: &mut Player,
    world: &World,
    movement: Vec3,
    sprinting: bool,
    jump_pressed: bool,
    frame_time: f32,
) {
    if jump_pressed && player.on_ground {
        player.vertical_speed = JUMP_SPEED;
        player.on_ground = false;
    }

    if movement.length_squared() > 0.0 {
        let speed = MOVE_SPEED * if sprinting { 1.5 } else { 1.0 };
        let delta = movement.normalize() * speed * frame_time;
        let next_x = vec3(
            player.position.x + delta.x,
            player.position.y,
            player.position.z,
        );
        if !collides_at(world, next_x) {
            player.position.x = next_x.x;
        }
        let next_z = vec3(
            player.position.x,
            player.position.y,
            player.position.z + delta.z,
        );
        if !collides_at(world, next_z) {
            player.position.z = next_z.z;
        }
    }

    player.vertical_speed -= GRAVITY * frame_time;
    let previous_feet = player.position.y - PLAYER_EYE_HEIGHT;
    let next_y = player.position.y + player.vertical_speed * frame_time;
    let next_feet = next_y - PLAYER_EYE_HEIGHT;

    if player.vertical_speed <= 0.0 {
        if let Some(landing_height) =
            find_landing_height(world, player.position, previous_feet, next_feet)
        {
            player.position.y = landing_height + PLAYER_EYE_HEIGHT;
            player.vertical_speed = 0.0;
            player.on_ground = true;
        } else {
            player.position.y = next_y;
            player.on_ground = false;
        }
    } else {
        let next_position = vec3(player.position.x, next_y, player.position.z);
        if collides_at(world, next_position) {
            player.vertical_speed = 0.0;
        } else {
            player.position.y = next_y;
            player.on_ground = false;
        }
    }

    if player.position.y < -32.0 {
        let spawn_height = world.surface_height(0, 0).unwrap_or(1) as f32;
        player.position = vec3(0.5, spawn_height + PLAYER_EYE_HEIGHT, 0.5);
        player.vertical_speed = 0.0;
        player.on_ground = true;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let block_atlas = make_block_atlas();
    let loaded_game = load_game();
    let (mut world, mut player, mut inventory, mut survival, mut has_started_world) =
        if let Some(save) = loaded_game {
            (
                save.world,
                save.player.restore(),
                save.inventory,
                save.survival,
                true,
            )
        } else {
            let world = World::generate(1_203_001);
            let surface = world.surface_height(0, 0).unwrap_or(1) as f32;
            (
                world,
                Player::new(0.5, surface, 0.5),
                Inventory::new(),
                Survival::new(),
                false,
            )
        };
    let mut world_meshes = make_world_meshes(&world, Some(&block_atlas));
    let mut hud_text = String::with_capacity(64);
    let mut save_message = String::with_capacity(128);
    let mut settings_text = String::with_capacity(96);
    let mut settings = Settings::load();
    set_fullscreen(settings.fullscreen);
    let mut screen = AppScreen::MainMenu;
    let mut settings_return_to = AppScreen::MainMenu;
    let mut mouse_captured = false;
    set_cursor_grab(false);
    show_mouse(true);

    'game: loop {
        let frame_time = get_frame_time().min(0.05);

        if screen == AppScreen::MainMenu {
            set_cursor_grab(false);
            show_mouse(true);
            draw_menu_background("MaazX", "Offline voxel survival");
            let button_x = (screen_width() - 280.0) * 0.5;
            if menu_button("New World", Rect::new(button_x, 250.0, 280.0, 48.0)) {
                let seed = (get_time() * 1000.0) as u32;
                world = World::generate(seed);
                let surface = world.surface_height(0, 0).unwrap_or(1) as f32;
                player = Player::new(0.5, surface, 0.5);
                world_meshes = make_world_meshes(&world, Some(&block_atlas));
                inventory = Inventory::new();
                survival = Survival::new();
                has_started_world = true;
                save_message.clear();
                screen = AppScreen::Playing;
            }
            if has_started_world && menu_button("Continue", Rect::new(button_x, 310.0, 280.0, 48.0))
            {
                screen = AppScreen::Playing;
            }
            if menu_button("Settings", Rect::new(button_x, 370.0, 280.0, 48.0)) {
                settings_return_to = AppScreen::MainMenu;
                screen = AppScreen::Settings;
            }
            if menu_button("Quit", Rect::new(button_x, 430.0, 280.0, 48.0)) {
                break 'game;
            }
            if !save_message.is_empty() {
                draw_text(&save_message, 24.0, screen_height() - 48.0, 16.0, WHITE);
            }
            next_frame().await;
            continue;
        }

        if screen == AppScreen::Settings {
            set_cursor_grab(false);
            show_mouse(true);
            draw_menu_background("Settings", "Performance and controls");
            let center_x = screen_width() * 0.5;
            settings_text.clear();
            let _ = write!(
                settings_text,
                "Render distance: {} chunks",
                settings.render_distance
            );
            let label_size = measure_text(&settings_text, None, 22, 1.0);
            draw_text(
                &settings_text,
                center_x - label_size.width * 0.5,
                246.0,
                22.0,
                WHITE,
            );
            if menu_button("-", Rect::new(center_x - 170.0, 260.0, 56.0, 44.0)) {
                settings.render_distance = (settings.render_distance - 1).max(1);
                settings.save();
            }
            if menu_button("+", Rect::new(center_x + 114.0, 260.0, 56.0, 44.0)) {
                settings.render_distance =
                    (settings.render_distance + 1).min(START_CHUNK_RADIUS * 2);
                settings.save();
            }

            settings_text.clear();
            let _ = write!(
                settings_text,
                "Mouse sensitivity: {:.1}",
                settings.mouse_sensitivity / LOOK_SPEED
            );
            let label_size = measure_text(&settings_text, None, 22, 1.0);
            draw_text(
                &settings_text,
                center_x - label_size.width * 0.5,
                340.0,
                22.0,
                WHITE,
            );
            if menu_button("-", Rect::new(center_x - 170.0, 354.0, 56.0, 44.0)) {
                settings.mouse_sensitivity = (settings.mouse_sensitivity - 0.0005).max(0.001);
                settings.save();
            }
            if menu_button("+", Rect::new(center_x + 114.0, 354.0, 56.0, 44.0)) {
                settings.mouse_sensitivity = (settings.mouse_sensitivity + 0.0005).min(0.006);
                settings.save();
            }
            if menu_button(
                if settings.fullscreen {
                    "Windowed mode"
                } else {
                    "Fullscreen"
                },
                Rect::new(center_x - 140.0, 430.0, 280.0, 44.0),
            ) {
                settings.fullscreen = !settings.fullscreen;
                set_fullscreen(settings.fullscreen);
                settings.save();
            }
            if menu_button("Back", Rect::new(center_x - 140.0, 500.0, 280.0, 44.0))
                || is_key_pressed(KeyCode::Escape)
            {
                screen = settings_return_to;
            }
            next_frame().await;
            continue;
        }

        if screen == AppScreen::Paused {
            set_cursor_grab(false);
            show_mouse(true);
            draw_menu_background("Paused", "The world is waiting");
            let button_x = (screen_width() - 280.0) * 0.5;
            if menu_button("Resume", Rect::new(button_x, 270.0, 280.0, 48.0)) {
                screen = AppScreen::Playing;
            }
            if menu_button("Settings", Rect::new(button_x, 330.0, 280.0, 48.0)) {
                settings_return_to = AppScreen::Paused;
                screen = AppScreen::Settings;
            }
            if menu_button("Save & Title Menu", Rect::new(button_x, 390.0, 280.0, 48.0)) {
                match save_game(&world, &player, &inventory, &survival) {
                    Ok(()) => {
                        save_message = "World saved".to_owned();
                        screen = AppScreen::MainMenu;
                    }
                    Err(error) => save_message = format!("Save failed: {error}"),
                }
            }
            if menu_button("Quit", Rect::new(button_x, 450.0, 280.0, 48.0)) {
                break 'game;
            }
            if !save_message.is_empty() {
                draw_text(&save_message, 24.0, screen_height() - 48.0, 16.0, WHITE);
            }
            next_frame().await;
            continue;
        }

        if screen == AppScreen::GameOver {
            set_cursor_grab(false);
            show_mouse(true);
            draw_menu_background("You were defeated", "Your saved world is still available");
            let button_x = (screen_width() - 280.0) * 0.5;
            if menu_button("Respawn", Rect::new(button_x, 300.0, 280.0, 48.0)) {
                let surface = world.surface_height(0, 0).unwrap_or(1) as f32;
                player = Player::new(0.5, surface, 0.5);
                survival = Survival::new();
                screen = AppScreen::Playing;
            }
            if menu_button("Save & Title Menu", Rect::new(button_x, 360.0, 280.0, 48.0)) {
                match save_game(&world, &player, &inventory, &survival) {
                    Ok(()) => {
                        save_message = "World saved".to_owned();
                        screen = AppScreen::MainMenu;
                    }
                    Err(error) => save_message = format!("Save failed: {error}"),
                }
            }
            next_frame().await;
            continue;
        }

        let capture_click = !mouse_captured && is_mouse_button_pressed(MouseButton::Left);
        if capture_click {
            mouse_captured = true;
            set_cursor_grab(true);
            show_mouse(false);
        }

        if mouse_captured {
            let mouse_delta = mouse_delta_position();
            player.yaw -= mouse_delta.x * screen_width() * 0.5 * settings.mouse_sensitivity;
            player.pitch = (player.pitch
                + mouse_delta.y * screen_height() * 0.5 * settings.mouse_sensitivity)
                .clamp(-1.45, 1.45);
        }

        let forward = vec3(player.yaw.cos(), 0.0, player.yaw.sin());
        let right = vec3(-player.yaw.sin(), 0.0, player.yaw.cos());
        let mut movement = Vec3::ZERO;

        if is_key_down(KeyCode::W) {
            movement += forward;
        }
        if is_key_down(KeyCode::S) {
            movement -= forward;
        }
        if is_key_down(KeyCode::D) {
            movement += right;
        }
        if is_key_down(KeyCode::A) {
            movement -= right;
        }
        let moving = movement.length_squared() > 0.0;
        let sprinting = moving && is_key_down(KeyCode::LeftShift);
        update_player_motion(
            &mut player,
            &world,
            movement,
            sprinting,
            is_key_pressed(KeyCode::Space),
            frame_time,
        );
        update_survival(&mut survival, moving, sprinting, frame_time);
        if survival.health == 0 {
            mouse_captured = false;
            set_cursor_grab(false);
            show_mouse(true);
            screen = AppScreen::GameOver;
        }

        let hotbar_keys = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];
        for (slot, key) in hotbar_keys.into_iter().enumerate() {
            if is_key_pressed(key) {
                inventory.selected = slot;
            }
        }
        let (_, wheel_y) = mouse_wheel();
        if wheel_y > 0.0 {
            inventory.selected = (inventory.selected + HOTBAR_SLOTS - 1) % HOTBAR_SLOTS;
        } else if wheel_y < 0.0 {
            inventory.selected = (inventory.selected + 1) % HOTBAR_SLOTS;
        }

        player.position.x = player.position.x.clamp(WORLD_MIN + 0.2, WORLD_MAX - 0.2);
        player.position.z = player.position.z.clamp(WORLD_MIN + 0.2, WORLD_MAX - 0.2);

        if is_key_pressed(KeyCode::Escape) {
            if mouse_captured {
                mouse_captured = false;
                set_cursor_grab(false);
                show_mouse(true);
            }
            screen = AppScreen::Paused;
        }

        if mouse_captured && !capture_click {
            if let Some((hit, adjacent)) = raycast_blocks(&world, player.position, player.forward())
            {
                if is_mouse_button_pressed(MouseButton::Left) {
                    inventory.add(world.get_block(hit.x, hit.y, hit.z));
                    world.set_block(hit.x, hit.y, hit.z, BlockId::Air);
                    world_meshes = make_world_meshes(&world, Some(&block_atlas));
                } else if is_mouse_button_pressed(MouseButton::Right)
                    && adjacent.y >= 0
                    && adjacent.y < CHUNK_HEIGHT as i32
                    && adjacent.x as f32 >= WORLD_MIN
                    && (adjacent.x as f32) < WORLD_MAX
                    && adjacent.z as f32 >= WORLD_MIN
                    && (adjacent.z as f32) < WORLD_MAX
                    && !block_overlaps_player(adjacent, player.position)
                    && inventory.selected_block().is_some()
                {
                    if let Some(block) = inventory.consume_selected() {
                        world.set_block(adjacent.x, adjacent.y, adjacent.z, block);
                        world_meshes = make_world_meshes(&world, Some(&block_atlas));
                    }
                }
            }
        }

        clear_background(sky_color(get_time()));
        set_camera(&Camera3D {
            position: player.position,
            target: player.position + player.forward(),
            up: Vec3::Y,
            fovy: 70.0,
            ..Default::default()
        });
        let player_chunk = ChunkPos {
            x: (player.position.x.floor() as i32).div_euclid(CHUNK_WIDTH as i32),
            z: (player.position.z.floor() as i32).div_euclid(CHUNK_WIDTH as i32),
        };
        let mut visible_chunk_count = 0;
        for chunk_mesh in &world_meshes {
            if chunk_is_visible(chunk_mesh.position, player_chunk, settings.render_distance) {
                visible_chunk_count += 1;
                for world_mesh in &chunk_mesh.meshes {
                    draw_mesh(world_mesh);
                }
            }
        }

        let targeted_block = if mouse_captured {
            raycast_blocks(&world, player.position, player.forward()).map(|hit| hit.0)
        } else {
            None
        };
        if let Some(block) = targeted_block {
            draw_cube_wires(
                vec3(
                    block.x as f32 + 0.5,
                    block.y as f32 + 0.5,
                    block.z as f32 + 0.5,
                ),
                vec3(1.02, 1.02, 1.02),
                WHITE,
            );
        }

        set_default_camera();
        draw_survival_bars(&survival);
        draw_hotbar(&inventory);
        hud_text.clear();
        let _ = write!(
            hud_text,
            "MaazX | FPS: {} | Chunks: {}/{}",
            get_fps(),
            visible_chunk_count,
            world_meshes.len()
        );
        draw_text(&hud_text, 10.0, 25.0, 22.0, WHITE);
        hud_text.clear();
        let _ = write!(
            hud_text,
            "XYZ: {:.1} {:.1} {:.1}",
            player.position.x, player.position.y, player.position.z
        );
        draw_text(&hud_text, 10.0, 50.0, 18.0, WHITE);
        hud_text.clear();
        let _ = write!(hud_text, "World seed: {}", world.seed());
        draw_text(&hud_text, 10.0, 72.0, 16.0, WHITE);
        hud_text.clear();
        let _ = write!(
            hud_text,
            "Daylight: {:.0}%",
            daylight_at(get_time()) * 100.0
        );
        draw_text(&hud_text, 10.0, 92.0, 16.0, WHITE);
        draw_text(
            if mouse_captured {
                "WASD move | SHIFT sprint | SPACE jump | Left break | Right place | ESC pause"
            } else {
                "Click to capture mouse | WASD move | ESC quit"
            },
            10.0,
            screen_height() - 15.0,
            16.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        chunk_is_visible, collides_at, daylight_at, make_atlas_image, make_world_meshes,
        raycast_blocks, sky_color, update_player_motion, update_survival, BlockId, BlockPos,
        ChunkPos, Inventory, Player, SavedGame, SavedPlayer, Survival, ATLAS_COLUMNS,
        ATLAS_TILE_SIZE, DRAW_CALL_MAX_INDICES, DRAW_CALL_MAX_VERTICES, PLAYER_EYE_HEIGHT,
    };
    use crate::world::World;
    use macroquad::prelude::{vec3, Vec3};

    #[test]
    fn block_world_batches_fit_macroquad_drawcall_limits() {
        let world = World::generate(1);
        let meshes = make_world_meshes(&world, None);

        assert!(!meshes.is_empty());
        assert!(meshes.iter().all(|chunk| {
            chunk.meshes.iter().all(|mesh| {
                mesh.vertices.len() < DRAW_CALL_MAX_VERTICES
                    && mesh.indices.len() < DRAW_CALL_MAX_INDICES
            })
        }));
    }

    #[test]
    fn raycast_returns_hit_and_adjacent_empty_block() {
        let mut world = World::empty(1);
        world.set_block(2, 1, 2, BlockId::Stone);
        let hit = raycast_blocks(&world, vec3(0.5, 1.5, 2.5), vec3(1.0, 0.0, 0.0));

        assert_eq!(
            hit,
            Some((BlockPos { x: 2, y: 1, z: 2 }, BlockPos { x: 1, y: 1, z: 2 }))
        );
    }

    #[test]
    fn render_distance_hides_distant_chunks() {
        let player_chunk = ChunkPos { x: 0, z: 0 };

        assert!(chunk_is_visible(ChunkPos { x: 1, z: -1 }, player_chunk, 1));
        assert!(!chunk_is_visible(ChunkPos { x: 2, z: 0 }, player_chunk, 1));
    }

    #[test]
    fn player_stands_on_top_of_floor_without_overlap() {
        let mut world = World::empty(1);
        world.set_block(4, 0, 4, BlockId::Grass);
        let player = vec3(4.5, PLAYER_EYE_HEIGHT + 1.0, 4.5);

        assert!(!collides_at(&world, player));
    }

    #[test]
    fn player_cannot_overlap_a_block_wall() {
        let mut world = World::empty(1);
        world.set_block(1, 1, 1, BlockId::Stone);
        let player = vec3(0.8, PLAYER_EYE_HEIGHT + 1.0, 1.5);

        assert!(collides_at(&world, player));
        assert!(super::block_overlaps_player(
            BlockPos { x: 1, y: 1, z: 1 },
            player
        ));
    }

    #[test]
    fn new_player_spawns_grounded_above_floor() {
        let world = World::generate(1);
        let height = world.surface_height(0, 0).unwrap() as f32;
        let player = Player::new(0.5, height, 0.5);

        assert!(player.on_ground);
        assert!((player.position.y - PLAYER_EYE_HEIGHT - height).abs() < 0.0001);
    }

    #[test]
    fn gravity_lands_player_on_floor() {
        let mut world = World::empty(1);
        world.set_block(4, 0, 4, BlockId::Grass);
        let mut player = Player::new(4.5, 1.0, 4.5);
        player.position.y += 1.0;
        player.on_ground = false;

        for _ in 0..120 {
            update_player_motion(&mut player, &world, Vec3::ZERO, false, false, 1.0 / 60.0);
        }

        assert!(player.on_ground);
        assert!((player.position.y - PLAYER_EYE_HEIGHT - 1.0).abs() < 0.001);
    }

    #[test]
    fn jump_moves_grounded_player_upward() {
        let mut world = World::empty(1);
        world.set_block(4, 0, 4, BlockId::Grass);
        let mut player = Player::new(4.5, 1.0, 4.5);
        let starting_height = player.position.y;

        update_player_motion(&mut player, &world, Vec3::ZERO, false, true, 1.0 / 60.0);

        assert!(player.position.y > starting_height);
        assert!(!player.on_ground);
    }

    #[test]
    fn atlas_uses_small_pixel_art_tiles() {
        let atlas = make_atlas_image();

        assert_eq!(atlas.width(), (ATLAS_TILE_SIZE * ATLAS_COLUMNS) as usize);
        assert_eq!(atlas.height(), ATLAS_TILE_SIZE as usize);
        assert_ne!(
            atlas.get_pixel(1, 1),
            atlas.get_pixel(ATLAS_TILE_SIZE as u32 + 1, 1)
        );
    }

    #[test]
    fn inventory_consumes_and_collects_blocks() {
        let mut inventory = Inventory::new();
        assert_eq!(inventory.selected_block(), Some(BlockId::Grass));
        assert_eq!(inventory.consume_selected(), Some(BlockId::Grass));
        assert_eq!(inventory.slots[0].count, 31);

        inventory.add(BlockId::Stone);
        assert_eq!(inventory.slots[2].count, 33);
    }

    #[test]
    fn sprinting_uses_hunger_and_starvation_damages_health() {
        let mut survival = Survival::new();
        update_survival(&mut survival, true, true, 10.0);
        assert_eq!(survival.hunger, 19);

        survival.hunger = 0;
        update_survival(&mut survival, false, false, 4.0);
        assert_eq!(survival.health, 19);
    }

    #[test]
    fn day_night_cycle_changes_sky_color() {
        assert!(daylight_at(45.0) > daylight_at(135.0));
        assert_ne!(sky_color(45.0), sky_color(135.0));
    }

    #[test]
    fn save_roundtrip_preserves_world_player_inventory_and_survival() {
        let mut world = World::generate(42);
        world.set_block(3, 100, -2, BlockId::Wood);
        let player = Player::new(3.5, 40.0, -1.5);
        let mut inventory = Inventory::new();
        inventory.selected = 2;
        inventory.slots[2].count = 7;
        let mut survival = Survival::new();
        survival.health = 14;
        survival.hunger = 9;
        let original = SavedGame {
            version: 1,
            world,
            player: SavedPlayer::capture(&player),
            inventory,
            survival,
        };

        let bytes = bincode::serialize(&original).unwrap();
        let restored: SavedGame = bincode::deserialize(&bytes).unwrap();

        assert_eq!(restored.world.seed(), 42);
        assert_eq!(restored.world.get_block(3, 100, -2), BlockId::Wood);
        assert_eq!(
            restored.player.position,
            [player.position.x, player.position.y, player.position.z]
        );
        assert_eq!(restored.inventory.selected, 2);
        assert_eq!(restored.inventory.slots[2].count, 7);
        assert_eq!(restored.survival.health, 14);
        assert_eq!(restored.survival.hunger, 9);
    }
}
