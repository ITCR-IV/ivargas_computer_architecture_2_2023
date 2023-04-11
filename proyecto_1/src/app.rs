use eframe::{
    egui::{self, Align, Align2, Id, Layout, Rgba, Sense, TextStyle, Ui},
    epaint::{Color32, Pos2, Vec2},
};
use std::{
    collections::VecDeque,
    mem::{size_of, variant_count},
    sync::mpsc::{Receiver, SyncSender, TryRecvError},
    time::{Duration, Instant},
};

use crate::{
    models::{
        cache::CacheLine, instructions::Instruction, system::SocProperties,
        Data, MemOp,
    },
    random::UniformRng,
};

const PROCESSORS_PER_ROW: usize = 2;
const PROCESSORS_HEIGHT_PERCENT: f32 = 0.66;

const INSTRUCTIONS_HIST: usize = 8;

#[derive(Debug, PartialEq)]
enum ExecutionMode {
    Automatic,
    Manual,
}

type GuiCache = Vec<CacheLine>;
type GuiMemory = Vec<Data>;

pub struct AppState {
    system_props: SocProperties,
    instruction_txs: Vec<SyncSender<Instruction>>,
    rng: UniformRng,
    mode: ExecutionMode,
    speed: f32,
    previous_time: Instant,
    ctx: egui::Context,
    events_rx: Receiver<Event>,

    // Last address that was missed per processor
    read_miss_addresses: Vec<usize>,
    write_miss_addresses: Vec<usize>,

    // Last instruction for each processor
    last_instructions: Vec<Instruction>,
    instructions_hist: VecDeque<(usize, Instruction)>,

    // These are different from the real system's memories, they're used for
    // the GUI to keep track of the current state of things
    caches: Vec<GuiCache>,
    main_memory: GuiMemory,

    offset_bits: usize,
    index_bits: usize,
    address_bits: usize,
}

pub enum Event {
    CacheWrite {
        cache_i: usize,
        block_i: usize,
        line: CacheLine,
    },
    MemWrite {
        block_i: usize,
        data: Data,
    },
    // Assumes a miss
    Alert {
        processor_i: usize,
        address: usize,
        op: MemOp,
    },
}

