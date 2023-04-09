use eframe::{
    egui::{self, Align2, Id, Rect, Rgba, Sense, TextStyle, Ui},
    epaint::{Color32, Pos2, Vec2},
};
use std::{
    mem::size_of,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use crate::{
    constants::*,
    models::{cache::CacheLine, system::SoC, Data},
    random::UniformRng,
};

#[derive(Debug, PartialEq)]
enum ExecutionMode {
    Automatic,
    Manual,
}

type GuiCache = Vec<CacheLine>;
type GuiMemory = Vec<Data>;

pub struct AppState {
    system: SoC,
    rng: UniformRng,
    nums: Vec<u32>,
    mode: ExecutionMode,
    speed: f32,
    previous_time: Instant,
    ctx: egui::Context,
    events_rx: Receiver<Event>,

    // These are different from the real system's memories, they're used for
    // the GUI to keep track of the current state of things
    caches: Vec<GuiCache>,
    main_memory: GuiMemory,
    offset_bits: usize,
    index_bits: usize,
    address_bits: usize,
}

pub enum Event {}

impl AppState {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        events_rx: Receiver<Event>,
        system: SoC,
    ) -> Self {
        let mut style: egui::Style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 5.0);
        style.animation_time = 1.0;
        cc.egui_ctx.set_style(style);

        // For cache drawing
        let mut index_bits = 0;
        let mut x = system.cache_sets() - 1;
        while x != 0 {
            x >>= 1;
            index_bits += 1;
        }

        let mut offset_bits = 0;
        let mut x = size_of::<Data>() - 1;
        while x != 0 {
            x >>= 1;
            offset_bits += 1;
        }

        let mut address_bits = 0;
        let mut x = system.main_memory_size() - 1;
        while x != 0 {
            x >>= 1;
            address_bits += 1;
        }
        address_bits <<= offset_bits;

        Self {
            nums: vec![0; system.num_processors()],
            caches: vec![
                vec![
                    CacheLine::new_cold();
                    system.cache_associativity() * system.cache_sets()
                ];
                system.num_processors()
            ],
            system,
            rng: UniformRng::from_seed(0),
            mode: ExecutionMode::Automatic,
            speed: 1.0,
            previous_time: Instant::now(),
            ctx: cc.egui_ctx.clone(),
            events_rx,
            main_memory: GuiMemory::new(),
            offset_bits,
            index_bits,
            address_bits,
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

    fn draw_cache(&self, i: usize, ui: &mut Ui) {
        let spacing = self.ctx.style().spacing.item_spacing;

        let address_width = (self.address_bits / 4) + 1 + 2;
        let data_width = size_of::<Data>() * 2 + 2;

        let font_id = TextStyle::Monospace.resolve(&self.ctx.style());
        let default_color = ui.visuals().text_color();

        let consultant_painter = ui.painter();

        let letter_size = consultant_painter
            .layout_no_wrap("M".to_string(), font_id.clone(), default_color)
            .rect;
        let data_text_width = consultant_painter
            .layout_no_wrap(
                format!("{:#0data_width$X}", 0),
                font_id.clone(),
                default_color,
            )
            .rect
            .width();
        let address_text_width = consultant_painter
            .layout_no_wrap(
                format!("{:#0address_width$X}", 0),
                font_id.clone(),
                default_color,
            )
            .rect
            .width();

        let grid_width = letter_size.width()
            + data_text_width
            + address_text_width
            + spacing.x * 6.0;
        let grid_height = (letter_size.height() + spacing.y * 2.0)
            * self.caches[i].len() as f32;
        let grid_size = Vec2 {
            x: grid_width,
            y: grid_height,
        };

        let (response, painter) = ui.allocate_painter(
            grid_size,
            Sense {
                click: false,
                drag: false,
                focusable: false,
            },
        );
        let grid_rect = response.rect;

        let stroke = ui.visuals().window_stroke;
        let rounding = ui.visuals().window_rounding;
        painter.rect_stroke(grid_rect, rounding, stroke);
        painter.vline(
            grid_rect.left() + letter_size.width() + spacing.x * 2.0,
            grid_rect.y_range(),
            stroke,
        );
        painter.vline(
            grid_rect.left()
                + letter_size.width()
                + address_text_width
                + spacing.x * 4.0,
            grid_rect.y_range(),
            stroke,
        );

        for (i, cache_line) in self.caches[i].iter().enumerate() {
            let index = i / self.system.cache_associativity();
            let address = ((cache_line.tag << self.index_bits) | index)
                << self.offset_bits;

            let y = grid_rect.top()
                + spacing.y * (i * 2 + 1) as f32
                + letter_size.height() * i as f32;
            let x = grid_rect.left() + spacing.x;

            painter.text(
                Pos2 { x, y },
                Align2::LEFT_TOP,
                cache_line.state.to_letter(),
                font_id.clone(),
                default_color,
            );

            let x = grid_rect.left() + spacing.x * 3.0 + letter_size.width();
            painter.text(
                Pos2 { x, y },
                Align2::LEFT_TOP,
                format!("{:#0address_width$X}", address),
                font_id.clone(),
                default_color,
            );

            let x = grid_rect.left()
                + letter_size.width()
                + address_text_width
                + spacing.x * 5.0;
            painter.text(
                Pos2 { x, y },
                Align2::LEFT_TOP,
                format!("{:#0data_width$X}", cache_line.data),
                font_id.clone(),
                default_color,
            );
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

            self.draw_cache(i, ui);
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
