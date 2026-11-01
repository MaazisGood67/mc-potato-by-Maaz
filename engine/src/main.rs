use std::{collections::HashMap, fmt::Write};

use macroquad::prelude::*;

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

fn push_quad(vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>, points: [Vec3; 4], color: Color) {
    let first_vertex = vertices.len() as u16;
    vertices.extend([
        Vertex::new(points[0].x, points[0].y, points[0].z, 0.0, 0.0, color),
        Vertex::new(points[1].x, points[1].y, points[1].z, 1.0, 0.0, color),
        Vertex::new(points[2].x, points[2].y, points[2].z, 1.0, 1.0, color),
        Vertex::new(points[3].x, points[3].y, points[3].z, 0.0, 1.0, color),
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

struct MeshBuilder {
    meshes: Vec<Mesh>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            meshes: Vec::new(),
            vertices: Vec::with_capacity(4096),
            indices: Vec::with_capacity(4096),
        }
    }

    fn push_face(&mut self, points: [Vec3; 4], color: Color) {
        if self.vertices.len() + 4 > DRAW_CALL_MAX_VERTICES
            || self.indices.len() + 6 > DRAW_CALL_MAX_INDICES
        {
            self.flush();
        }
        push_quad(&mut self.vertices, &mut self.indices, points, color);
    }

    fn flush(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        self.meshes.push(Mesh {
            vertices: std::mem::replace(&mut self.vertices, Vec::with_capacity(4096)),
            indices: std::mem::replace(&mut self.indices, Vec::with_capacity(4096)),
            texture: None,
        });
    }

    fn finish(mut self) -> Vec<Mesh> {
        self.flush();
        self.meshes
    }
}

fn make_world_meshes(world: &World) -> Vec<Mesh> {
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
            .or_insert_with(MeshBuilder::new);

        let x = block.x as f32;
        let y = block.y as f32;
        let z = block.z as f32;
        let (top_color, side_color, bottom_color) = match block_id {
            BlockId::Grass => (
                Color::from_rgba(78, 154, 52, 255),
                Color::from_rgba(134, 96, 67, 255),
                Color::from_rgba(105, 75, 53, 255),
            ),
            BlockId::Dirt => (
                Color::from_rgba(134, 96, 67, 255),
                Color::from_rgba(120, 84, 58, 255),
                Color::from_rgba(105, 75, 53, 255),
            ),
            BlockId::Stone => {
                let stone = Color::from_rgba(125, 125, 125, 255);
                (stone, stone, stone)
            }
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
            ),
        ];

        for (neighbor, points, color) in faces {
            if world.get_block(neighbor.x, neighbor.y, neighbor.z) == BlockId::Air {
                builder.push_face(points, color);
            }
        }
    });

    builders
        .into_values()
        .flat_map(MeshBuilder::finish)
        .collect()
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
    jump_pressed: bool,
    frame_time: f32,
) {
    if jump_pressed && player.on_ground {
        player.vertical_speed = JUMP_SPEED;
        player.on_ground = false;
    }

    if movement.length_squared() > 0.0 {
        let delta = movement.normalize() * MOVE_SPEED * frame_time;
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
    let mut world = World::generate(1_203_001);
    let spawn_x = 0.5;
    let spawn_z = 0.5;
    let spawn_height = world.surface_height(0, 0).unwrap_or(1) as f32;
    let mut player = Player::new(spawn_x, spawn_height, spawn_z);
    let mut world_meshes = make_world_meshes(&world);
    let mut hud_text = String::with_capacity(64);
    let mut mouse_captured = false;
    set_cursor_grab(false);
    show_mouse(true);

    loop {
        let frame_time = get_frame_time().min(0.05);
        let capture_click = !mouse_captured && is_mouse_button_pressed(MouseButton::Left);
        if capture_click {
            mouse_captured = true;
            set_cursor_grab(true);
            show_mouse(false);
        }

        if mouse_captured {
            let mouse_delta = mouse_delta_position();
            player.yaw -= mouse_delta.x * screen_width() * 0.5 * LOOK_SPEED;
            player.pitch = (player.pitch + mouse_delta.y * screen_height() * 0.5 * LOOK_SPEED)
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
        update_player_motion(
            &mut player,
            &world,
            movement,
            is_key_pressed(KeyCode::Space),
            frame_time,
        );

        player.position.x = player.position.x.clamp(WORLD_MIN + 0.2, WORLD_MAX - 0.2);
        player.position.z = player.position.z.clamp(WORLD_MIN + 0.2, WORLD_MAX - 0.2);

        if is_key_pressed(KeyCode::Escape) {
            if mouse_captured {
                mouse_captured = false;
                set_cursor_grab(false);
                show_mouse(true);
            } else {
                break;
            }
        }

        if mouse_captured && !capture_click {
            if let Some((hit, adjacent)) = raycast_blocks(&world, player.position, player.forward())
            {
                if is_mouse_button_pressed(MouseButton::Left) {
                    world.set_block(hit.x, hit.y, hit.z, BlockId::Air);
                    world_meshes = make_world_meshes(&world);
                } else if is_mouse_button_pressed(MouseButton::Right)
                    && adjacent.y >= 0
                    && adjacent.y < CHUNK_HEIGHT as i32
                    && adjacent.x as f32 >= WORLD_MIN
                    && (adjacent.x as f32) < WORLD_MAX
                    && adjacent.z as f32 >= WORLD_MIN
                    && (adjacent.z as f32) < WORLD_MAX
                    && !block_overlaps_player(adjacent, player.position)
                {
                    world.set_block(adjacent.x, adjacent.y, adjacent.z, BlockId::Grass);
                    world_meshes = make_world_meshes(&world);
                }
            }
        }

        clear_background(Color::from_rgba(100, 160, 220, 255));
        set_camera(&Camera3D {
            position: player.position,
            target: player.position + player.forward(),
            up: Vec3::Y,
            fovy: 70.0,
            ..Default::default()
        });
        for world_mesh in &world_meshes {
            draw_mesh(world_mesh);
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
        hud_text.clear();
        let _ = write!(hud_text, "MaazX Potato Edition | FPS: {}", get_fps());
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
        draw_text(
            if mouse_captured {
                "WASD move | SPACE jump | Mouse look | Left break | Right place | ESC release"
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
        collides_at, make_world_meshes, raycast_blocks, update_player_motion, BlockId, BlockPos,
        Player, DRAW_CALL_MAX_INDICES, DRAW_CALL_MAX_VERTICES, PLAYER_EYE_HEIGHT,
    };
    use crate::world::World;
    use macroquad::prelude::{vec3, Vec3};

    #[test]
    fn block_world_batches_fit_macroquad_drawcall_limits() {
        let world = World::generate(1);
        let meshes = make_world_meshes(&world);

        assert!(!meshes.is_empty());
        assert!(meshes.iter().all(|mesh| {
            mesh.vertices.len() < DRAW_CALL_MAX_VERTICES
                && mesh.indices.len() < DRAW_CALL_MAX_INDICES
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
            update_player_motion(&mut player, &world, Vec3::ZERO, false, 1.0 / 60.0);
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

        update_player_motion(&mut player, &world, Vec3::ZERO, true, 1.0 / 60.0);

        assert!(player.position.y > starting_height);
        assert!(!player.on_ground);
    }
}
