use calculator_core::{storage, units, Angle, Calculator, Mode};
use eframe::egui::{self, Color32, RichText};
use std::path::PathBuf;
const ORANGE: Color32 = Color32::from_rgb(239, 145, 36);
const MUTED: Color32 = Color32::from_rgb(158, 162, 174);

// Use ordinary font characters with layout positioning rather than relying on
// Unicode superscript/subscript glyphs that the bundled font may not contain.
fn keypad_label(label: &str, command: &str, size: f32, color: Color32) -> egui::WidgetText {
    if command == "fn:cbrt" {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            "3",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(size * 0.65),
                color,
                valign: egui::Align::TOP,
                ..Default::default()
            },
        );
        job.append(
            "√x",
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(size),
                color,
                valign: egui::Align::BOTTOM,
                ..Default::default()
            },
        );
        return job.into();
    }
    let notation = match command {
        "fn:exp" => Some(("e", "x", false)),
        "fn:exp10" => Some(("10", "x", false)),
        "fn:exp2" => Some(("2", "x", false)),
        "fn:square" => Some(("x", "2", false)),
        "fn:cube" => Some(("x", "3", false)),
        "op:^" => Some(("x", "y", false)),
        "fn:log10" => Some(("log", "10", true)),
        "fn:log2" => Some(("log", "2", true)),
        "log" => Some(("log", "y", true)),
        _ => None,
    };
    if let Some((base, script, subscript)) = notation {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            base,
            0.0,
            egui::TextFormat {
                font_id: egui::FontId::proportional(size),
                color,
                valign: if subscript {
                    egui::Align::TOP
                } else {
                    egui::Align::BOTTOM
                },
                ..Default::default()
            },
        );
        job.append(
            script,
            0.5,
            egui::TextFormat {
                font_id: egui::FontId::proportional(size * 0.65),
                color,
                valign: if subscript {
                    egui::Align::BOTTOM
                } else {
                    egui::Align::TOP
                },
                ..Default::default()
            },
        );
        job.into()
    } else {
        RichText::new(label).size(size).color(color).into()
    }
}

