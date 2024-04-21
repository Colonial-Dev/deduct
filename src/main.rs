mod check;
mod parse;
mod ui;

use egui::*;
use serde::{Serialize, Deserialize};

use crate::check::Checker;
use crate::check::rulesets::*;

use crate::parse::Proof;

const LINE_NUMBER_FONT_SIZE : f32 = 15.0;
const SENTENCE_FONT_SIZE    : f32 = 15.0;
const LINE_NUMBER_VERT_PAD  : f32 = 10.0;
const LINE_NUMBER_HORI_PAD  : f32 = 0.0;
const LEFT_LINE_HORI_PAD    : f32 = LINE_NUMBER_HORI_PAD + 5.0;
const SUBPROOF_INDENTATION  : f32 = 15.0;
const SUBPROOF_LINE_PAD     : f32 = 5.0;
const SENTENCE_CITATION_PAD : f32 = 15.0;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Deduct {
    #[serde(skip)]
    proof: ProofUi,
    #[serde(skip)]
    check: String,
    #[serde(skip)]
    show_prefs: bool,
}

#[derive(Debug)]
pub struct LineUi {
    pub premise  : bool,
    pub depth    : u16,
    pub sentence : String,
    pub citation : String,
}

#[derive(Default)]
pub struct ProofUi {
    pub premises   : Vec<String>,
    pub conclusion : String,
    pub checker    : Checker,
    pub lines      : Vec<LineUi>,
    transform      : emath::TSTransform,
}

impl ProofUi {
    pub fn new() -> Self {
        let mut new = Self::default();
        new.checker.add_ruleset(check::rulesets::TFL_BASIC);
        new
    }

    fn draw_surroundings(&mut self, ui: &mut Ui, p: &Painter) -> (f32, f32) {
        // Prefetch TeX mathematics font.
        let font = FontId::new(
            SENTENCE_FONT_SIZE,
            FontFamily::Name( "math".into() )
        );
        
        // Compute the text layout of the largest line number.
        let max = p.layout_no_wrap(
            format!( "{}", self.lines.len() ),
            FontId::monospace(LINE_NUMBER_FONT_SIZE),
            ui.visuals().text_color()
        );

        // Pull out the width and height of the largest line number.
        let w = max.rect.width() * 2.0;
        let h = max.rect.height();

        // Init Y-axis pointer value, starting from the top of the painter area.
        let mut y = 0.0;

        // Format premises for "instructions" above the proof.
        let mut premises = String::new();

        for premise in &self.premises {
            premises.push_str(premise);
            premises.push_str(", ");
        }

        let premises = premises
            .trim()
            .trim_end_matches(',');

        // Layout and render the instructions.
        let instructions = p.layout_no_wrap(
            format!("Construct a proof for the argument {premises} ∴ {}", self.conclusion),
            font.clone(),
            ui.visuals().text_color()
        );

        p.galley(
            Pos2::new(w, y),
            instructions,
            Color32::RED
        );

        // Bump Y-axis pointer downwards.
        y += h + LINE_NUMBER_VERT_PAD;

        // If we aren't trying to prove a theorem, outline the space for our premises.
        if !self.premises.is_empty() {
            // Compute the width of the widest premise.
            let mut max_width = 0.0;
            
            for premise in &self.premises {
                let layout = p.layout_no_wrap(
                    premise.to_owned(),
                    font.clone(),
                    ui.visuals().text_color()
                );
    
                if layout.rect.width() > max_width {
                    max_width = layout.rect.width();
                }
            }

            // Fudge factor.
            max_width *= 1.20;

            // Draw a horizontal line separating the premises from the body of the proof.
            p.hline(
                (w + LEFT_LINE_HORI_PAD)..=(w + LEFT_LINE_HORI_PAD + max_width),
                y + (h + LINE_NUMBER_VERT_PAD / 2.0) * self.premises.len() as f32,
                Stroke::new(1.0, ui.visuals().text_color())
            );
        }

        // Render the line numbers down the left side of the proof body.
        for (i, _) in self.lines.iter().enumerate() {
            let mut text = text::LayoutJob::simple_singleline(
                format!("{}", i + 1),
                FontId::monospace(15.0),
                ui.visuals().text_color()
            );
            
            // Manually setting the alignment to RIGHT ensures the numbers "stick"
            // to the leftmost v-line.
            text.halign = Align::RIGHT;

            p.galley(
                Pos2::new(w + LINE_NUMBER_HORI_PAD, y),
                p.layout_job(text),
                Color32::RED
            );

            // Bump y-axis pointer.
            y += h + LINE_NUMBER_VERT_PAD;
        }

        // Draw leftmost vertical line, separating the line numbers from the proof.
        p.vline(
            w + LEFT_LINE_HORI_PAD, 
            0.0 + (h + LINE_NUMBER_VERT_PAD)..=(y - LINE_NUMBER_VERT_PAD),
            Stroke::new(1.0, ui.visuals().text_color())
        );

        // Return the computed line number width and height for use in rendering the proof body.
        (w, h)
    }

