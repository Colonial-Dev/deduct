use egui::*;
use serde::{Deserialize, Serialize};

mod popups;
mod proof;

#[cfg(not(target_arch = "wasm32"))]
const MODIFIER: Modifiers = Modifiers::ALT;

#[cfg(target_arch = "wasm32")]
const MODIFIER: Modifiers = Modifiers::CTRL.plus(Modifiers::ALT);

const NEW_L: KeyboardShortcut = KeyboardShortcut::new(
    MODIFIER,
    Key::Q
);

const NEW_S: KeyboardShortcut = KeyboardShortcut::new(
    MODIFIER,
    Key::W
);

const NEW_LO: KeyboardShortcut = KeyboardShortcut::new(
    MODIFIER,
    Key::A
);

const NEW_SO: KeyboardShortcut = KeyboardShortcut::new(
    MODIFIER,
    Key::S
);

const UI_ZOOM_FACTORS: [f32; 5] = [1.0, 1.25, 1.50, 1.75, 2.0];

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Deduct {
    #[serde(skip)]
    proof : Option<proof::ProofUi>,
    #[serde(skip)]
    vis   : popups::Visibility,
    #[serde(skip)]
    new   : popups::NewProof,
    prefs : popups::Preferences,
}

impl Deduct {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        fonts_init(cc);

        if let Some(storage) = cc.storage {
            let loaded: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            cc.egui_ctx.set_zoom_factor(UI_ZOOM_FACTORS[loaded.prefs.ui_scale]);
            
            match loaded.prefs.dark_mode {
                false => cc.egui_ctx.set_visuals(Visuals::light()), 
                true => cc.egui_ctx.set_visuals(Visuals::dark())
            }

            return loaded;
        }

        Default::default()
    }

    pub fn try_new_proof(&mut self) {
        if let Some(ui) = self.new.try_create() {
            self.proof = Some(ui);
            self.vis.new_proof = false;
            self.new.ready = false;
        }
    }

    fn handle_shortcuts(&mut self, ctx: &Context) {
        let mut op = None;

        let Some(proof) = &mut self.proof else {
            return
        };

        ctx.input_mut(|i| {
            let n = proof.current.unwrap_or(
                proof.lines.len() - 1
            );

            let d = proof.lines[n].depth;

            if i.consume_shortcut(&NEW_L) {
                op = Some((n, false, d));
            }

            if i.consume_shortcut(&NEW_S) {
                op = Some((n, true, d + 1));
            }

            if i.consume_shortcut(&NEW_LO) && d > 0 {
                op = Some((n, false, d - 1));
            }

            if i.consume_shortcut(&NEW_SO) {
                op = Some((
                    n,
                    true,
                    if d == 0 { 1 } else { d } 
                ));
            }
        });

        if let Some((idx, premise, depth)) = op {
            ctx.memory_mut(|m| m.stop_text_input() );
            proof.insert_line(idx, premise, depth);
        }
    }
}

impl eframe::App for Deduct {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let w = ctx.input(|i| {
            let r = i.screen_rect().x_range();
            r.max
        });

        self.handle_shortcuts(ctx);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Proof", |ui| {
                    if ui.button("New...").clicked() {
                        self.new.reset();
                        self.vis.new_proof = true;
                        self.proof = None;
                        ui.close_menu();
                    };

                    if ui.button("Edit Argument").clicked && self.proof.is_some() {
                        self.vis.new_proof = true;
                        ui.close_menu();
                    }

