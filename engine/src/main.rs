use std::{collections::HashSet, fmt::Write};

use macroquad::prelude::*;

const WORLD_SIZE: usize = 64;
const WORLD_EDGE: f32 = WORLD_SIZE as f32;
const WORLD_BATCH_SIZE: usize = 16;
const DRAW_CALL_MAX_VERTICES: usize = 10_000;
const DRAW_CALL_MAX_INDICES: usize = 5_000;
const MAX_BLOCK_HEIGHT: i32 = 16;
const PLAYER_EYE_HEIGHT: f32 = 2.1;
const MOVE_SPEED: f32 = 5.0;
const LOOK_SPEED: f32 = 0.0025;
const REACH: f32 = 6.0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

struct Player {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Player {
    fn new() -> Self {
        Self {
            position: vec3(WORLD_EDGE / 2.0, PLAYER_EYE_HEIGHT, WORLD_EDGE / 2.0),
            yaw: 0.0,
            pitch: -0.35,
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

fn make_flat_world() -> HashSet<BlockPos> {
    let mut blocks = HashSet::with_capacity(WORLD_SIZE * WORLD_SIZE);
    for x in 0..WORLD_SIZE as i32 {
        for z in 0..WORLD_SIZE as i32 {
            blocks.insert(BlockPos { x, y: 0, z });
        }
    }
    blocks
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

fn make_world_meshes(blocks: &HashSet<BlockPos>) -> Vec<Mesh> {
    let mut meshes = Vec::new();
    let batch_size = WORLD_BATCH_SIZE as i32;

    for batch_x in (0..WORLD_SIZE as i32).step_by(WORLD_BATCH_SIZE) {
        for batch_z in (0..WORLD_SIZE as i32).step_by(WORLD_BATCH_SIZE) {
            let mut vertices = Vec::with_capacity(2048);
            let mut indices = Vec::with_capacity(3072);

            for block in blocks.iter().filter(|block| {
                block.x >= batch_x
                    && block.x < batch_x + batch_size
                    && block.z >= batch_z
                    && block.z < batch_z + batch_size
            }) {
                let x = block.x as f32;
                let y = block.y as f32;
                let z = block.z as f32;
                let grass = if ((block.x + block.z) & 1) == 0 {
                    Color::from_rgba(78, 154, 52, 255)
                } else {
                    Color::from_rgba(70, 145, 46, 255)
                };
                let dirt = Color::from_rgba(134, 96, 67, 255);
                let dark_dirt = Color::from_rgba(105, 75, 53, 255);
                let faces = [
                    (
                        BlockPos {
                            y: block.y + 1,
                            ..*block
                        },
                        [
                            vec3(x, y + 1.0, z),
                            vec3(x + 1.0, y + 1.0, z),
                            vec3(x + 1.0, y + 1.0, z + 1.0),
                            vec3(x, y + 1.0, z + 1.0),
                        ],
                        grass,
                    ),
                    (
                        BlockPos {
                            y: block.y - 1,
                            ..*block
                        },
                        [
                            vec3(x, y, z + 1.0),
                            vec3(x + 1.0, y, z + 1.0),
                            vec3(x + 1.0, y, z),
                            vec3(x, y, z),
                        ],
                        dark_dirt,
                    ),
                    (
                        BlockPos {
                            x: block.x - 1,
                            ..*block
                        },
                        [
                            vec3(x, y, z),
                            vec3(x, y + 1.0, z),
                            vec3(x, y + 1.0, z + 1.0),
                            vec3(x, y, z + 1.0),
                        ],
                        dirt,
                    ),
                    (
                        BlockPos {
                            x: block.x + 1,
                            ..*block
                        },
                        [
                            vec3(x + 1.0, y, z + 1.0),
                            vec3(x + 1.0, y + 1.0, z + 1.0),
                            vec3(x + 1.0, y + 1.0, z),
                            vec3(x + 1.0, y, z),
                        ],
                        dirt,
                    ),
                    (
                        BlockPos {
                            z: block.z - 1,
                            ..*block
                        },
                        [
                            vec3(x + 1.0, y, z),
                            vec3(x + 1.0, y + 1.0, z),
                            vec3(x, y + 1.0, z),
                            vec3(x, y, z),
                        ],
                        dirt,
                    ),
                    (
                        BlockPos {
                            z: block.z + 1,
                            ..*block
                        },
                        [
                            vec3(x, y, z + 1.0),
                            vec3(x, y + 1.0, z + 1.0),
                            vec3(x + 1.0, y + 1.0, z + 1.0),
                            vec3(x + 1.0, y, z + 1.0),
                        ],
                        dirt,
                    ),
                ];

                for (neighbor, points, color) in faces {
                    if blocks.contains(&neighbor) {
                        continue;
                    }
                    if vertices.len() + 4 > DRAW_CALL_MAX_VERTICES
                        || indices.len() + 6 > DRAW_CALL_MAX_INDICES
                    {
                        meshes.push(Mesh {
                            vertices: std::mem::take(&mut vertices),
                            indices: std::mem::take(&mut indices),
                            texture: None,
                        });
                        vertices = Vec::with_capacity(2048);
                        indices = Vec::with_capacity(3072);
                    }
                    push_quad(&mut vertices, &mut indices, points, color);
                }
            }

            if !vertices.is_empty() {
                meshes.push(Mesh {
                    vertices,
                    indices,
                    texture: None,
                });
            }
        }
    }

    meshes
}

fn raycast_blocks(
    blocks: &HashSet<BlockPos>,
    origin: Vec3,
    direction: Vec3,
) -> Option<(BlockPos, BlockPos)> {
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
        if blocks.contains(&current) {
            return Some((current, previous));
        }
        previous = current;
    }

    None
}

fn block_overlaps_player(block: BlockPos, player: Vec3) -> bool {
    let horizontal_radius = 0.3;
    let player_feet = player.y - PLAYER_EYE_HEIGHT;
    (block.x as f32) < player.x + horizontal_radius
        && (block.x as f32) + 1.0 > player.x - horizontal_radius
        && (block.z as f32) < player.z + horizontal_radius
        && (block.z as f32) + 1.0 > player.z - horizontal_radius
        && (block.y as f32) < player.y
        && (block.y as f32) + 1.0 > player_feet
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = Player::new();
    let mut blocks = make_flat_world();
    let mut world_meshes = make_world_meshes(&blocks);
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
        if movement.length_squared() > 0.0 {
            player.position += movement.normalize() * MOVE_SPEED * frame_time;
        }

        player.position.x = player.position.x.clamp(0.2, WORLD_EDGE - 0.2);
        player.position.z = player.position.z.clamp(0.2, WORLD_EDGE - 0.2);

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
            if let Some((hit, adjacent)) =
                raycast_blocks(&blocks, player.position, player.forward())
            {
                if is_mouse_button_pressed(MouseButton::Left) {
                    blocks.remove(&hit);
                    world_meshes = make_world_meshes(&blocks);
                } else if is_mouse_button_pressed(MouseButton::Right)
                    && adjacent.y >= 0
                    && adjacent.y < MAX_BLOCK_HEIGHT
                    && adjacent.x >= 0
                    && adjacent.x < WORLD_SIZE as i32
                    && adjacent.z >= 0
                    && adjacent.z < WORLD_SIZE as i32
                    && !block_overlaps_player(adjacent, player.position)
                    && blocks.insert(adjacent)
                {
                    world_meshes = make_world_meshes(&blocks);
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
            raycast_blocks(&blocks, player.position, player.forward()).map(|hit| hit.0)
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
        draw_text(
            if mouse_captured {
                "WASD move | Mouse look | Left break | Right place | ESC release"
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
        make_flat_world, make_world_meshes, raycast_blocks, BlockPos, DRAW_CALL_MAX_INDICES,
        DRAW_CALL_MAX_VERTICES, WORLD_SIZE,
    };
    use macroquad::prelude::vec3;

    #[test]
    fn block_world_batches_fit_macroquad_drawcall_limits() {
        let blocks = make_flat_world();
        let meshes = make_world_meshes(&blocks);

        assert_eq!(blocks.len(), WORLD_SIZE * WORLD_SIZE);
        assert_eq!(meshes.len(), 16);
        assert!(meshes.iter().all(|mesh| {
            mesh.vertices.len() < DRAW_CALL_MAX_VERTICES
                && mesh.indices.len() < DRAW_CALL_MAX_INDICES
        }));
    }

    #[test]
    fn raycast_returns_hit_and_adjacent_empty_block() {
        let blocks = [BlockPos { x: 2, y: 1, z: 2 }].into_iter().collect();
        let hit = raycast_blocks(&blocks, vec3(0.5, 1.5, 2.5), vec3(1.0, 0.0, 0.0));

        assert_eq!(
            hit,
            Some((BlockPos { x: 2, y: 1, z: 2 }, BlockPos { x: 1, y: 1, z: 2 }))
        );
    }
}
