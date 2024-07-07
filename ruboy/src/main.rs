use core::time::Duration;
use std::array;
use std::fmt::Display;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;
use std::{error::Error, fs::File};

use anyhow::{Context, Result};
use clap::Parser;
use eframe::egui::{
    self, load::SizedTexture, CentralPanel, Color32, ColorImage, Image, TextureHandle,
    TextureOptions,
};
use eframe::NativeOptions;
use ruboy_lib::{Frame, GBGraphicsDrawer, GbMonoColor, InlineAllocator, Ruboy, FRAME_X, FRAME_Y};
use std::sync::Mutex;

use crate::args::CLIArgs;

mod args;

#[derive(Debug)]
struct VideoOutput {
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
struct VideoOutputErr(anyhow::Error);

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

const WHITE: Color32 = Color32::from_rgb(123, 130, 15);
const LIGHT_GRAY: Color32 = Color32::from_rgb(90, 121, 66);
const DARK_GRAY: Color32 = Color32::from_rgb(57, 89, 74);
const BLACK: Color32 = Color32::from_rgb(41, 65, 57);

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
struct FrameData {
    buf: [Color32; FRAME_X * FRAME_Y],
}

impl TryFrom<&[Color32]> for FrameData {
    type Error = ();

    fn try_from(value: &[Color32]) -> std::prelude::v1::Result<Self, Self::Error> {
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

struct RuboyApp {
    emu_thread: Option<JoinHandle<()>>,
    emu_died: bool,
    cli_args: CLIArgs,
    frame_dirty: Arc<AtomicBool>,
    framebuf: Arc<Mutex<FrameData>>,
    frametex: Option<TextureHandle>,
}

impl RuboyApp {
    pub fn new(args: CLIArgs) -> Self {
        Self {
            emu_thread: None,
            emu_died: false,
            cli_args: args,
            framebuf: Arc::new(Mutex::new(FrameData::default())),
            frame_dirty: Arc::new(AtomicBool::new(false)),
            frametex: None,
        }
    }

    const fn get_gb_tex_options() -> TextureOptions {
        TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
            wrap_mode: egui::TextureWrapMode::ClampToEdge,
        }
    }

    fn init_emuthread(&mut self, ctx: &egui::Context) {
        debug_assert!(self.emu_thread.is_none());

        let thread_args = self.cli_args.clone();
        let cloned_framebuf = self.framebuf.clone();
        let cloned_dirty_flag = self.frame_dirty.clone();
        let cloned_context = ctx.clone();

        let thread = thread::Builder::new()
            .name("emulator".to_owned())
            .spawn(move || {
                emulator_thread(
                    cloned_context,
                    thread_args,
                    cloned_framebuf,
                    cloned_dirty_flag,
                )
            })
            .expect("Could not spawn emulator thread");

        self.emu_thread = Some(thread);
    }

    fn init_gbtexture(&mut self, ctx: &egui::Context) {
        debug_assert!(self.frametex.is_none());

        let framedata = self.framebuf.lock().unwrap();

        self.frametex = Some(ctx.load_texture(
            "Ruboy Display",
            ColorImage::from(framedata.deref()),
            Self::get_gb_tex_options(),
        ));
    }

    fn ensure_initialized(&mut self, ctx: &egui::Context) {
        if self.emu_thread.is_none() && !self.emu_died {
            self.init_emuthread(ctx);
        }

        if self.frametex.is_none() {
            self.init_gbtexture(ctx);
        }
    }

    fn update_texture_from_framedata(&mut self) {
        if !self.frame_dirty.load(Ordering::Relaxed) {
            return;
        }

        let locked_framebuf = self.framebuf.lock().unwrap();

        self.frametex.as_mut().unwrap().set(
            ColorImage::from(locked_framebuf.deref()),
            Self::get_gb_tex_options(),
        );

        self.frame_dirty.store(true, Ordering::Relaxed);
    }

    fn show_gameboy_frame(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                if self.frametex.is_some() {
                    let sized_tex = SizedTexture::from_handle(&self.frametex.clone().unwrap());

                    let image = Image::new(sized_tex)
                        .maintain_aspect_ratio(true)
                        .shrink_to_fit();
                    ui.add(image);
                }
            });
        });
    }

    fn ensure_emulator_alive(&mut self) -> bool {
        match &mut self.emu_thread {
            Some(handle) => {
                if handle.is_finished() {
                    self.emu_died = true;
                    let handle = self.emu_thread.take().unwrap();

                    let _ = handle.join();

                    false
                } else {
                    true
                }
            }
            None => false,
        }
    }
}

impl eframe::App for RuboyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_initialized(ctx);

        if !self.ensure_emulator_alive() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        self.update_texture_from_framedata();
        self.show_gameboy_frame(ctx);
    }
}

fn emulator_thread(
    ctx: egui::Context,
    args: CLIArgs,
    framebuf: Arc<Mutex<FrameData>>,
    dirty_flag: Arc<AtomicBool>,
) {
    let romfile = File::open(args.rom)
        .context("Could not open file at provided path")
        .unwrap();

    let reader = BufReader::new(romfile);

    let video = VideoOutput::new(dirty_flag, framebuf, ctx);

    let ruboy = Ruboy::<InlineAllocator, _, _>::new(reader, video)
        .context("Could not initialize Ruboy")
        .unwrap();

    ruboy
        .start()
        .context("Error during Ruboy execution")
        .unwrap();
}

fn main() -> Result<()> {
    let args = CLIArgs::parse();

    let logconfig = simplelog::ConfigBuilder::new()
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set logger time offset to local")
        .build();

    simplelog::TermLogger::init(
        args.verbosity.clone().into(),
        logconfig,
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .expect("Could not initialize logger");

    log::info!("Starting Ruboy Emulator Frontend");

    let options = NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "Ruboy",
        options,
        Box::new(|_| Ok(Box::new(RuboyApp::new(args)))),
    )
    .expect("Could not initialize window");

    Ok(())
}