pub struct Desktop {
    pub(crate) calc: Calculator,
    path: Option<PathBuf>,
    saved: String,
    storage_error: Option<String>,
    storage_disabled: bool,
    history_open: bool,
    secondary: bool,
    show_bits: bool,
    topmost: bool,
    help_open: bool,
    unit_category: String,
    smoke_frames: Option<u8>,
    #[cfg(test)]
    layout_bounds: Option<(egui::Rect, egui::Rect)>,
}
impl Desktop {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::with_path(&cc.egui_ctx, storage::default_path());
        app.resize(&cc.egui_ctx);
        app
    }
    fn with_path(ctx: &egui::Context, path: Option<PathBuf>) -> Self {
        // The calculator's colours are designed for dark mode, regardless of OS preference.
        ctx.set_theme(egui::ThemePreference::Dark);
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(29, 30, 35);
        style.visuals.window_fill = Color32::from_rgb(34, 35, 41);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 8.0);
        ctx.set_style(style);
        let loaded = path.as_deref().map(storage::load).transpose();
        let (calc, storage_error, storage_disabled) = match loaded {
            Ok(calc) => (calc.unwrap_or_default(), None, false),
            Err(err) => (Calculator::default(), Some(err.to_string()), true),
        };
        let unit_category = calc
            .conversion
            .map(|(a, _)| units::UNITS[a].category.to_owned())
            .unwrap_or_else(|| "Length".into());
        let saved = storage::encode(&calc);
        Self {
            calc,
            path,
            saved,
            storage_error,
            storage_disabled,
            history_open: false,
            secondary: false,
            show_bits: true,
            topmost: false,
            help_open: false,
            unit_category,
            smoke_frames: std::env::var_os("CALCULATOR_SMOKE_TEST").map(|_| 3),
            #[cfg(test)]
            layout_bounds: None,
        }
    }
    fn persist(&mut self) {
        if self.storage_disabled {
            return;
        }
        let encoded = storage::encode(&self.calc);
        if encoded == self.saved {
            return;
        }
        if let Some(path) = &self.path {
            match storage::save(path, &self.calc) {
                Ok(()) => {
                    self.saved = encoded;
                    self.storage_error = None;
                }
                Err(e) => self.storage_error = Some(e.to_string()),
            }
        }
    }
    fn resize(&self, ctx: &egui::Context) {
        let width = if self.calc.mode == Mode::Basic {
            440.0
        } else {
            860.0
        } + if self.history_open { 280.0 } else { 0.0 };
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(width, 600.0)));
        let minimum_width = if self.calc.mode == Mode::Basic {
            400.0
        } else {
            760.0
        } + if self.history_open { 220.0 } else { 0.0 };
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(
            minimum_width,
            600.0,
        )));
    }
    fn press(&mut self, command: &str) {
        if let Some(text) = command.strip_prefix("text:") {
            self.calc.append(text);
            return;
        }
        if let Some(op) = command.strip_prefix("op:") {
            self.calc.operator(op);
            return;
        }
        if let Some(name) = command.strip_prefix("fn:") {
            self.calc.apply_function(name);
            return;
        }
        if let Some(name) = command.strip_prefix("const:") {
            self.calc.constant(name);
            return;
        }
        match command {
            "AC" => self.calc.clear(),
            "C" => self.calc.clear_entry(),
            "delete" => self.calc.delete(),
            "=" => self.calc.equals(),
            "%" => self.calc.percent(),
            "sign" => self.calc.change_sign(),
            "MC" | "MR" | "M+" | "M-" => self.calc.memory(command),
            "second" => self.secondary = !self.secondary,
            "root" | "rroot" | "log" | "pow" => {
                if self.calc.rpn {
                    self.calc.operator(command);
                    return;
                }
                let current = if self.calc.input.trim().is_empty() {
                    "0".to_owned()
                } else {
                    self.calc.input.clone()
                };
                self.calc.input = format!("{command}({current},");
                self.calc.edited();
            }
            "EE" => self.calc.append("e"),
            text if text.len() == 1 && text.chars().next().unwrap().is_ascii_hexdigit() => {
                self.calc.digit(text.chars().next().unwrap())
            }
            _ => {}
        }
    }
    fn keyboard(&mut self, ctx: &egui::Context) {
        let focused = ctx.wants_keyboard_input();
        for event in ctx.input(|i| i.events.clone()) {
            match event {
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                } => self.calc.equals(),
                egui::Event::Key {
                    key: egui::Key::Escape,
                    pressed: true,
                    ..
                } => self.calc.clear(),
                egui::Event::Key {
                    key: egui::Key::Backspace,
                    pressed: true,
                    ..
                } if !focused => self.calc.delete(),
                egui::Event::Copy if !focused => ctx.copy_text(self.calc.preview()),
                egui::Event::Paste(text) if !focused => self.calc.append(&text),
                egui::Event::Text(text) if !focused => {
                    for c in text.chars() {
                        match c {
                            '=' => self.calc.equals(),
                            '%' => self.calc.percent(),
                            '+' | '-' | '*' | '/' | '^' | '&' | '|' => {
                                self.calc.operator(&c.to_string())
                            }
                            'p' if self.calc.mode != Mode::Programmer => self.calc.constant("pi"),
                            '0'..='9' | 'A'..='F' | 'a'..='f'
                                if self.calc.mode == Mode::Programmer =>
                            {
                                self.calc.digit(c.to_ascii_uppercase())
                            }
                            '0'..='9' | '.' | '(' | ')' => self.calc.append(&c.to_string()),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                for mode in [Mode::Basic, Mode::Scientific, Mode::Programmer] {
                    if ui
                        .selectable_label(self.calc.mode == mode, mode.name())
                        .clicked()
                    {
                        self.calc.set_mode(mode);
                        self.resize(ctx);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("?")
                        .on_hover_text("Keyboard shortcuts and calculator policies")
                        .clicked()
                    {
                        self.help_open = true;
                    }
                    if ui.selectable_label(self.history_open, "History").clicked() {
                        self.history_open = !self.history_open;
                        self.resize(ctx);
                    }
                });
            });
            ui.add_space(5.0);
        });
    }
    fn history(&mut self, ctx: &egui::Context) {
        if !self.history_open {
            return;
        }
        egui::SidePanel::right("history")
            .default_width(260.0)
            .width_range(220.0..=420.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("History");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !self.calc.history.is_empty(),
                                egui::Button::new("Clear all"),
                            )
                            .clicked()
                        {
                            self.calc.history.clear();
                        }
                    });
                });
                ui.label(
                    RichText::new("Select a calculation to restore it")
                        .small()
                        .color(MUTED),
                );
                ui.separator();
                let entries = self.calc.history.clone();
                let mut restore = None;
                let mut delete = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if entries.is_empty() {
                        ui.add_space(30.0);
                        ui.label("No calculations yet");
                    }
                    for (index, h) in entries.iter().enumerate().rev() {
                        ui.push_id(index, |ui| {
                            egui::Frame::new()
                                .fill(Color32::from_rgb(39, 41, 48))
                                .corner_radius(8.0)
                                .inner_margin(10.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(h.mode.name()).small().color(MUTED));
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .small_button("×")
                                                    .on_hover_text("Delete this calculation")
                                                    .clicked()
                                                {
                                                    delete = Some(index);
                                                }
                                            },
                                        );
                                    });
                                    if ui
                                        .add(egui::Button::new(&h.expression).wrap().frame(false))
                                        .on_hover_text("Restore expression and mode")
                                        .clicked()
                                    {
                                        restore = Some(h.clone());
                                    }
                                    ui.label(
                                        RichText::new(format!("= {}", h.result))
                                            .size(22.0)
                                            .color(ORANGE),
                                    );
                                    if let Some((a, b)) = h.conversion {
                                        ui.label(
                                            RichText::new(format!(
                                                "{} → {}",
                                                units::UNITS[a].name,
                                                units::UNITS[b].name
                                            ))
                                            .small(),
                                        );
                                    }
                                });
                            ui.add_space(4.0);
                        });
                    }
                });
                if let Some(entry) = restore {
                    self.calc.restore(&entry);
                    self.unit_category = self
                        .calc
                        .conversion
                        .map(|(a, _)| units::UNITS[a].category.into())
                        .unwrap_or_else(|| "Length".into());
                    self.resize(ctx);
                }
                if let Some(index) = delete {
                    self.calc.history.remove(index);
                }
            });
    }
    fn settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal_wrapped(|ui| {
            if self.calc.mode == Mode::Scientific {
                if ui
                    .selectable_value(&mut self.calc.angle, Angle::Degrees, "Degrees")
                    .clicked()
                {
                    self.calc.edited();
                }
                if ui
                    .selectable_value(&mut self.calc.angle, Angle::Radians, "Radians")
                    .clicked()
                {
                    self.calc.edited();
                }
            }
            if self.calc.mode != Mode::Programmer {
                let mut rpn = self.calc.rpn;
                if ui.checkbox(&mut rpn, "RPN").changed() {
                    self.calc.set_rpn(rpn);
                }
                if !rpn {
                    let mut conversion = self.calc.conversion.is_some();
                    if ui.checkbox(&mut conversion, "Convert").changed() {
                        self.calc.conversion = if conversion { Some((0, 1)) } else { None };
                        self.unit_category = "Length".into();
                        self.calc.edited();
                    }
                }
            }
            if ui.checkbox(&mut self.topmost, "Always on top").changed() {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if self.topmost {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                }));
            }
        });
        if self.calc.mode == Mode::Programmer {
            ui.horizontal_wrapped(|ui| {
                for (base, label) in [(2, "BIN"), (8, "OCT"), (10, "DEC"), (16, "HEX")] {
                    if ui
                        .selectable_label(self.calc.integer.base == base, label)
                        .clicked()
                    {
                        self.calc.set_base(base);
                    }
                }
                ui.separator();
                for width in [8, 16, 32, 64] {
                    if ui
                        .selectable_label(self.calc.integer.width == width, format!("{width}-bit"))
                        .clicked()
                    {
                        self.calc.set_width(width);
                    }
                }
                if ui
                    .checkbox(&mut self.calc.integer.signed, "Signed")
                    .changed()
                {
                    self.calc.edited();
                }
                if ui
                    .checkbox(&mut self.calc.integer.arithmetic_shift, "Arithmetic shift")
                    .changed()
                {
                    self.calc.edited();
                }
                ui.checkbox(&mut self.show_bits, "Bits");
            });
        }
    }
    fn conversions(&mut self, ui: &mut egui::Ui) -> Option<[egui::Rect; 4]> {
        let (mut from, mut to) = self.calc.conversion?;
        let before = (from, to);
        let bounds = ui
            .horizontal_top(|ui| {
                ui.spacing_mut().button_padding = egui::vec2(6.0, 6.0);
                ui.spacing_mut().interact_size.y = 32.0;
                let swap_width = 48.0;
                let dropdown_width =
                    (ui.available_width() - swap_width - 3.0 * ui.spacing().item_spacing.x) / 3.0;
                let cell_size = egui::vec2(dropdown_width, 32.0);
                let category = ui
                    .allocate_ui_with_layout(
                        cell_size,
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_max_width(dropdown_width);
                            egui::ComboBox::from_id_salt("category")
                                .width(dropdown_width)
                                .truncate()
                                .selected_text(&self.unit_category)
                                .show_ui(ui, |ui| {
                                    for category in
                                        ["Length", "Mass", "Temperature", "Time", "Volume", "Area"]
                                    {
                                        ui.selectable_value(
                                            &mut self.unit_category,
                                            category.into(),
                                            category,
                                        );
                                    }
                                })
                                .response
                        },
                    )
                    .inner;
                let available: Vec<_> = units::UNITS
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| u.category == self.unit_category)
                    .collect();
                if units::UNITS[from].category != self.unit_category {
                    from = available[0].0;
                    to = available[1].0;
                }
                let from_control = ui
                    .allocate_ui_with_layout(
                        cell_size,
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_max_width(dropdown_width);
                            egui::ComboBox::from_id_salt("from_unit")
                                .width(dropdown_width)
                                .truncate()
                                .selected_text(units::UNITS[from].name)
                                .show_ui(ui, |ui| {
                                    for (index, unit) in &available {
                                        ui.selectable_value(&mut from, *index, unit.name);
                                    }
                                })
                                .response
                        },
                    )
                    .inner;
                let swap = ui
                    .add_sized([swap_width, 32.0], egui::Button::new("Swap"))
                    .on_hover_text("Swap units");
                if swap.clicked() {
                    std::mem::swap(&mut from, &mut to);
                }
                let to_control = ui
                    .allocate_ui_with_layout(
                        cell_size,
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_max_width(dropdown_width);
                            egui::ComboBox::from_id_salt("to_unit")
                                .width(dropdown_width)
                                .truncate()
                                .selected_text(units::UNITS[to].name)
                                .show_ui(ui, |ui| {
                                    for (index, unit) in &available {
                                        ui.selectable_value(&mut to, *index, unit.name);
                                    }
                                })
                                .response
                        },
                    )
                    .inner;
                [category.rect, from_control.rect, swap.rect, to_control.rect]
            })
            .inner;
        self.calc.conversion = Some((from, to));
        if before != (from, to) {
            self.calc.edited();
        }
        Some(bounds)
    }
    fn display(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(Color32::from_rgb(22, 23, 28))
            .corner_radius(14.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if self.calc.rpn {
                            "STACK CALCULATOR"
                        } else {
                            "EXPRESSION"
                        })
                        .small()
                        .color(MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Copy").clicked() {
                            ctx.copy_text(self.calc.preview());
                        }
                    });
                });
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.calc.input)
                        .id(egui::Id::new("expression-input"))
                        .font(egui::TextStyle::Monospace)
                        .hint_text("0")
                        .desired_width(f32::INFINITY)
                        .frame(false),
                );
                if response.changed() {
                    self.calc.edited();
                }
                if self.calc.rpn && !self.calc.stack.is_empty() {
                    let top: Vec<_> = self.calc.stack.iter().rev().take(4).collect();
                    for (row, values) in top.chunks(2).enumerate() {
                        ui.horizontal(|ui| {
                            let width = (ui.available_width() - 8.0) / 2.0;
                            for (column, value) in values.iter().enumerate() {
                                let text = format!(
                                    "{}: {}",
                                    row * 2 + column + 1,
                                    calculator_core::format_number(**value)
                                );
                                ui.add_sized(
                                    [width, 20.0],
                                    egui::Label::new(RichText::new(&text).monospace().color(MUTED))
                                        .truncate(),
                                )
                                .on_hover_text(text);
                            }
                        });
                    }
                }
                // A horizontal row bounds the result to its text height. A bare
                // with_layout here consumes the remaining vertical scroll area.
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let result = self.calc.preview();
                        ui.add(egui::Label::new(RichText::new(&result).size(36.0)).truncate())
                            .on_hover_text(result);
                    });
                });
                if let Some(error) = &self.calc.error {
                    ui.label(RichText::new(error).color(Color32::from_rgb(255, 139, 129)));
                }
                if let Some((_, to)) = self.calc.conversion {
                    ui.label(RichText::new(units::UNITS[to].name).color(ORANGE));
                }
            });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for action in ["MC", "MR", "M+", "M-"] {
                if ui.button(action).clicked() {
                    self.calc.memory(action);
                }
            }
            let memory = if self.calc.mode == Mode::Programmer {
                self.calc
                    .integer_memory
                    .map(|n| self.calc.integer.format(n))
            } else {
                self.calc.memory.map(calculator_core::format_number)
            };
            if let Some(memory) = memory {
                ui.label(RichText::new(format!("M: {memory}")).small().color(MUTED));
            }
            if ui
                .button("C")
                .on_hover_text("Clear current entry")
                .clicked()
            {
                self.calc.clear_entry();
            }
        });
    }
    fn keypad(&mut self, ui: &mut egui::Ui) {
        let extra: Vec<Vec<(&str, &str)>> = match self.calc.mode {
            Mode::Scientific => vec![
                vec![
                    ("2nd", "second"),
                    ("(", "text:("),
                    (")", "text:)"),
                    ("x²", "fn:square"),
                    ("x³", "fn:cube"),
                    ("x^y", "op:^"),
                ],
                vec![
                    ("e^x", "fn:exp"),
                    ("10^x", "fn:exp10"),
                    ("1/x", "fn:recip"),
                    ("√x", "fn:sqrt"),
                    ("cbrt", "fn:cbrt"),
                    ("y√x", "root"),
                ],
                vec![
                    ("ln", "fn:ln"),
                    ("log10", "fn:log10"),
                    ("log2", "fn:log2"),
                    ("log(y)", "log"),
                    ("2^x", "fn:exp2"),
                    ("x!", "fn:fact"),
                ],
                if self.secondary {
                    vec![
                        ("asin", "fn:asin"),
                        ("acos", "fn:acos"),
                        ("atan", "fn:atan"),
                        ("asinh", "fn:asinh"),
                        ("acosh", "fn:acosh"),
                        ("atanh", "fn:atanh"),
                    ]
                } else {
                    vec![
                        ("sin", "fn:sin"),
                        ("cos", "fn:cos"),
                        ("tan", "fn:tan"),
                        ("sinh", "fn:sinh"),
                        ("cosh", "fn:cosh"),
                        ("tanh", "fn:tanh"),
                    ]
                },
                vec![
                    ("π", "const:pi"),
                    ("e", "const:e"),
                    ("EE", "EE"),
                    ("rand", "const:random"),
                    ("x√y", "rroot"),
                    ("mod", "op:mod"),
                ],
            ],
            Mode::Programmer => vec![
                vec![
                    ("A", "A"),
                    ("B", "B"),
                    ("C", "C-digit"),
                    ("D", "D"),
                    ("E", "E"),
                    ("F", "F"),
                ],
                vec![
                    ("AND", "op:&"),
                    ("OR", "op:|"),
                    ("XOR", "op:xor"),
                    ("NOR", "op:nor"),
                    ("NOT", "fn:not"),
                    ("NEG", "fn:neg"),
                ],
                vec![
                    ("<< 1", "fn:shl"),
                    (">> 1", "fn:shr"),
                    ("<< y", "op:<<"),
                    (">> y", "op:>>"),
                    ("ROL", "fn:rol"),
                    ("ROR", "fn:ror"),
                ],
                vec![
                    ("Byte flip", "fn:byteflip"),
                    ("Word flip", "fn:wordflip"),
                    ("MOD", "op:mod"),
                    ("(", "text:("),
                    (")", "text:)"),
                    ("FF", "text:FF"),
                ],
                vec![
                    ("00", "text:00"),
                    ("&", "op:&"),
                    ("|", "op:|"),
                    ("~", "fn:not"),
                    ("XOR", "op:xor"),
                    ("NEG", "fn:neg"),
                ],
            ],
            Mode::Basic => vec![],
        };
        let basic = [
            [("AC", "AC"), ("+/−", "sign"), ("%", "%"), ("÷", "op:/")],
            [("7", "7"), ("8", "8"), ("9", "9"), ("×", "op:*")],
            [("4", "4"), ("5", "5"), ("6", "6"), ("−", "op:-")],
            [("1", "1"), ("2", "2"), ("3", "3"), ("+", "op:+")],
            [
                ("0", "0"),
                (".", "text:."),
                ("Del", "delete"),
                (if self.calc.rpn { "Enter" } else { "=" }, "="),
            ],
        ];
        let cols = if extra.is_empty() { 4 } else { 10 };
        let width = ((ui.available_width() - 8.0 * (cols - 1) as f32) / cols as f32).max(24.0);
        let footer = if self.calc.mode == Mode::Programmer && self.show_bits {
            68.0
        } else if self.calc.rpn {
            60.0
        } else {
            8.0
        } + if self.storage_error.is_some() {
            36.0
        } else {
            0.0
        };
        let remaining = ui.clip_rect().bottom() - ui.cursor().top();
        // Let the keypad fill the available height instead of leaving unused
        // space below a fixed-height last row when the window is taller.
        let height = ((remaining - footer - 4.0 * 8.0) / 5.0).max(28.0);
        egui::Grid::new("keypad")
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                ui.spacing_mut().button_padding = egui::vec2(6.0, 3.0);
                for (row, keys) in basic.iter().enumerate() {
                    for &(label, command) in extra.get(row).into_iter().flatten().chain(keys.iter())
                    {
                        let operator = command.starts_with("op:") || command == "=";
                        let function = command.starts_with("fn:")
                            || ["AC", "sign", "%", "second"].contains(&command);
                        let color = if operator {
                            ORANGE
                        } else if function {
                            Color32::from_rgb(68, 70, 79)
                        } else {
                            Color32::from_rgb(49, 51, 59)
                        };
                        let enabled = if self.calc.mode == Mode::Programmer {
                            if command == "text:." {
                                false
                            } else if command == "C-digit" || command == "text:FF" {
                                self.calc.integer.base == 16
                            } else if command.len() == 1
                                && command.chars().next().unwrap().is_ascii_hexdigit()
                            {
                                command
                                    .chars()
                                    .next()
                                    .unwrap()
                                    .is_digit(self.calc.integer.base)
                            } else {
                                true
                            }
                        } else {
                            true
                        };
                        let button = egui::Button::new(keypad_label(
                            label,
                            command,
                            if cols == 4 { 25.0 } else { 17.0 },
                            if operator {
                                Color32::BLACK
                            } else {
                                Color32::WHITE
                            },
                        ))
                        .fill(color)
                        .corner_radius(11.0);
                        if ui
                            .add_enabled_ui(enabled, |ui| ui.add_sized([width, height], button))
                            .inner
                            .on_hover_text(
                                command
                                    .replace("fn:", "")
                                    .replace("op:", "")
                                    .replace("text:", ""),
                            )
                            .clicked()
                        {
                            if command == "C-digit" {
                                self.calc.digit('C');
                            } else {
                                self.press(command);
                            }
                        }
                    }
                    ui.end_row();
                }
            });
        if self.calc.rpn {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                for (label, cmd) in [
                    ("Drop", "drop"),
                    ("Swap", "swap"),
                    ("Roll ↑", "up"),
                    ("Roll ↓", "down"),
                ] {
                    if ui.button(label).clicked() {
                        self.calc.stack_action(cmd);
                    }
                }
            });
        }
    }
    fn bits(&self, ui: &mut egui::Ui) {
        if self.calc.mode != Mode::Programmer || !self.show_bits {
            return;
        }
        if let Ok(bits) = self.calc.integer_value() {
            ui.add_space(10.0);
            let text = format!(
                "{:0width$b}",
                bits,
                width = self.calc.integer.width as usize
            );
            let grouped = text
                .as_bytes()
                .chunks(4)
                .map(|b| std::str::from_utf8(b).unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            ui.label(RichText::new(grouped).monospace().color(MUTED));
            let scalar =
                char::from_u32(bits as u32).filter(|c| !c.is_control() && bits <= u32::MAX as u64);
            ui.label(
                RichText::new(format!(
                    "Unicode: {}",
                    scalar.map(|c| c.to_string()).unwrap_or_else(|| "—".into())
                ))
                .small()
                .color(MUTED),
            );
        }
    }
    fn help(&mut self, ctx: &egui::Context) {
        egui::Window::new("About SoftOxide Calculator").open(&mut self.help_open).resizable(false).show(ctx,|ui|{
            ui.set_max_width(480.0);ui.heading("SoftOxide Calculator");ui.label(concat!("Version ", env!("CARGO_PKG_VERSION")));ui.label("A native egui app inspired by Apple Calculator for macOS.");ui.separator();
            ui.label("Enter evaluates; Escape clears; Backspace deletes. Click the expression to edit or paste a formula. Copy uses your system clipboard.");
            ui.label("Scientific: ^ is right-associative power. Functions use parentheses; root(x,n), rroot(n,x), log(x,base). Complete the closing parenthesis after using a two-argument function button.");
            ui.label("RPN: Enter pushes input or duplicates the top. Binary operations consume the top two values. Errors leave operands intact.");
            ui.label("Programmer: exact wrapping 8/16/32/64-bit integers. ^ is XOR, % is remainder. Signed controls division and decimal display; arithmetic shift is a separate choice.");
            ui.label("Real arithmetic uses f64. Percent after +/− is relative to the left value. Unit conversions are offline; currency and Math Notes are not implemented. Apple numerical parity has not been established.");
            if let Some(path)=&self.path{ui.separator();ui.label("History and settings:");ui.label(path.display().to_string());}
        });
    }
}
impl Desktop {
    fn draw(&mut self, ctx: &egui::Context) {
        self.keyboard(ctx);
        self.toolbar(ctx);
        self.history(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.settings(ui, ctx);
                self.conversions(ui);
                let _display_bounds = ui.scope(|ui| self.display(ui, ctx)).response.rect;
                ui.add_space(10.0);
                let _keypad_bounds = ui.scope(|ui| self.keypad(ui)).response.rect;
                #[cfg(test)]
                {
                    self.layout_bounds = Some((_display_bounds, _keypad_bounds));
                }
                self.bits(ui);
                if let Some(error) = &self.storage_error {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("History storage: {error}"))
                            .small()
                            .color(Color32::LIGHT_RED),
                    );
                }
            });
        });
        self.help(ctx);
        self.persist();
        if let Some(frames) = &mut self.smoke_frames {
            if *frames == 0 {
                eprintln!("Native egui smoke test: frames rendered");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                *frames -= 1;
                ctx.request_repaint();
            }
        }
    }
}
impl eframe::App for Desktop {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dark_design_survives_system_theme_changes() {
        let ctx = egui::Context::default();
        ctx.set_theme(egui::ThemePreference::Light);
        let mut app = Desktop::with_path(&ctx, None);
        for system_theme in [egui::Theme::Light, egui::Theme::Dark, egui::Theme::Light] {
            let _ = ctx.run(
                egui::RawInput {
                    system_theme: Some(system_theme),
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            assert_eq!(ctx.theme(), egui::Theme::Dark);
            let style = ctx.style();
            assert!(style.visuals.dark_mode);
            assert_eq!(style.visuals.panel_fill, Color32::from_rgb(29, 30, 35));
            assert_eq!(style.visuals.window_fill, Color32::from_rgb(34, 35, 41));
        }
    }

    #[test]
    fn conversion_controls_share_one_aligned_row() {
        for width in [400.0, 440.0, 860.0] {
            for (category, pair) in [("Temperature", (12, 13)), ("Volume", (21, 22))] {
                let ctx = egui::Context::default();
                let mut app = Desktop::with_path(&ctx, None);
                app.calc.conversion = Some(pair);
                app.unit_category = category.into();
                for _ in 0..2 {
                    let raw = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 600.0),
                        )),
                        ..Default::default()
                    };
                    let _ = ctx.run(raw, |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let bounds = app.conversions(ui).unwrap();
                            let center = bounds[0].center().y;
                            for rect in bounds {
                                assert!(
                                    (rect.center().y - center).abs() < 1.0,
                                    "{width}: {bounds:?}"
                                );
                                assert!(rect.right() <= width - 7.0, "{width}: {bounds:?}");
                            }
                            for controls in bounds.windows(2) {
                                assert!(controls[0].right() <= controls[1].left(), "{bounds:?}");
                            }
                        });
                    });
                }
            }
        }
    }
    #[test]
    fn compact_display_keeps_keypad_in_view() {
        for (mode, size, rpn, conversion) in [
            (Mode::Basic, [400.0, 600.0], false, false),
            (Mode::Basic, [440.0, 700.0], false, false),
            (Mode::Scientific, [760.0, 600.0], false, false),
            (Mode::Programmer, [760.0, 600.0], false, false),
            (Mode::Scientific, [860.0, 700.0], true, false),
            (Mode::Basic, [440.0, 600.0], false, true),
        ] {
            let ctx = egui::Context::default();
            let mut app = Desktop::with_path(&ctx, None);
            app.calc.set_mode(mode);
            if rpn {
                app.calc.set_rpn(true);
                app.calc.stack = vec![1.0, 2.0, 3.0, 4.0];
            }
            if conversion {
                app.calc.conversion = Some((0, 1));
            }
            for _ in 0..3 {
                let raw = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(size[0], size[1]),
                    )),
                    ..Default::default()
                };
                let _ = ctx.run(raw, |ctx| app.draw(ctx));
            }
            let (display, keypad) = app.layout_bounds.unwrap();
            assert!(
                display.height()
                    < if rpn {
                        240.0
                    } else if conversion {
                        210.0
                    } else {
                        180.0
                    },
                "{mode:?}: display {display:?}"
            );
            assert!(
                keypad.bottom() <= size[1] - 8.0,
                "{mode:?}: keypad {keypad:?}, screen {size:?}"
            );
            if mode == Mode::Basic && !conversion {
                assert!(
                    size[1] - keypad.bottom() <= 24.0,
                    "Unfilled space below keypad: {keypad:?}, screen {size:?}"
                );
            }
        }
    }
    #[test]
    fn focused_editor_enter_and_repeat() {
        let ctx = egui::Context::default();
        let mut app = Desktop::with_path(&ctx, None);
        let screen = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(600.0, 800.0),
        ));
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: screen,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("expression-input")));
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Text("2+3".into())],
                screen_rect: screen,
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        assert_eq!(app.calc.input, "2+3");
        for expected in ["5", "8"] {
            let key = egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            };
            let _ = ctx.run(
                egui::RawInput {
                    events: vec![key],
                    screen_rect: screen,
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
            assert_eq!(app.calc.display, expected);
            let _ = ctx.run(
                egui::RawInput {
                    events: vec![egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
        }
    }
    #[test]
    fn native_ui_renders_all_modes() {
        let ctx = egui::Context::default();
        let mut app = Desktop::with_path(&ctx, None);
        for mode in [Mode::Basic, Mode::Scientific, Mode::Programmer] {
            app.calc.set_mode(mode);
            app.history_open = true;
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1200.0, 800.0),
                )),
                ..Default::default()
            };
            let out = ctx.run(raw, |ctx| app.draw(ctx));
            assert!(!out.shapes.is_empty());
        }
        app.calc.set_mode(Mode::Scientific);
        app.calc.set_rpn(true);
        app.calc.append("2");
        app.press("=");
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.draw(ctx));
        app.calc.set_rpn(false);
        app.calc.conversion = Some((12, 13));
        app.unit_category = "Temperature".into();
        let _ = ctx.run(egui::RawInput::default(), |ctx| app.draw(ctx));
    }
    #[test]
    fn keypad_scientific_and_programmer_workflows() {
        let ctx = egui::Context::default();
        let mut app = Desktop::with_path(&ctx, None);
        for key in ["2", "op:+", "3", "op:*", "4", "="] {
            app.press(key);
        }
        assert_eq!(app.calc.display, "14");
        app.calc.set_mode(Mode::Scientific);
        for key in ["3", "0", "fn:sin", "="] {
            app.press(key);
        }
        assert_eq!(app.calc.display, "0.5");
        app.calc.set_mode(Mode::Programmer);
        for key in ["F", "F", "op:&", "0", "F", "="] {
            app.press(key);
        }
        assert_eq!(app.calc.display, "F");
    }
}
