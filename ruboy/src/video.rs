use core::{
    array,
    fmt::Display,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::{
    error::Error,
    sync::{Arc, Mutex},
    time::Instant,
};

use eframe::egui::{self, Color32, ColorImage};
use ruboy_lib::{Frame, GBGraphicsDrawer, GbMonoColor, FRAME_X, FRAME_Y};

#[derive(Debug)]
pub struct VideoOutput {
    frame_dirty: Arc<AtomicBool>,
    framebuf: Arc<Mutex<FrameData>>,
    ui_ctx: egui::Context,
    num_frames: usize,
    last_frametime_check: Option<Instant>,
}

impl VideoOutput {
    pub fn new(
        dirty_flag: Arc<AtomicBool>,
        framebuf: Arc<Mutex<FrameData>>,
        ui_ctx: egui::Context,
    ) -> Self {
        Self {
            frame_dirty: dirty_flag,
            framebuf,
            ui_ctx,
            num_frames: 0,
            last_frametime_check: None,
        }
    }

    fn check_frametime(&mut self) {
        if self.last_frametime_check.is_none() {
            self.last_frametime_check = Some(Instant::now());
            return;
        }

        let last_check_time = self.last_frametime_check.unwrap();

        let now = Instant::now();
        let duration_since = now.duration_since(last_check_time);
        if duration_since > Duration::from_millis(100) {
            log::debug!(
                "FPS: {}",
                self.num_frames as f64 / duration_since.as_secs_f64()
            );

            self.num_frames = 0;
            self.last_frametime_check = Some(now);
        }
    }
}

#[derive(Debug)]
pub struct VideoOutputErr(anyhow::Error);

impl Display for VideoOutputErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for VideoOutputErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

pub const WHITE: Color32 = Color32::from_rgb(123, 130, 15);
pub const LIGHT_GRAY: Color32 = Color32::from_rgb(90, 121, 66);
pub const DARK_GRAY: Color32 = Color32::from_rgb(57, 89, 74);
pub const BLACK: Color32 = Color32::from_rgb(41, 65, 57);

impl GBGraphicsDrawer for VideoOutput {
    type Err = VideoOutputErr;

    fn output(&mut self, frame: &Frame) -> std::result::Result<(), Self::Err> {
        let converted_frame: Vec<Color32> = frame
            .get_raw()
            .iter()
            .map(|color| match color {
                GbMonoColor::White => WHITE,
                GbMonoColor::LightGray => LIGHT_GRAY,
                GbMonoColor::DarkGray => DARK_GRAY,
                GbMonoColor::Black => BLACK,
            })
            .collect();

        let mut locked_framebuf = self.framebuf.lock().unwrap();

        for (i, pix) in locked_framebuf.buf.iter_mut().enumerate() {
            *pix = converted_frame[i];
        }

        std::mem::drop(locked_framebuf);

        self.frame_dirty.store(true, Ordering::Relaxed);

        self.ui_ctx.request_repaint();

        self.num_frames += 1;
        self.check_frametime();

        Ok(())
    }
}

#[derive(Debug)]
pub struct FrameData {
    buf: [Color32; FRAME_X * FRAME_Y],
}

impl TryFrom<&[Color32]> for FrameData {
    type Error = ();

    fn try_from(value: &[Color32]) -> Result<Self, Self::Error> {
        if value.len() != FRAME_X * FRAME_Y {
            return Err(());
        }

        Ok(Self {
            buf: array::from_fn(|i| value[i]),
        })
    }
}

impl From<&FrameData> for ColorImage {
    fn from(value: &FrameData) -> Self {
        ColorImage {
            size: [FRAME_X, FRAME_Y],
            pixels: value.buf.to_vec(),
        }
    }
}

impl Default for FrameData {
    fn default() -> Self {
        let mut default_buf = [WHITE; FRAME_X * FRAME_Y];

        let mut cur_color = 0;
        for (y, row) in default_buf.chunks_mut(FRAME_X).enumerate() {
            for (x, pixel) in row.iter_mut().enumerate() {
                *pixel = Color32::from_rgb(
                    ((x as f64) / (FRAME_X as f64) * 255.0) as u8,
                    ((y as f64) / (FRAME_Y as f64) * 255.0) as u8,
                    0,
                )
            }

            cur_color = i32::min(3, cur_color + 1);
        }

        Self { buf: default_buf }
    }
}
