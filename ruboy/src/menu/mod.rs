use audio::AudioMenuData;
use debugger::DebuggerMenuData;
use eframe::egui::{self, Ui};
use rom::RomMenuData;
use save::SaveMenuData;
use window::WindowMenuData;

mod audio;
mod debugger;
mod rom;
mod save;
mod window;

#[derive(Debug, Default)]
pub struct MenuData {
    rom: RomMenuData,
    save: SaveMenuData,
    window: WindowMenuData,
    debugger: DebuggerMenuData,
    audio: AudioMenuData,
}

pub fn draw_menu(data: &mut MenuData, ui: &mut Ui) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("ROM", |ui| {
            rom::draw_menu(&mut data.rom, ui);
        });

        ui.menu_button("Save", |ui| {
            save::draw_menu(&mut data.save, ui);
        });

        ui.menu_button("Audio", |ui| {
            audio::draw_menu(&mut data.audio, ui);
        });

        ui.menu_button("Window", |ui| {
            window::draw_menu(&mut data.window, ui);
        });

        ui.menu_button("Debugger", |ui| {
            debugger::draw_menu(&mut data.debugger, ui);
        });
    });
}