    pub fn draw(&mut self, ctx: &Context, ui: &mut Ui) {          
        let p = ui.painter().to_owned();

        let font = FontId::new(
            SENTENCE_FONT_SIZE,
            FontFamily::Name( "math".into() )
        );
        
        let (w, h) = self.draw_surroundings(ui, &p);

        let mut y = 0.0 + (h + LINE_NUMBER_VERT_PAD);
        let x = w + LEFT_LINE_HORI_PAD + 5.0;

        let max_depth = self
            .lines
            .iter()
            .map(|l| l.depth)
            .max()
            .unwrap_or_default();

        let mut sentence_max_width = 0.0;
        let mut citation_max_width = 0.0;

        for line in &self.lines {
            let s = p.layout_no_wrap(
                line.sentence.clone(),
                font.clone(),
                ui.visuals().text_color()
            );

            let c = p.layout_no_wrap(
                line.citation.clone(),
                font.clone(),
                ui.visuals().text_color()
            );

            if s.rect.width() > sentence_max_width {
                sentence_max_width = s.rect.width();
            }

            if c.rect.width() > citation_max_width {
                citation_max_width = c.rect.width();
            }
        }

        sentence_max_width *= 2.0;

        // Compute starting X coordinate for citation field.
        let mut citation_x_start = x;
        citation_x_start += SUBPROOF_INDENTATION * max_depth as f32;
        citation_x_start += sentence_max_width;
        citation_x_start += SENTENCE_CITATION_PAD;

        // Compute ending X coordinate for citation field.
        let mut citation_x_end = citation_x_start;
        citation_x_end += citation_max_width;
        // Fudge factor to account for unreliable text width measurements.
        citation_x_end += SENTENCE_CITATION_PAD;

        let linectl_x_start = citation_x_end;
        let linectl_x_end   = ctx.input(|i| {
            let r = i.screen_rect().x_range();
            r.max
        }) * 0.70;

        for (i, line) in self.lines.iter_mut().enumerate() {
            if line.premise && line.depth == 0 {
                let text = p.layout_no_wrap(
                    line.sentence.clone(),
                    font.clone(),
                    ui.visuals().text_color()
                );

                p.galley(
                    Pos2::new(x, y),
                    text,
                    Color32::RED
                );

                y += h + LINE_NUMBER_VERT_PAD;

                continue;
            }

            let te = TextEdit::singleline(&mut line.sentence)
                .font(font.clone())
                .frame(false)
                .id_source((i, 1));

            let mut x_start = x;
            x_start += SUBPROOF_INDENTATION * line.depth as f32;

            let mut x_end = x;
            x_end += sentence_max_width;
            
            let res = ui.put(
                Rect::from_two_pos(Pos2::new(x_start, y), Pos2::new(x_end, y + h)),
                te
            );

            if res.changed() {
                line.sentence = parse::normalize_ops(&line.sentence);
            }

            // This is a premise, so no citation is needed - 
            // just some fancy lines.
            if line.premise {
                let y_end = y + (h + LINE_NUMBER_VERT_PAD / 2.0);

                p.vline(
                    x + (SUBPROOF_INDENTATION * line.depth as f32) - SUBPROOF_LINE_PAD,
                    y..=y_end,
                    Stroke::new(1.0, ui.visuals().text_color())
                );
                
                let mut x_start = x;
                x_start += SUBPROOF_INDENTATION * line.depth as f32;
                x_start -= SUBPROOF_LINE_PAD;

                let mut x_end = x_start;
                x_end += sentence_max_width;
                x_end += SUBPROOF_LINE_PAD * 2.0;

                p.hline(
                    x_start..=x_end,
                    y + (h + LINE_NUMBER_VERT_PAD / 2.0),
                    Stroke::new(1.0, ui.visuals().text_color())
                );
            }
            // We're making a deduction, so a citation is needed.
            else {
                // Create field for citation.
                let te = TextEdit::singleline(&mut line.citation)
                    .font(font.clone())
                    .frame(false)
                    .id_source((i, 2));

                let res = ui.put(
                    Rect::from_two_pos(Pos2::new(citation_x_start, y), Pos2::new(citation_x_end, y + h)),
                    te
                );

                if res.changed() {
                    line.citation = parse::normalize_ops(&line.citation);
                }
            }

            let r = Rect::from_two_pos(pos2(0.0, y), pos2(linectl_x_end, y + 90.0));

            if let Some(pointer) = ctx.input(|i| i.pointer.hover_pos() ) {
                if r.contains(self.transform.inverse() * pointer) {
                    if line.premise || line.depth == 0 {
                        ui.put(
                            Rect::from_two_pos(pos2(linectl_x_start, y), pos2(linectl_x_end, y + h)),
                            |ui: &mut Ui| ui.horizontal(|ui| {
                                ui.button("X");
                                ui.button("NL");
                                ui.button("NS")
                            }).inner
                        );
                    }
                    else {
                        ui.put(
                            Rect::from_two_pos(pos2(linectl_x_start, y), pos2(linectl_x_end, y + h)),
                            |ui: &mut Ui| ui.horizontal(|ui| {
                                ui.button("X");
                                ui.button("NL");
                                ui.button("NS");
                                ui.button("NLO");
                                ui.button("NSO")
                            }).inner
                        );
                    }      
                }
            }

            // Go back and draw nested subproof lines where needed.
            if line.depth > 0 {
                let y_end = y + (h + LINE_NUMBER_VERT_PAD / 2.0);

                let r = match line.premise {
                    false => 1..=line.depth,
                    true => 1..=(line.depth - 1)
                };

                for i in r {
                    p.vline(
                        x + (SUBPROOF_INDENTATION * i as f32) - SUBPROOF_LINE_PAD,
                        y - (LINE_NUMBER_VERT_PAD / 2.0)..=y_end,
                        Stroke::new(1.0, ui.visuals().text_color())
                    );
                }
            }

            y += h + LINE_NUMBER_VERT_PAD;
        }
    }
}