                    if ui.button("Restart").clicked() && self.proof.is_some() {
                        self.try_new_proof();
                        ui.close_menu();
                    };
                });

                ui.menu_button("Help", |ui| {
                    if ui.hyperlink_to(
                        "Quick Start",
                        "https://github.com/Colonial-Dev/deduct#getting-started"
                    ).clicked() 
                    {
                        ui.close_menu();
                    }

                    if ui.button("Shortcuts").clicked() {
                        self.vis.shortcuts = true;
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("About").clicked() {
                        self.vis.about = true;
                        ui.close_menu();
                    }
                });

                if ui.button("Preferences").clicked() {
                    self.vis.settings = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    egui::warn_if_debug_build(ui);
                });
            });

        });
        
        egui::SidePanel::right("proof_rules")
            .resizable(false)
            .min_width(w * 0.25)
            .max_width(w * 0.25)
            .show(ctx, |ui| {
                containers::ScrollArea::vertical().show(ui, |ui| {                   
                    ui.collapsing("Operator Shorthands", |ui| {
                        ui.label("Negation: ~");
                        ui.label("Conjunction: ^ or &");
                        ui.label("Disjunction: v");
                        ui.label("Biconditional: <->");
                        ui.label("Conditional: ->");
                        ui.label("Contradiction: XX or #");
                        ui.separator();
                        ui.label("Necessity: []");
                        ui.label("Possibility: <>");
                    });

                    ui.separator();
                    ui.collapsing("Basic TFL", |ui| {});
                    ui.collapsing("Derived TFL", |ui| {});
                    ui.collapsing("System K", |ui| {});
                    ui.collapsing("System T", |ui| {});
                    ui.collapsing("System S4", |ui| {});
                    ui.collapsing("System S5", |ui| {});
                });
        });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                let Some(proof) = &mut self.proof else {
                    ui.with_layout(
                        Layout::centered_and_justified(Direction::TopDown),
                        |ui| ui.label("Get started using Proof > New...")
                    );

                    return;
                };

                proof.ui(ui);
            });

        new_window("Preferences", &mut self.vis.settings)
            .show(ctx, |ui| self.prefs.ui(ui) );

        new_window("New Proof", &mut self.vis.new_proof)
            .min_width(w * 0.50)
            .max_width(w * 0.50)
            .show(ctx, |ui| self.new.ui(ui) );

        if self.new.ready {
            self.try_new_proof();
        }

        new_window("About", &mut self.vis.about)
            .show(ctx, about);

        new_window("Keyboard Shortcuts", &mut self.vis.shortcuts)
            .show(ctx, shortcuts);
    }
}

fn about(ui: &mut Ui) {
    let title_font = FontId::new(
        40.0,
        FontFamily::Name( "math".into() )
    );

    ui.vertical_centered(|ui| {
        let label = RichText::new("D∃DUCT").font(title_font)
            .italics()
            .strong()
            .line_height(Some(40.0));

        ui.label(label);
        ui.label("A Fitch-style natural deduction proof checker.");
        
        ui.label(
            RichText::new("Built with love by Colonial, using Rust and egui!").weak().italics()
        );

        ui.separator();

        ui.label(
            format!("Version {}", env!("CARGO_PKG_VERSION"))
        );

        ui.add(
            Hyperlink::from_label_and_url(
                "Source Code", env!("CARGO_PKG_REPOSITORY")
            ).open_in_new_tab(true)
        );

        ui.add(
            Hyperlink::from_label_and_url(
                "Report an Issue", format!("{}/issues", env!("CARGO_PKG_REPOSITORY"))
            ).open_in_new_tab(true)
        );

        ui.separator();

        ui.label("Deduct is licensed under the GNU Affero GPL, version 3.");
        ui.label("This application comes with absolutely no warranty.");

        ui.separator();

        ui.label("Thank you to:\nDr. Sharon Berry\nKevin Klement\nThe Open Logic Project");
    });
}

fn shortcuts(ui: &mut Ui) {
    ui.label("All shortcuts act on the currently selected line or (if no line is selected) the last line.");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Add new line").strong()
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                ui.ctx().format_shortcut(&NEW_L)
            );
        });
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Add new subproof").strong()
        );
        
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                ui.ctx().format_shortcut(&NEW_S)
            );
        });
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Add new line below the current subproof").strong()
        );
        
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                ui.ctx().format_shortcut(&NEW_LO)
            );
        });
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Add new subproof below the current subproof").strong()
        );
        
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                ui.ctx().format_shortcut(&NEW_SO)
            );
        });
    });
}

fn fonts_init(cc: &eframe::CreationContext<'_>) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "math".to_owned(),
        FontData::from_static( include_bytes!("../static/latinmodern-math.otf") )
    );

    fonts.families.insert(
        FontFamily::Name( "math".into() ),
        vec!["math".into()]
    );

    cc.egui_ctx.set_fonts(fonts);
}

fn window_size(ui: &Ui) -> (f32, f32) {
    let w = ui.ctx().input(|i| {
        let r = i.screen_rect().x_range();
        r.max
    });

    let h = ui.ctx().input(|i| {
        let r = i.screen_rect().y_range();
        r.max
    });

    (w, h)
}

fn dummy_response(ui: &mut Ui) -> Response {
    ui.allocate_response(
        Vec2::ZERO,
        Sense::click()
    )
}

fn new_window<'a>(title: &'static str, open: &'a mut bool) -> Window<'a> {
    egui::Window::new(title)
        .default_open(true)
        .collapsible(false)
        .resizable(false)
        .open(open)
        .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
}