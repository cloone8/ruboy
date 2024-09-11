use core::cell::RefCell;
use core::{array, fmt::Display};
use std::error::Error;
use std::rc::Rc;

use eframe::egui::{Color32, ColorImage};
use ruboy_lib::{Frame, GBGraphicsDrawer, GbMonoColor, FRAME_X, FRAME_Y};

#[derive(Debug, Clone)]
pub struct VideoOutput {
    pub framebuf: Rc<RefCell<FrameData>>,
    pub dirty: Rc<RefCell<bool>>,
}

impl VideoOutput {
    pub fn new() -> Self {
        Self {
            framebuf: Rc::new(RefCell::new(FrameData::default())),
            dirty: Rc::new(RefCell::new(true)),
        }
    }
}

#[derive(Debug)]
pub enum VideoOutputErr {}

impl Display for VideoOutputErr {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!();
    }
}
impl Error for VideoOutputErr {}

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

        for (i, pix) in self.framebuf.borrow_mut().buf.iter_mut().enumerate() {
            *pix = converted_frame[i];
        }

        *self.dirty.borrow_mut() = true;

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