impl Default for Deduct {
    fn default() -> Self {
        let mut proof = ProofUi::new();

        proof.premises = vec!["¬A".to_owned()];
        proof.conclusion = "¬A".to_owned();

        proof.lines.push(LineUi {
            premise: true,
            depth: 0,
            sentence: "¬A".to_string(),
            citation: "PR".to_string()
        });

        proof.lines.push(LineUi {
            premise: true,
            depth: 1,
            sentence: "A".to_string(),
            citation: "PR".to_string()
        });

        proof.lines.push(LineUi {
            premise: true,
            depth: 2,
            sentence: "A".to_string(),
            citation: "PR".to_string()
        });

        proof.lines.push(LineUi {
            premise: false,
            depth: 2,
            sentence: "#".to_string(),
            citation: "~E 1 2".to_string()
        });

        proof.lines.push(LineUi {
            premise: true,
            depth: 2,
            sentence: "A".to_string(),
            citation: "PR".to_string()
        });

        proof.lines.push(LineUi {
            premise: false,
            depth: 2,
            sentence: "#".to_string(),
            citation: "~E 1 2".to_string()
        });

        proof.lines.push(LineUi {
            premise: false,
            depth: 1,
            sentence: "#".to_string(),
            citation: "~E 1 2".to_string()
        });

        proof.lines.push(LineUi {
            premise: false,
            depth: 0,
            sentence: "~A".to_string(),
            citation: "~I 2-4".to_string()
        });

        Self {
            proof,
            check: Default::default(),
            show_prefs: false,
        }
    }
}