impl AppState {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        events_rx: Receiver<Event>,
        instruction_txs: Vec<SyncSender<Instruction>>,
        system_props: SocProperties,
    ) -> Self {
        let mut style: egui::Style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 5.0);
        style.animation_time = 1.0;
        cc.egui_ctx.set_style(style);

        // For cache drawing
        let mut index_bits = 0;
        let mut x = system_props.cache_sets - 1;
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
        let mut x = system_props.main_memory_blocks - 1;
        while x != 0 {
            x >>= 1;
            address_bits += 1;
        }
        address_bits += offset_bits;

        Self {
            caches: vec![
                vec![
                    CacheLine::new_cold();
                    system_props.cache_associativity
                        * system_props.cache_sets
                ];
                system_props.num_processors
            ],
            main_memory: vec![0; system_props.main_memory_blocks],
            read_miss_addresses: vec![0; system_props.num_processors],
            write_miss_addresses: vec![0; system_props.num_processors],
            last_instructions: vec![
                Instruction::Calc;
                system_props.num_processors
            ],
            instructions_hist: vec![(0, Instruction::Calc); INSTRUCTIONS_HIST]
                .into(),
            system_props,
            rng: UniformRng::from_seed(0),
            mode: ExecutionMode::Automatic,
            speed: 1.0,
            previous_time: Instant::now(),
            ctx: cc.egui_ctx.clone(),
            events_rx,
            offset_bits,
            index_bits,
            address_bits,
            instruction_txs,
        }
    }

    fn gen_random_address(&mut self) -> usize {
        (self
            .rng
            .gen_range(0..self.system_props.main_memory_blocks as u32)
            << self.offset_bits) as usize
    }

    fn gen_random_instruction(&mut self) -> Instruction {
        match (self.rng.gen() >> 16) % variant_count::<Instruction>() as u32 {
            0 => Instruction::Calc,
            1 => Instruction::Read {
                address: self.gen_random_address(),
            },
            2 => Instruction::Write {
                address: self.gen_random_address(),
                data: self.rng.gen() as u16,
            },
            _ => panic!("Unaccounted for instruction"),
        }
    }

    fn save_instruction(&mut self, cpu_i: usize, instruction: Instruction) {
        self.last_instructions[cpu_i] = instruction.clone();
        self.instructions_hist.push_back((cpu_i, instruction));
        while self.instructions_hist.len() > INSTRUCTIONS_HIST {
            self.instructions_hist.pop_front();
        }
    }

    fn give_instruction_to_all(&mut self) {
        println!("---------------------------");
        for i in 0..self.system_props.num_processors {
            let instruction = self.gen_random_instruction();
            self.save_instruction(i, instruction.clone());
            println!("Sending instruction {instruction:?} to processor {i}");
            let processor_tx = &self.instruction_txs[i];
            match processor_tx.send(instruction) {
                Ok(_) => (),
                Err(_) => {
                    panic!("One of the system threads died unexpectedly")
                }
            }
        }
    }

    fn get_cache_line_id(&self, cache_i: usize, line_i: usize) -> Id {
        Id::new(format!("cache_line_id_{cache_i}_{line_i}"))
    }

    fn get_mem_line_id(&self, line_i: usize) -> Id {
        Id::new(format!("main_memory_line_id__{line_i}"))
    }

    fn get_alert_id(&self, processor_id: usize, op: MemOp) -> Id {
        Id::new(format!("miss_alert_{processor_id}_{op:?}"))
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
                    self.give_instruction_to_all();
                }
            }
            ExecutionMode::Automatic => {
                ui.add(
                    egui::Slider::new(&mut self.speed, 1.0..=10.0)
                        .text("seconds"),
                );
                let time_passed = Instant::now() - self.previous_time;
                if time_passed
                    > Duration::from_millis((self.speed * 1000.0) as u64)
                {
                    self.give_instruction_to_all();
                    self.previous_time = Instant::now();
                }
                self.ctx.request_repaint();
            }
        }

        ui.separator();
        ui.vertical(|ui| {
            ui.heading("Instructions History");
            for (cpu, instruction) in &self.instructions_hist {
                ui.label(format!("CPU{}: {}", cpu + 1, instruction));
            }
        });

        let spacing = self.ctx.style().spacing.item_spacing;
        ui.add_space(spacing.y * 2.0);
        ui.label("(Most recent at the top)");
    }

    fn draw_alerts(&self, i: usize, ui: &mut Ui) {
        let default_color: Rgba = ui.visuals().window_fill().into();

        let read_red_portion = self.ctx.animate_bool_with_time(
            self.get_alert_id(i, MemOp::Read),
            false,
            3.0,
        );
        let write_red_portion = self.ctx.animate_bool_with_time(
            self.get_alert_id(i, MemOp::Write),
            false,
            3.0,
        );

        let read_mixed_color = default_color * (1.0 - read_red_portion)
            + Rgba::RED * read_red_portion;
        let write_mixed_color = default_color * (1.0 - write_red_portion)
            + Rgba::RED * write_red_portion;

        let read_text_color: Color32 = read_mixed_color.into();
        let write_text_color: Color32 = write_mixed_color.into();

        let address_width = self.address_bits + 2;

        ui.colored_label(
            read_text_color,
            format!(
                "Read Miss: {:#0address_width$b}",
                self.read_miss_addresses[i]
            ),
        );
        ui.colored_label(
            write_text_color,
            format!(
                "Write Miss: {:#0address_width$b}",
                self.write_miss_addresses[i]
            ),
        );
    }

    fn draw_cache(&self, i: usize, ui: &mut Ui) {
        let spacing = self.ctx.style().spacing.item_spacing;

        let address_width = self.address_bits + 2;
        let data_width = size_of::<Data>() * 2 + 2;

        let font_id = TextStyle::Monospace.resolve(&self.ctx.style());
        let default_color = ui.visuals().text_color();

        const STATE_HEADER: &str = "State";
        const ADDRESS_HEADER: &str = "Address";
        const DATA_HEADER: &str = "Data";
        const HEADERS: [&str; 3] = [STATE_HEADER, ADDRESS_HEADER, DATA_HEADER];

        let consultant_painter = ui.painter();

        let letter_size = consultant_painter
            .layout_no_wrap("M".to_owned(), font_id.clone(), default_color)
            .rect;

        let get_width = |text| {
            consultant_painter
                .layout_no_wrap(text, font_id.clone(), default_color)
                .rect
                .width()
        };

        let state_header_width = get_width(STATE_HEADER.to_owned());
        let state_max_width = letter_size.width().max(state_header_width);

        let data_text_width = get_width(format!("{:#0data_width$X}", 0));
        let data_header_width = get_width(DATA_HEADER.to_owned());
        let data_max_width = data_text_width.max(data_header_width);

        let address_text_width = get_width(format!("{:#0address_width$b}", 0));
        let address_header_width = get_width(ADDRESS_HEADER.to_owned());
        let address_max_width = address_text_width.max(address_header_width);

        let grid_width = state_max_width
            + data_max_width
            + address_max_width
            + spacing.x * 6.0;
        let grid_height = (letter_size.height() + spacing.y * 2.0)
            * (self.caches[i].len() + 1) as f32;
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
            grid_rect.left() + state_max_width + spacing.x * 2.0,
            grid_rect.y_range(),
            stroke,
        );
        painter.vline(
            grid_rect.left()
                + state_max_width
                + address_max_width
                + spacing.x * 4.0,
            grid_rect.y_range(),
            stroke,
        );

        let mut x_locs: [f32; 3] = [
            grid_rect.left() + spacing.x,
            grid_rect.left() + spacing.x * 3.0 + state_max_width,
            grid_rect.left()
                + state_max_width
                + address_max_width
                + spacing.x * 5.0,
        ];
        let y = grid_rect.top() + spacing.y;

        for (header, x) in HEADERS.iter().zip(x_locs) {
            painter.text(
                Pos2 { x, y },
                Align2::LEFT_TOP,
                header,
                font_id.clone(),
                default_color,
            );
        }

        painter.hline(
            grid_rect.x_range(),
            grid_rect.top() + 2.0 * spacing.y + letter_size.height(),
            stroke,
        );

        // center columns
        x_locs[0] += (state_max_width - letter_size.width()) / 2.0;
        x_locs[1] += (address_max_width - address_text_width) / 2.0;
        x_locs[2] += (data_max_width - data_text_width) / 2.0;

        for (line_i, cache_line) in self.caches[i].iter().enumerate() {
            let red_portion = self
                .ctx
                .animate_bool(self.get_cache_line_id(i, line_i), false);
            let default_color: Rgba = default_color.into();
            let mixed_color =
                default_color * (1.0 - red_portion) + Rgba::RED * red_portion;
            let text_color: Color32 = mixed_color.into();

            let index = line_i / self.system_props.cache_associativity;
            let address = ((cache_line.tag << self.index_bits) | index)
                << self.offset_bits;

            let y = grid_rect.top()
                + spacing.y * ((line_i + 1) * 2 + 1) as f32
                + letter_size.height() * (line_i + 1) as f32;

            painter.text(
                Pos2 { x: x_locs[0], y },
                Align2::LEFT_TOP,
                cache_line.state.get_letter(),
                font_id.clone(),
                text_color,
            );

            painter.text(
                Pos2 { x: x_locs[1], y },
                Align2::LEFT_TOP,
                format!("{address:#0address_width$b}"),
                font_id.clone(),
                text_color,
            );

            painter.text(
                Pos2 { x: x_locs[2], y },
                Align2::LEFT_TOP,
                format!("{:#0data_width$X}", cache_line.data),
                font_id.clone(),
                text_color,
            );
        }
    }

    fn draw_memory(&mut self, ui: &mut Ui) {
        let spacing = self.ctx.style().spacing.item_spacing;

        let address_width = self.address_bits + 2;
        let data_width = size_of::<Data>() * 2 + 2;

        let font_id = TextStyle::Monospace.resolve(&self.ctx.style());
        let default_color = ui.visuals().text_color();

        const ADDRESS_HEADER: &str = "Address";
        const DATA_HEADER: &str = "Data";
        const HEADERS: [&str; 2] = [ADDRESS_HEADER, DATA_HEADER];

        let consultant_painter = ui.painter();

        let letter_size = consultant_painter
            .layout_no_wrap("M".to_owned(), font_id.clone(), default_color)
            .rect;

        let get_width = |text| {
            consultant_painter
                .layout_no_wrap(text, font_id.clone(), default_color)
                .rect
                .width()
        };

        let data_text_width = get_width(format!("{:#0data_width$X}", 0));
        let data_header_width = get_width(DATA_HEADER.to_owned());
        let data_max_width = data_text_width.max(data_header_width);

        let address_text_width = get_width(format!("{:#0address_width$b}", 0));
        let address_header_width = get_width(ADDRESS_HEADER.to_owned());
        let address_max_width = address_text_width.max(address_header_width);

        let grid_width = data_max_width + address_max_width + spacing.x * 4.0;
        let grid_height = (letter_size.height() + spacing.y * 2.0)
            * (self.main_memory.len() + 1) as f32;
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
            grid_rect.left() + address_max_width + spacing.x * 2.0,
            grid_rect.y_range(),
            stroke,
        );

        let mut x_locs: [f32; 2] = [
            grid_rect.left() + spacing.x,
            grid_rect.left() + address_max_width + spacing.x * 3.0,
        ];
        let y = grid_rect.top() + spacing.y;

        for (header, x) in HEADERS.iter().zip(x_locs) {
            painter.text(
                Pos2 { x, y },
                Align2::LEFT_TOP,
                header,
                font_id.clone(),
                default_color,
            );
        }

        painter.hline(
            grid_rect.x_range(),
            grid_rect.top() + 2.0 * spacing.y + letter_size.height(),
            stroke,
        );

        // center columns
        x_locs[0] += (address_max_width - address_text_width) / 2.0;
        x_locs[1] += (data_max_width - data_text_width) / 2.0;

        for (i, data) in self.main_memory.iter().enumerate() {
            let red_portion =
                self.ctx.animate_bool(self.get_mem_line_id(i), false);
            let default_color: Rgba = default_color.into();
            let mixed_color =
                default_color * (1.0 - red_portion) + Rgba::RED * red_portion;
            let text_color: Color32 = mixed_color.into();

            let address = i << self.offset_bits;

            let y = grid_rect.top()
                + spacing.y * ((i + 1) * 2 + 1) as f32
                + letter_size.height() * (i + 1) as f32;

            painter.text(
                Pos2 { x: x_locs[0], y },
                Align2::LEFT_TOP,
                format!("{address:#0address_width$b}"),
                font_id.clone(),
                text_color,
            );

            painter.text(
                Pos2 { x: x_locs[1], y },
                Align2::LEFT_TOP,
                format!("{data:#0data_width$X}"),
                font_id.clone(),
                text_color,
            );
        }
    }

    fn draw_processor(&mut self, i: usize, ui: &mut Ui) {
        let spacing = self.ctx.style().spacing.item_spacing;
        let width = (ui.available_width()
            - spacing.x * (PROCESSORS_PER_ROW - 1) as f32)
            / PROCESSORS_PER_ROW as f32;
        let height = ui.available_height();

        let layout = Layout::top_down(Align::Center).with_cross_justify(false);

        ui.allocate_ui_with_layout((width, height).into(), layout, |ui| {
            ui.group(|ui| {
                ui.heading(format!("CPU{}", i + 1));

                self.draw_alerts(i, ui);

                self.draw_cache(i, ui);

                ui.add_space(spacing.y * 2.0);
                let label = ui.heading("Last Instruction: ");
                ui.label(format!("{}", self.last_instructions[i]))
                    .labelled_by(label.id);
            })
        });
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.events_rx.try_recv() {
            Ok(event) => match event {
                Event::CacheWrite {
                    cache_i,
                    block_i,
                    line,
                } => {
                    self.caches[cache_i][block_i] = line;
                    self.ctx.animate_bool(
                        self.get_cache_line_id(cache_i, block_i),
                        true,
                    );
                }
                Event::MemWrite { block_i, data } => {
                    self.main_memory[block_i] = data;
                    self.ctx.animate_bool(self.get_mem_line_id(block_i), true);
                }
                Event::Alert {
                    address,
                    processor_i,
                    op,
                } => {
                    match op {
                        MemOp::Write => {
                            self.write_miss_addresses[processor_i] = address
                        }
                        MemOp::Read => {
                            self.read_miss_addresses[processor_i] = address
                        }
                    }
                    self.ctx
                        .animate_bool(self.get_alert_id(processor_i, op), true);
                }
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                panic!("One of the system threads died unexpectedly")
            }
        }

        egui::SidePanel::right("controls_panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| self.controls_panel(ui))
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    (
                        ui.available_width(),
                        ui.available_height() * PROCESSORS_HEIGHT_PERCENT,
                    )
                        .into(),
                    Layout::left_to_right(Align::Min).with_main_wrap(true),
                    |ui| {
                        for i in 0..self.system_props.num_processors {
                            self.draw_processor(i, ui);
                        }
                    },
                );

                ui.allocate_ui_with_layout(
                    (ui.available_width(), ui.available_height()).into(),
                    Layout::top_down(Align::Center),
                    |ui| {
                        ui.group(|ui| {
                            ui.heading("Mem");

                            self.draw_memory(ui);
                        });
                    },
                );
            });
        });
    }
}
