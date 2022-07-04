use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use eframe::egui::{CentralPanel, Context, RichText, Ui};

use eframe::epaint::{Color32, Rect, Vec2};
use eframe::{egui, Frame, NativeOptions};
use rfd::FileHandle;

use crate::cpu::{Chip8, Chip8IO, KEYPAD_TO_QWERTY};

const WINDOW_NAME: &str = "CHIP8";

pub const SCALE: usize = 16;
pub const REFRESH_RATE: u64 = 60;

pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;

pub const PIXEL_HEIGHT: f32 = WINDOW_HEIGHT as f32 / SCREEN_HEIGHT as f32;
pub const PIXEL_WIDTH: f32 = WINDOW_WIDTH as f32 / SCREEN_WIDTH as f32;

const WINDOW_HEIGHT: f32 = SCREEN_HEIGHT as f32 * SCALE as f32;
const WINDOW_WIDTH: f32 = SCREEN_WIDTH as f32 * SCALE as f32;

/// The font set
pub const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug, Clone)]
pub struct App {
    chip8: Arc<Mutex<Chip8>>,
    io: Arc<Mutex<Chip8IO>>,
    /// Whether the execution should be paused
    pause_execution: bool,
    /// Step between frames
    step: bool,

    pub fg_color: [f32; 3],
    pub bg_color: [f32; 3],
    bold_text_color: Color32,
    reg_read_color: Color32,
    reg_write_color: Color32,

    target_ips: Arc<AtomicU64>,
}

impl App {
    pub fn new(
        cpu: Arc<Mutex<Chip8>>,
        io: Arc<Mutex<Chip8IO>>,
        target_ips: Arc<AtomicU64>,
    ) -> Self {
        Self {
            chip8: cpu,
            io,
            target_ips,
            pause_execution: false,
            step: false,
            bold_text_color: Color32::from_rgb(110, 255, 110),
            reg_read_color: Color32::from_rgb(110, 110, 255),
            reg_write_color: Color32::from_rgb(255, 110, 110),
            fg_color: [1.; 3],
            bg_color: [0.; 3],
        }
    }

    pub fn run(self) {
        let native_options = NativeOptions {
            initial_window_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..Default::default()
        };

        eframe::run_native(WINDOW_NAME, native_options, Box::new(|cc| Box::new(self)));
    }