impl Deduct {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "math".to_owned(),
            FontData::from_static( include_bytes!("static/latinmodern-math.otf") )
        );

        fonts.families.insert(
            FontFamily::Name( "math".into() ),
            vec!["math".into()]
        );

        cc.egui_ctx.set_fonts(fonts);

        let mut visuals = Visuals::dark();

        visuals.override_text_color = Some(Color32::WHITE);

        cc.egui_ctx.set_visuals(visuals);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Deduct {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let w = ctx.input(|i| {
            let r = i.screen_rect().x_range();
            r.max
        });

        let h = ctx.input(|i| {
            let r = i.screen_rect().y_range();
            r.max
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                /* NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }*/

                ui.menu_button("Proof", |ui| {
                    ui.button("New...");
                    ui.button("Alter Rulesets");
                    ui.button("Restart");
                    ui.separator();
                    ui.button("Save...");
                    ui.button("Load...");
                });

                ui.menu_button("Help", |ui| {
                    ui.hyperlink_to("Quick Start", "https://github.com/Colonial-Dev/deduct#getting-started");
                    ui.button("Shortcuts");
                    ui.separator();
                    ui.button("About");
                });

                if ui.button("Preferences").clicked() {
                    self.show_prefs = true;
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
                    ui.label("Proof rules go here");
                    ui.collapsing("Basic TFL", |ui| {});
                    ui.collapsing("Derived TFL", |ui| {});
                    ui.collapsing("System K", |ui| {});
                    ui.collapsing("System T", |ui| {});
                    ui.collapsing("System S4", |ui| {});
                    ui.collapsing("System S5", |ui| {});
                });
        });    

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            /* ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));*/
            

            let (id, rect) = ui.allocate_space(Vec2::new(w * 0.70, h * 0.80));

            let transform = &mut self.proof.transform;

            if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos() ) {
                if rect.contains(pointer) {
                    let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
                    *transform = *transform * emath::TSTransform::from_translation(pan_delta);
                }
            }

            if transform.translation.y > 0.0 {
                transform.translation.y = 0.0;
            }

            let transform = *transform * emath::TSTransform::from_translation(
                Vec2::new(0.0, ui.min_rect().left_top().y)
            );
            let id = egui::Area::new(id.with("proof_area") )
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    ui.set_clip_rect(transform.inverse() * rect);
                    ui.style_mut().wrap = Some(false);
                    self.proof.draw(ctx, ui)
                })
                .response
                .layer_id;

            ui.ctx().set_transform_layer(id, transform);
            
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Check Proof").clicked() {
                    let p: Vec<_> = self
                        .proof
                        .lines
                        .iter()
                        .map(|l| {
                            (l.depth, l.sentence.as_str(), l.citation.as_str())
                        })
                        .collect();

                    match Proof::parse(p) {
                        Ok(p) => {
                            if let Err(e) = self.proof.checker.check_proof(&p) {
                                self.check.clear();
                                self.check.push_str("Invalid proof!\n");

                                for (line, err) in e {
                                    self.check.push_str(
                                        &format!("line {line}: {err}\n")
                                    )
                                }
                            }
                            else {
                                self.check.clear();
                                self.check.push_str("This proof is correct!");
                            }
                        }
                        Err(e) => {
                            self.check.clear();
                            self.check.push_str("Failed to parse proof!\n");

                            for (line, err) in e {
                                self.check.push_str(
                                    &format!("line {line}: {err}\n")
                                )
                            }
                        }
                    }
                }
                
                let output = TextEdit::multiline(&mut self.check)
                    .code_editor()
                    .interactive(false)
                    .desired_width(f32::INFINITY)
                    .cursor_at_end(true);

                ui.add(output);
            })
        });

        egui::Window::new("Preferences")
            .default_open(false)
            .open(&mut self.show_prefs)
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Theme: ");
                    egui::global_dark_light_mode_buttons(ui);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("UI Scale: ");
                    // add combo box
                });
            });
    }
}

fn main() {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 500.0])
            .with_min_inner_size([720.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Deduct",
        native_options,
        Box::new(|cc| Box::new(Deduct::new(cc))),
    ).unwrap();
}