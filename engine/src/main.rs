use std::fmt::Write;

use macroquad::prelude::*;

const WORLD_SIZE: usize = 64;
const WORLD_EDGE: f32 = WORLD_SIZE as f32;
const PLAYER_EYE_HEIGHT: f32 = 2.1;
const MOVE_SPEED: f32 = 5.0;
const LOOK_SPEED: f32 = 0.0025;

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
            pitch: 0.0,
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

fn make_flat_world() -> Mesh {
    let mut vertices = Vec::with_capacity(WORLD_SIZE * WORLD_SIZE * 4);
    let mut indices = Vec::with_capacity(WORLD_SIZE * WORLD_SIZE * 6);

    for x in 0..WORLD_SIZE {
        for z in 0..WORLD_SIZE {
            let color = if ((x + z) & 1) == 0 {
                Color::from_rgba(34, 139, 34, 255)
            } else {
                Color::from_rgba(28, 120, 28, 255)
            };
            let x = x as f32;
            let z = z as f32;
            let first_vertex = vertices.len() as u16;

            vertices.extend([
                Vertex::new(x, 0.5, z, 0.0, 0.0, color),
                Vertex::new(x + 1.0, 0.5, z, 1.0, 0.0, color),
                Vertex::new(x + 1.0, 0.5, z + 1.0, 1.0, 1.0, color),
                Vertex::new(x, 0.5, z + 1.0, 0.0, 1.0, color),
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
    }

    Mesh {
        vertices,
        indices,
        texture: None,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player = Player::new();
    let world_mesh = make_flat_world();
    let mut hud_text = String::with_capacity(64);
    set_cursor_grab(true);
    show_mouse(false);

    loop {
        let frame_time = get_frame_time().min(0.05);
        let mouse_delta = mouse_delta_position();
        player.yaw += mouse_delta.x * LOOK_SPEED;
        player.pitch = (player.pitch - mouse_delta.y * LOOK_SPEED).clamp(-1.5, 1.5);

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
            set_cursor_grab(false);
            show_mouse(true);
            break;
        }

        clear_background(Color::from_rgba(100, 160, 220, 255));
        set_camera(&Camera3D {
            position: player.position,
            target: player.position + player.forward(),
            up: Vec3::Y,
            fovy: 70.0,
            ..Default::default()
        });
        draw_mesh(&world_mesh);

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
            "WASD move | Mouse look | ESC quit",
            10.0,
            screen_height() - 15.0,
            16.0,
            Color::from_rgba(200, 200, 200, 255),
        );

        next_frame().await;
    }
}
