use macroquad::prelude::*;

#[macroquad::main("MaazX Potato Edition")]
async fn main() {
    loop {
        clear_background(Color::from_rgba(13, 31, 14, 255));

        draw_text(
            "MaazX Potato Edition v0.1",
            screen_width() / 2.0 - 200.0,
            screen_height() / 2.0 - 40.0,
            32.0,
            WHITE,
        );

        draw_text(&format!("FPS: {}", get_fps()), 20.0, 30.0, 24.0, GREEN);
        draw_text(
            "OpenGL 2.1 Mode - Potato Ready",
            20.0,
            screen_height() - 20.0,
            20.0,
            GREEN,
        );

        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        next_frame().await;
    }
}