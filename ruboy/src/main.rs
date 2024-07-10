use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use anyhow::{Context, Result};
use clap::Parser;
use eframe::egui::Key;
use eframe::egui::{
    self, load::SizedTexture, CentralPanel, ColorImage, Image, TextureHandle, TextureOptions,
};
use eframe::NativeOptions;
use input::keyboard::KeyboardInput;
use input::Inputs;
use menu::{draw_menu, MenuData};
use ruboy_lib::{InlineAllocator, Ruboy};
use std::sync::Mutex;
use video::{FrameData, VideoOutput};

use crate::args::CLIArgs;

mod args;
mod input;
mod menu;
mod video;

struct RuboyApp {
    emu_thread: Option<JoinHandle<()>>,
    emu_died: bool,
    cli_args: CLIArgs,
    frame_dirty: Arc<AtomicBool>,
    framebuf: Arc<Mutex<FrameData>>,
    frametex: Option<TextureHandle>,
    input_handler: Arc<Inputs>,
    menu_data: MenuData,
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
            input_handler: Arc::new(Inputs::default()),
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

    fn init_emuthread(&mut self, ctx: &egui::Context) {
        debug_assert!(self.emu_thread.is_none());

        let thread_args = self.cli_args.clone();
        let cloned_framebuf = self.framebuf.clone();
        let cloned_dirty_flag = self.frame_dirty.clone();
        let cloned_context = ctx.clone();
        let cloned_inputs = self.input_handler.clone();

        let thread = thread::Builder::new()
            .name("emulator".to_owned())
            .spawn(move || {
                emulator_thread(
                    cloned_context,
                    thread_args,
                    cloned_inputs,
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

    fn update_keyboard_input(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            if !input.focused {
                self.input_handler.set_to_none();
                return;
            }

            let keys_down = &input.keys_down;

            self.input_handler
                .left
                .store(keys_down.contains(&Key::ArrowLeft), Ordering::Relaxed);
            self.input_handler
                .right
                .store(keys_down.contains(&Key::ArrowRight), Ordering::Relaxed);
            self.input_handler
                .up
                .store(keys_down.contains(&Key::ArrowUp), Ordering::Relaxed);
            self.input_handler
                .down
                .store(keys_down.contains(&Key::ArrowDown), Ordering::Relaxed);
            self.input_handler
                .a
                .store(keys_down.contains(&Key::A), Ordering::Relaxed);
            self.input_handler
                .b
                .store(keys_down.contains(&Key::B), Ordering::Relaxed);
            self.input_handler
                .start
                .store(keys_down.contains(&Key::Enter), Ordering::Relaxed);
            self.input_handler
                .select
                .store(keys_down.contains(&Key::Backspace), Ordering::Relaxed);
        });
    }
}

impl eframe::App for RuboyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_initialized(ctx);

        if !self.ensure_emulator_alive() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        self.update_keyboard_input(ctx);
        self.update_texture_from_framedata();

        // Actual UI code now
        CentralPanel::default().show(ctx, |ui| {
            draw_menu(&mut self.menu_data, ui);
            ui.separator();
            self.show_gameboy_frame(ui);
        });
    }
}

fn emulator_thread(
    ctx: egui::Context,
    args: CLIArgs,
    inputs: Arc<Inputs>,
    framebuf: Arc<Mutex<FrameData>>,
    dirty_flag: Arc<AtomicBool>,
) {
    let romfile = File::open(args.rom)
        .context("Could not open file at provided path")
        .unwrap();

    let reader = BufReader::new(romfile);

    let video = VideoOutput::new(dirty_flag, framebuf, ctx);

    let input = KeyboardInput::new(inputs);

    let ruboy = Ruboy::<InlineAllocator, _, _, _>::new(reader, video, input)
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
