use std::fmt::Display;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::{error::Error, fs::File};

use anyhow::{Context, Result};
use clap::Parser;
use eframe::egui::load::SizedTexture;
use eframe::egui::{
    self, CentralPanel, Color32, ColorImage, Image, TextureHandle, TextureOptions, TextureWrapMode,
};
use eframe::NativeOptions;
use ruboy_lib::{GBGraphicsDrawer, Gameboy, StackAllocator, FRAME_X, FRAME_Y};
use std::sync::Mutex;

use crate::args::CLIArgs;

mod args;

struct VideoOutput {
    framebuf: Arc<Mutex<FrameData>>,
}

impl VideoOutput {
    pub fn new(framebuf: Arc<Mutex<FrameData>>) -> Self {
        Self { framebuf }
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

const COLOR_0: Color32 = Color32::from_gray(255);
const COLOR_1: Color32 = Color32::from_gray(170);
const COLOR_2: Color32 = Color32::from_gray(85);
const COLOR_3: Color32 = Color32::from_gray(0);

impl GBGraphicsDrawer for VideoOutput {
    type Err = VideoOutputErr;

    fn output(&mut self, frame: &ruboy_lib::Frame) -> std::prelude::v1::Result<(), Self::Err> {
        todo!()
    }
}

#[derive(Debug)]
struct FrameData {
    buf: [Color32; FRAME_X * FRAME_Y],
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
        let mut default_buf = [COLOR_0; FRAME_X * FRAME_Y];

        let mut cur_color = 0;
        for row in default_buf.chunks_mut(FRAME_X) {
            for col in row {
                *col = match cur_color {
                    0 => COLOR_0,
                    1 => COLOR_1,
                    2 => COLOR_2,
                    3 => COLOR_3,
                    _ => panic!("Invalid color"),
                }
            }

            cur_color = (cur_color + 1) % 4;
        }

        Self { buf: default_buf }
    }
}

struct RuboyApp {
    emu_thread: Option<JoinHandle<()>>,
    emu_died: bool,
    cli_args: CLIArgs,
    framebuf: Arc<Mutex<FrameData>>,
    frametex: Option<TextureHandle>,
}

impl RuboyApp {
    pub fn new(args: CLIArgs, framebuf: Arc<Mutex<FrameData>>) -> Self {
        Self {
            emu_thread: None,
            emu_died: false,
            cli_args: args,
            framebuf,
            frametex: None,
        }
    }

    fn init_emuthread(&mut self) {
        let thread_args = self.cli_args.clone();
        let cloned_framebuf = self.framebuf.clone();

        self.emu_thread = Some(thread::spawn(move || {
            emulator_thread(thread_args, cloned_framebuf)
        }));
    }

    fn init_gbtexture(&mut self, ctx: &egui::Context) {
        let framedata = self.framebuf.lock().unwrap();

        self.frametex = Some(ctx.load_texture(
            "Gameboy Display",
            ColorImage::from(framedata.deref()),
            TextureOptions {
                magnification: egui::TextureFilter::Nearest,
                minification: egui::TextureFilter::Nearest,
                wrap_mode: TextureWrapMode::default(),
            },
        ));
    }

    fn ensure_initialized(&mut self, ctx: &egui::Context) {
        if self.emu_thread.is_none() && !self.emu_died {
            self.init_emuthread();
        }

        if self.frametex.is_none() {
            self.init_gbtexture(ctx);
        }
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ensure_initialized(ctx);

        if !self.ensure_emulator_alive() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        self.show_gameboy_frame(ctx);
    }
}

fn emulator_thread(args: CLIArgs, framebuf: Arc<Mutex<FrameData>>) {
    let romfile = File::open(args.rom)
        .context("Could not open file at provided path")
        .unwrap();

    let reader = BufReader::new(romfile);

    let video = VideoOutput::new(framebuf);

    let gameboy = Gameboy::<StackAllocator, _, _>::new(reader, video)
        .context("Could not initialize Gameboy")
        .unwrap();

    gameboy
        .start()
        .context("Error during Gameboy execution")
        .unwrap();
}

fn main() -> Result<()> {
    let args = CLIArgs::parse();

    let logconfig = simplelog::ConfigBuilder::new()
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

    let framebuf = Arc::new(Mutex::new(FrameData::default()));

    let options = NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "Ruboy",
        options,
        Box::new(|_| Box::new(RuboyApp::new(args, framebuf))),
    )
    .expect("Could not initialize window");

    Ok(())
}
