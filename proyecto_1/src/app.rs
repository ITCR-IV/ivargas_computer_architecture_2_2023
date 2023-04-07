use eframe::egui::{self, Id, Rgba, Ui};
use std::time::{Duration, Instant};

use crate::{constants::*, random::UniformRng};

#[derive(Debug, PartialEq)]
enum ExecutionMode {
    Automatic,
    Manual,
}

pub struct AppState {
    rng: UniformRng,
    nums: Vec<u32>,
    mode: ExecutionMode,
    speed: f32,
    previous_time: Instant,
    ctx: egui::Context,
}

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style: egui::Style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 5.0);
        style.animation_time = 1.0;
        cc.egui_ctx.set_style(style);

        Self {
            rng: UniformRng::from_seed(0),
            nums: vec![0; NUM_PROCESSORS],
            mode: ExecutionMode::Automatic,
            speed: 1.0,
            previous_time: Instant::now(),
            ctx: cc.egui_ctx.clone(),
        }
    }

    fn update_random_processor(&mut self, num: u32) {
        let i = self.rng.gen_range(0u32..NUM_PROCESSORS as u32) as usize;
        self.nums[i] = num;
        self.ctx.animate_bool(self.get_processor_id(i), true);
    }

    fn get_processor_id(&self, i: usize) -> Id {
        Id::new(format!("cpu_id_{}", i))
    }
}

impl AppState {
    fn controls_panel(&mut self, ui: &mut Ui) {
        ui.heading("Execution Mode");
        ui.radio_value(
            &mut self.mode,
            ExecutionMode::Automatic,
            format!("{:?}", ExecutionMode::Automatic),
        );
        ui.radio_value(
            &mut self.mode,
            ExecutionMode::Manual,
            format!("{:?}", ExecutionMode::Manual),
        );

        ui.separator();

        ui.heading("Controls");
        match self.mode {
            ExecutionMode::Manual => {
                if ui.button("Step").clicked() {
                    let num = self.rng.gen_range(0..=10);
                    self.update_random_processor(num);
                }
            }
            ExecutionMode::Automatic => {
                ui.add(
                    egui::Slider::new(&mut self.speed, 0.1..=10.0)
                        .text("seconds"),
                );
                let time_passed = Instant::now() - self.previous_time;
                if time_passed
                    > Duration::from_millis((self.speed * 1000.0) as u64)
                {
                    let num = self.rng.gen_range(0..=10);
                    self.update_random_processor(num);
                    self.previous_time = Instant::now();
                }
                self.ctx.request_repaint();
            }
        }
    }

    fn draw_processor(&mut self, i: usize, ui: &mut Ui) {
        let width = ui.available_width() / NUM_PROCESSORS as f32;
        let height = ui.available_height();
        let spacing = self.ctx.style().spacing.item_spacing;

        let max_rect = egui::Rect::from_min_size(
            // On purpose the spacing isn't divided by 2 so that first block
            // will get double spacing on the left and last block won't get
            // spacing on the right, since it's already accounted for
            (width * i as f32 + spacing.x, spacing.y).into(),
            (width - spacing.x, height).into(),
        );
        let layout = egui::Layout::top_down(egui::Align::Center);

        ui.child_ui(max_rect, layout).group(|ui| {
            ui.heading(format!("CPU{}", i + 1));

            let red_portion =
                self.ctx.animate_bool(self.get_processor_id(i), false);
            let default_color: Rgba = ui.visuals().text_color().into();
            let mixed_color =
                default_color * (1.0 - red_portion) + Rgba::RED * red_portion;

            ui.colored_label(mixed_color, format!("{}", self.nums[i]));
        });
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("controls_panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| self.controls_panel(ui))
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                for i in 0..NUM_PROCESSORS {
                    self.draw_processor(i, ui);
                }
            });
        });
    }
}
