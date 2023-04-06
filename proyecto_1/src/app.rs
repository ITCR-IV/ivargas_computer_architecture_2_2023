use eframe::egui::{self, Ui};

use crate::{constants::*, random::UniformRng};

#[derive(Debug)]
enum ExecutionMode {
    Automatic,
    Manual,
}

impl ExecutionMode {
    fn toggle(&mut self) {
        *self = match self {
            ExecutionMode::Manual => ExecutionMode::Automatic,
            ExecutionMode::Automatic => ExecutionMode::Manual,
        }
    }
}

pub struct AppState {
    rng: UniformRng,
    nums: Vec<u32>,
    mode: ExecutionMode,
    speed: f32,
}

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

impl AppState {
    fn controls_panel(&mut self, ui: &mut Ui) {
        ui.heading("Controls");
        if ui.button(format!("{:?}", self.mode)).clicked() {
            self.mode.toggle();
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
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("controls_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| self.controls_panel(ui))
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
