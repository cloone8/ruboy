use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;
use eframe::egui::Key;
use eframe::egui::{
    self, load::SizedTexture, CentralPanel, ColorImage, Image, TextureHandle, TextureOptions,
};
use eframe::NativeOptions;
use input::SharedInputs;
use menu::{draw_menu, MenuData};
use ruboy_lib::{InlineAllocator, Ruboy};
use video::VideoOutput;

use crate::args::CLIArgs;

mod args;
mod input;
mod menu;
mod video;

struct RuboyApp {
    pub cli_args: CLIArgs,
    pub rom: Option<PathBuf>,
    pub prev_frame_time: Instant,
    pub ruboy: Option<Ruboy<InlineAllocator, BufReader<File>, VideoOutput, SharedInputs>>,
    pub frametex: Option<TextureHandle>,
    pub input_handler: SharedInputs,
    pub video_handler: VideoOutput,
    pub menu_data: MenuData,
}

impl RuboyApp {
    pub fn new(args: CLIArgs) -> Self {
        Self {
            cli_args: args,
            rom: None,
            prev_frame_time: Instant::now(),
            ruboy: None,
            frametex: None,
            input_handler: SharedInputs::new(),
            video_handler: VideoOutput::new(),
            menu_data: MenuData::default(),
        }
    }

    const fn get_gb_tex_options() -> TextureOptions {
        TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
            wrap_mode: egui::TextureWrapMode::ClampToEdge,
        }
    }

    fn init_ruboy(&mut self, romfile: impl AsRef<Path>) {
        debug_assert!(self.ruboy.is_none());

        let romfile = File::open(romfile).expect("Could not open file at provided path");

        let reader = BufReader::new(romfile);

        let ruboy = Ruboy::<InlineAllocator, _, _, _>::new(
            reader,
            self.video_handler.clone(),
            self.input_handler.clone(),
        )
        .expect("Could not initialize Ruboy");

        self.ruboy = Some(ruboy);
        self.prev_frame_time = Instant::now();
    }

    fn init_gbtexture(&mut self, ctx: &egui::Context) {
        debug_assert!(self.frametex.is_none());

        self.frametex = Some(ctx.load_texture(
            "Ruboy Display",
            ColorImage::from(self.video_handler.framebuf.borrow().deref()),
            Self::get_gb_tex_options(),
        ));
    }

    fn try_initialize(&mut self, ctx: &egui::Context) {
        if self.ruboy.is_none() {
            if let Some(rom) = self.rom.clone() {
                self.init_ruboy(rom);
            }
        }

        if self.frametex.is_none() {
            self.init_gbtexture(ctx);
        }
    }

    fn update_texture_from_framedata(&mut self) {
        if !(*self.video_handler.dirty.borrow()) {
            return;
        }

        self.frametex.as_mut().unwrap().set(
            ColorImage::from(self.video_handler.framebuf.borrow().deref()),
            Self::get_gb_tex_options(),
        );

        *self.video_handler.dirty.borrow_mut() = false;
    }

    fn show_gameboy_frame(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            if self.frametex.is_some() {
                let sized_tex = SizedTexture::from_handle(&self.frametex.clone().unwrap());

                let image = Image::new(sized_tex)
                    .maintain_aspect_ratio(true)
                    .shrink_to_fit();
                ui.add(image);
            }
        });
    }

    fn update_keyboard_input(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            if !input.focused {
                self.input_handler.inputs.borrow_mut().set_to_none();
                return;
            }

            let keys_down = &input.keys_down;
            let mut inputs = self.input_handler.inputs.borrow_mut();

            inputs.left = keys_down.contains(&Key::ArrowLeft);
            inputs.right = keys_down.contains(&Key::ArrowRight);
            inputs.up = keys_down.contains(&Key::ArrowUp);
            inputs.down = keys_down.contains(&Key::ArrowDown);
            inputs.a = keys_down.contains(&Key::A);
            inputs.b = keys_down.contains(&Key::B);
            inputs.start = keys_down.contains(&Key::Enter);
            inputs.select = keys_down.contains(&Key::Backspace);
        });
    }

    fn step_emulator(&mut self, ctx: &egui::Context) {
        self.update_keyboard_input(ctx);

        let cur_time = Instant::now();

        let dt = cur_time.duration_since(self.prev_frame_time).as_secs_f64();
        let _cycles_ran = self.ruboy.as_mut().unwrap().step(dt).unwrap();

        self.prev_frame_time = cur_time;

        self.update_texture_from_framedata();
    }
}

impl eframe::App for RuboyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.try_initialize(ctx);

        if self.ruboy.is_some() {
            self.step_emulator(ctx);
        }

        // Actual UI code now
        CentralPanel::default().show(ctx, |ui| {
            draw_menu(self, ui);
            ui.separator();

            if self.ruboy.is_none() {
                ui.label("No ROM selected. Select a ROM with 'ROM -> Open'");
            }

            self.show_gameboy_frame(ui);
        });

        ctx.request_repaint();
    }
}

fn main() {
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
}