    pub fn show_main_menubar(&mut self, egui_ctx: &Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load ROM").clicked() {
                        let self_cpy = self.clone();

                        tokio::spawn(async move {
                            let file = Self::get_path_buf().await;
                            if let Some(file_handle) = file {
                                let rom = file_handle.read().await;
                                if let Ok(mut chip8) = self_cpy.chip8.lock() {
                                    chip8.reset();
                                    chip8.load_rom(&rom);
                                }
                            }
                        });

                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory().reset_areas();
                        ui.close_menu();
                    }
                    if ui
                        .button("Reset egui memory")
                        .on_hover_text("Forget scroll, positions, sizes etc")
                        .clicked()
                    {
                        *ui.ctx().memory() = Default::default();
                        ui.close_menu();
                    }
                });
            });
        });
    }

    pub fn show_controls(&mut self, egui_ctx: &Context) {
        //pub fn show_controls(&mut self, egui_ctx: &Context, chip8: &mut Chip8, speed: &mut i32, pause_execution: &mut bool, step: &mut bool, fg_color: &mut [f32;3], bg_color: &mut [f32;3]) {
        egui::Window::new("Control").show(egui_ctx, |ui| {
            ui.set_max_width(190.);

            ui.horizontal(|ui| {
                if ui.button("Toggle execution").clicked() {
                    self.pause_execution = !self.pause_execution;
                }
                if ui.button("Step").clicked() {
                    self.step = true;
                }
            });

            ui.separator();
            ui.label(RichText::new("Display Color:").color(self.bold_text_color));
            ui.horizontal(|ui| {
                ui.label("FG:");
                if ui.color_edit_button_rgb(&mut self.fg_color).changed() {}
            });
            ui.horizontal(|ui| {
                ui.label("BG:");
                if ui.color_edit_button_rgb(&mut self.bg_color).changed() {}
            });
        });
    }

    fn show_chip8_display(&self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_at_least(
            Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            egui::Sense {
                click: false,
                drag: false,
                focusable: false,
            },
        );

        ui.set_min_height(SCREEN_HEIGHT as f32);

        let mut pos = rect.min;
        let value = self.io.lock().unwrap().display;
        for row in value {
            pos.x = 0.;
            for pixel in row {
                ui.painter().rect(
                    Rect::from_min_size(pos, Vec2::new(PIXEL_WIDTH, PIXEL_HEIGHT)),
                    0.,
                    if pixel {
                        Color32::from_rgb(
                            (self.fg_color[0] * 255.) as u8,
                            (self.fg_color[1] * 255.) as u8,
                            (self.fg_color[2] * 255.) as u8,
                        )
                    } else {
                        Color32::from_rgb(
                            (self.bg_color[0] * 255.) as u8,
                            (self.bg_color[1] * 255.) as u8,
                            (self.bg_color[2] * 255.) as u8,
                        )
                    },
                    (
                        0.,
                        Color32::from_rgb(
                            (self.bg_color[0] * 255.) as u8,
                            (self.bg_color[1] * 255.) as u8,
                            (self.bg_color[2] * 255.) as u8,
                        ),
                    ),
                );
                pos.x += PIXEL_WIDTH;
            }
            pos.y += PIXEL_HEIGHT as f32;
        }

        response
    }

    pub fn show_general_state(&mut self, egui_ctx: &Context) {
        let self_cpy = self.clone();
        let m_chip8 = match self_cpy.chip8.lock() {
            Ok(value) => value,
            Err(_) => return,
        };

        egui::Window::new("General State").show(egui_ctx, |ui| {
            ui.set_max_width(190.);

            self.label_bold("CPU Info:", ui);

            ui.horizontal_wrapped(|ui| {
                self.label_bold("PC:", ui);
                ui.label(format!("{:02X} ", m_chip8.pc));
                self.label_bold("OP:", ui);
                let opcode: u16 = m_chip8.current_instruction().unwrap().into();
                ui.label(format!("{:02X} ", opcode));
                self.label_bold("IR:", ui);
                ui.label(format!("{:02X} ", m_chip8.reg.i));
            });

            ui.separator();

            if !m_chip8.stack.is_empty() {
                ui.label(format!(
                    "Stack: {:#04x}",
                    m_chip8.stack[m_chip8.stack.len() - 1]
                ));
            } else {
                ui.label("Stack: empty");
            }
        });
    }

    pub fn label_bold(&mut self, text: &str, ui: &mut Ui) {
        ui.label(RichText::new(text).color(self.bold_text_color));
    }

    async fn get_path_buf() -> Option<FileHandle> {
        rfd::AsyncFileDialog::new()
            .add_filter("CHIP-8 ROM", &["ch8"])
            .set_directory("/")
            .pick_file()
            .await
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        {
            let chip8_keys = &mut self.io.lock().unwrap().keystate;
            let pressed_keys = &ctx.input().keys_down;
            for key in 0..chip8_keys.len() {
                chip8_keys[key] =
                    pressed_keys.contains(&key_for_char(KEYPAD_TO_QWERTY[&(key as u8)]).unwrap())
            }
        }

        self.show_general_state(ctx);
        self.show_controls(ctx);

        self.show_main_menubar(ctx, frame);

        {
            CentralPanel::default().show(ctx, |ui| {
                self.show_chip8_display(ui);
            });
        }
        // If not paused or paused but step requested
        if !self.pause_execution || self.step {}

        {}

        ctx.request_repaint()
    }
}

fn key_for_char(value: char) -> Option<egui::Key> {
    match value {
        '1' => Some(egui::Key::Num1),
        '2' => Some(egui::Key::Num2),
        '3' => Some(egui::Key::Num3),
        '4' => Some(egui::Key::Num4),
        '5' => Some(egui::Key::Num5),
        '6' => Some(egui::Key::Num6),
        '7' => Some(egui::Key::Num7),
        '8' => Some(egui::Key::Num8),
        '9' => Some(egui::Key::Num9),
        '0' => Some(egui::Key::Num0),
        'q' | 'Q' => Some(egui::Key::Q),
        'w' | 'W' => Some(egui::Key::W),
        'e' | 'E' => Some(egui::Key::E),
        'r' | 'R' => Some(egui::Key::R),
        't' | 'T' => Some(egui::Key::T),
        'y' | 'Y' => Some(egui::Key::Y),
        'u' | 'U' => Some(egui::Key::U),
        'i' | 'I' => Some(egui::Key::I),
        'o' | 'O' => Some(egui::Key::O),
        'p' | 'P' => Some(egui::Key::P),
        'a' | 'A' => Some(egui::Key::A),
        's' | 'S' => Some(egui::Key::S),
        'd' | 'D' => Some(egui::Key::D),
        'f' | 'F' => Some(egui::Key::F),
        'g' | 'G' => Some(egui::Key::G),
        'h' | 'H' => Some(egui::Key::H),
        'j' | 'J' => Some(egui::Key::J),
        'k' | 'K' => Some(egui::Key::K),
        'l' | 'L' => Some(egui::Key::L),
        'z' | 'Z' => Some(egui::Key::Z),
        'x' | 'X' => Some(egui::Key::X),
        'c' | 'C' => Some(egui::Key::C),
        'v' | 'V' => Some(egui::Key::V),
        'b' | 'B' => Some(egui::Key::B),
        'n' | 'N' => Some(egui::Key::N),
        'm' | 'M' => Some(egui::Key::M),
        _ => None,
    }
}
