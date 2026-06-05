use codimate_animation::Playable;
use codimate_core::*;
use codimate_layout::{layout_scene, Viewport};
use codimate_render::{inject_debug_metadata, rasterize, render_frame, Bitmap, RenderCommand};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

pub struct PreviewConfig {
    pub fps: f32,
    pub viewport: Viewport,
    pub show_debug: bool,
    pub title: String,
}

impl PreviewConfig {
    pub fn new(fps: f32, viewport: Viewport) -> Self {
        Self {
            fps,
            viewport,
            show_debug: true,
            title: "Codimate Preview".into(),
        }
    }
    pub fn show_debug(mut self, v: bool) -> Self {
        self.show_debug = v;
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }
}

pub struct Previewer {
    playable: Box<dyn Playable>,
    config: PreviewConfig,
}

impl Previewer {
    pub fn new(playable: Box<dyn Playable>, config: PreviewConfig) -> Self {
        Self { playable, config }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let c = &self.config;
        let w = c.viewport.width as usize;
        let h = c.viewport.height as usize;
        let frame_dt = Duration::from_secs_f32(1.0 / c.fps);
        let total = self.playable.duration();

        let mut window = Window::new(&c.title, w, h, WindowOptions::default())
            .map_err(|e| format!("minifb: {e}"))?;
        let mut buf = vec![0u32; w * h];
        let mut playing = true;
        let mut elapsed = 0.0f32;
        let mut last_frame = Instant::now();
        let mut need_render = true;
        let mut prev_keys: Vec<Key> = vec![];

        while window.is_open() && !window.is_key_down(Key::Escape) && !window.is_key_down(Key::Q) {
            let held = window.get_keys();
            for k in &held {
                if !prev_keys.contains(k) {
                    match k {
                        Key::Space => playing = !playing,
                        Key::Left => {
                            let dt = 1.0 / c.fps;
                            elapsed = (elapsed - dt).max(0.0);
                            need_render = true;
                        }
                        Key::Right => {
                            let dt = 1.0 / c.fps;
                            elapsed = (elapsed + dt).min(total);
                            need_render = true;
                        }
                        _ => {}
                    }
                }
            }
            prev_keys = held;

            if playing {
                let now = Instant::now();
                if now >= last_frame + frame_dt {
                    elapsed += frame_dt.as_secs_f32();
                    if elapsed > total {
                        elapsed = 0.0;
                    }
                    last_frame = now;
                    need_render = true;
                }
            }

            if need_render {
                need_render = false;
                let bm = self.render_bitmap(elapsed, playing);
                render_into(&bm, &mut buf);
            }
            window.update_with_buffer(&buf, w, h)?;

            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(())
    }

    fn render_bitmap(&self, elapsed: f32, playing: bool) -> Bitmap {
        let scene = self.playable.resolve_at(elapsed);
        let layout = layout_scene(scene, self.config.viewport);
        let mut frame = render_frame(self.playable.name(), elapsed, &layout);
        if self.config.show_debug {
            inject_debug_metadata(&mut frame);
            frame.commands.push(RenderCommand::Text {
                x: 8.0,
                y: self.config.viewport.height - 12.0,
                text: "Space:play/pause  ←→:step  C:ss  Q/Esc:quit".into(),
                font_size: 12.0,
                fill: Color {
                    r: 0.6,
                    g: 0.6,
                    b: 0.6,
                    a: 1.0,
                },
                align: TextAlign::Left,
            });
            if !playing {
                frame.commands.push(RenderCommand::Text {
                    x: self.config.viewport.width * 0.5,
                    y: self.config.viewport.height * 0.5 - 12.0,
                    text: "[ PAUSED ]".into(),
                    font_size: 24.0,
                    fill: Color {
                        r: 1.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    align: TextAlign::Center,
                });
            }
        }
        rasterize(&frame)
    }
}

fn render_into(bitmap: &Bitmap, out: &mut [u32]) {
    let n = out.len().min(bitmap.rgba.len() / 4);
    for (i, p) in bitmap.rgba.chunks_exact(4).take(n).enumerate() {
        out[i] =
            ((p[3] as u32) << 24) | ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32);
    }
}
