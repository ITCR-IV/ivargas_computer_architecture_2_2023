// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod random;

use eframe::egui;
use random::UniformRng;

const NUM_PROCESSORS: usize = 4;

#[derive(Debug)]
enum ExecutionMode {
    Automatic,
    Manual,
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Cache Sim",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}

struct AppState {
    rng: UniformRng,
    nums: Vec<u32>,
    mode: ExecutionMode,
    speed: f32,
}

impl AppState {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style: egui::Style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 5.0);
        cc.egui_ctx.set_style(style);

        Self {
            nums: vec![0; NUM_PROCESSORS],
            rng: UniformRng::from_seed(0),
            mode: ExecutionMode::Manual,
            speed: 1.0,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("controls_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.heading("Controls");
                    if ui.button(format!("{:?}", self.mode)).clicked() {
                        self.mode = match self.mode {
                            ExecutionMode::Manual => ExecutionMode::Automatic,
                            ExecutionMode::Automatic => ExecutionMode::Manual,
                        }
                    }
                    match self.mode {
                        ExecutionMode::Manual => {
                            if ui.button("Step").clicked() {
                                for num in &mut self.nums {
                                    *num = self.rng.gen_range(0..=3);
                                }
                            }
                        }
                        ExecutionMode::Automatic => {
                            ui.add(
                                egui::Slider::new(&mut self.speed, 0.1..=10.0)
                                    .text("speed"),
                            );
                        }
                    }
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Numbers");
            ui.horizontal(|ui| {
                for num in &self.nums {
                    ui.label(format!("{num}"));
                    ui.separator();
                }
            });
        });
    }
}
