mod check;
mod parse;
mod ui;

use egui::*;
use serde::{Serialize, Deserialize};

use crate::check::Checker;

use crate::parse::Proof;

const LINE_NUMBER_FONT_SIZE : f32 = 15.0;
const SENTENCE_FONT_SIZE    : f32 = 15.0;
const LINE_NUMBER_VERT_PAD  : f32 = 10.0;
const LINE_NUMBER_HORI_PAD  : f32 = 0.0;
const LEFT_LINE_HORI_PAD    : f32 = LINE_NUMBER_HORI_PAD + 5.0;
const SUBPROOF_INDENTATION  : f32 = 15.0;
const SUBPROOF_LINE_PAD     : f32 = 5.0;
const SENTENCE_CITATION_PAD : f32 = 10.0;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Deduct {
    #[serde(skip)]
    proof: ProofUi,
    #[serde(skip)]
    check: String,
    #[serde(skip)]
    show_prefs: bool,
    #[serde(skip)]
    show_new_p: bool,
    #[serde(skip)]
    new_proof: NewProof,
}

// TODO jfc this is a mess, MODULARIZE

#[derive(Debug, Default)]
pub struct LineUi {
    pub premise  : bool,
    pub depth    : u16,
    pub sentence : String,
    pub citation : String,
}

impl LineUi {
    pub fn new(premise: bool, depth: u16) -> Self {
        let mut citation = String::new();

        if premise {
            citation = "PR".to_string()
        }
        
        Self {
            premise,
            depth,
            citation,
            ..Default::default()
        }
    }
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
    // TODO account for zero-premise case (theorem)
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
        let w = max.rect.width();
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

        let w = w + LINE_NUMBER_VERT_PAD;

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
            max_width += SENTENCE_CITATION_PAD;

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

    fn draw_linectl(&mut self, n: usize, ui: &mut Ui) {
        let premise = self.lines[n].premise;
        let depth   = self.lines[n].depth;

        // The delete line button is available everywhere except the starting premises.
        if !(premise && depth == 0) && ui.button("X")
            .on_hover_text("Remove this line")
            .clicked()
        {
            if premise {
                let mut end = n;

                for i in (n + 1)..self.lines.len() {
                    end = i;
    
                    if (self.lines[i].premise && self.lines[i].depth == depth) || self.lines[i].depth < depth {
                        break;
                    }
                }

                self.lines.drain(n..end); 
            }
            else {
                self.lines.remove(n);
            }
        }

        // The new line below button is universal.
        if ui.button("NL")
            .on_hover_text("Create a new line below this one")
            .clicked() 
            {
                self.lines.insert(
                    n + 1,
                    LineUi::new(false, depth)
                )
            }
        
        // The new subproof below button is universal.
        if ui.button("NS")
            .on_hover_text("Create a new subproof below this line")
            .clicked() 
            {
                self.lines.insert(
                    n + 1,
                    LineUi::new(true, depth + 1)
                )
            }

        let (n_premise, n_depth) = self
            .lines
            .get(n + 1)
            .map(|l| (l.premise, l.depth) )
            .unwrap_or( (false, 0) );

        if (n_premise || n_depth < depth) && depth != 0 {
            if ui.button("NLO")
                .on_hover_text("Create a new line below/outside this subproof")
                .clicked()
                {
                    self.lines.insert(
                        n + 1,
                        LineUi::new(false, depth - 1)
                    )
                }

            if ui.button("NSO")
                .on_hover_text("Create a new subproof below/outside this one")
                .clicked()
                {
                    self.lines.insert(
                        n + 1,
                        LineUi::new(true, depth)
                    )
                }
        }
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

        // Fudge factor.
        sentence_max_width += SENTENCE_CITATION_PAD;

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

            let mut x_end = x_start;
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

                let mut x_end = citation_x_start;
                x_end -= SENTENCE_CITATION_PAD;

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

        let mut y = 0.0 + (h + LINE_NUMBER_VERT_PAD);

        for i in 0..self.lines.len() {
            let hover_zone = Rect::from_two_pos(pos2(0.0, y), pos2(linectl_x_end, y + 90.0));

            let linectl_r = Rect::from_two_pos(
                pos2(linectl_x_start, y),
                pos2(linectl_x_end, y + h)
            );

            // Because the size can change during loops, we add a check
            // and break if we're out of bounds.           
            if i >= self.lines.len() {
                break;
            }
    
            if let Some(pointer) = ctx.input(|i| i.pointer.hover_pos() ) {
                if hover_zone.contains(self.transform.inverse() * pointer) {
                    ui.put(
                        linectl_r,
                        |ui: &mut Ui| ui.horizontal(|ui| {
                            self.draw_linectl(i, ui);

                            ui.allocate_response(
                                Vec2::ZERO,
                                Sense::click()
                            )
                        }).inner
                    );
                }
            }

            y += h + LINE_NUMBER_VERT_PAD;
        }

        if self.transform.translation.y < -y + 100.0 {
            self.transform.translation.y = -y + 100.0;
        }
    }
}

#[derive(Debug)]
pub struct NewProof {
    pub premises   : String,
    pub conclusion : String,
    pub error      : String,
    pub rules      : [bool; 6]
}

impl NewProof {
    pub fn try_create(&mut self) -> Option<ProofUi> {
        let mut checker = Checker::new();
        let mut lines = Vec::new();

        let premises: Vec<_> = self
            .premises
            .split(',')
            .map(str::trim)
            .map(str::to_owned)
            .collect();

        if !self.premises.trim().is_empty() {
            for (i, premise) in premises.iter().enumerate() {
                if let Err(e) = parse::Sentence::parse(premise) {
                    self.error = format!("Premise {} is not well formed ({e})", i + 1);
                    return None;
                }
            }

            for premise in &premises {
                let line = LineUi {
                    premise: true,
                    depth: 0,
                    sentence: premise.to_owned(),
                    citation: "PR".to_owned()
                };
    
                lines.push(line);
            }
        }

        if let Err(e) = parse::Sentence::parse(&self.conclusion) {
            self.error = format!("Conclusion is not well formed ({e})");
            return None;
        }


        for (i, rule) in self.rules.iter().enumerate() {
            if *rule {
                checker.add_ruleset(check::rulesets::ALL_RULESETS[i])
            }
        }


        let new_ui = ProofUi {
            premises: premises.clone(),
            conclusion: self.conclusion.clone(),
            checker,
            lines,
            ..Default::default()
        };

        self.premises.clear();
        self.conclusion.clear();
        self.error.clear();

        Some(new_ui)
    }
}

impl Default for NewProof {
    fn default() -> Self {
        Self {
            premises: String::new(),
            conclusion: String::new(),
            error: String::new(),
            rules: [true, false, false, false, false, false]
        }
    }
}

impl Default for Deduct {
    fn default() -> Self {
        let proof = ProofUi::default();

        Self {
            proof,
            check: Default::default(),
            show_prefs: false,
            show_new_p: false,
            new_proof: Default::default(),
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
                    if ui.button("New...").clicked() {
                        self.show_new_p = true;
                    };

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

        egui::CentralPanel::default()
            .show(ctx, |ui| {
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
            .default_open(true)
            .collapsible(false)
            .resizable(false)
            .open(&mut self.show_prefs)
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
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
        
        egui::Window::new("New Proof")
            .default_open(true)
            .collapsible(false)
            .resizable(false)
            .open(&mut self.show_new_p)
            .anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
            .min_width(w * 0.30)
            .show(ctx, |ui| {
                let font = FontId::new(
                    SENTENCE_FONT_SIZE,
                    FontFamily::Name( "math".into() )
                );

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut self.new_proof.rules[0], "Basic TFL");
                        ui.checkbox(&mut self.new_proof.rules[1], "Derived TFL");
                        ui.checkbox(&mut self.new_proof.rules[2], "System K");
                    });

                    ui.vertical(|ui| {
                        ui.checkbox(&mut self.new_proof.rules[3], "System T");
                        ui.checkbox(&mut self.new_proof.rules[4], "System S4");
                        ui.checkbox(&mut self.new_proof.rules[5], "System S5");
                    });

                    let highest = self
                        .new_proof
                        .rules
                        .iter()
                        .enumerate()
                        .rev()
                        .find(|(_, v)| **v)
                        .map(|(i, _)| i)
                        .unwrap_or_default();

                    for i in 0..highest {
                        self.new_proof.rules[i] = true;
                    }

                    ui.separator();

                    ui.vertical(|ui| {
                        let p = TextEdit::singleline(&mut self.new_proof.premises)
                            .hint_text("Premises...")
                            .font(font.clone())
                            .desired_width(f32::INFINITY)
                            .show(ui);

                        let c = TextEdit::singleline(&mut self.new_proof.conclusion)
                            .hint_text("Conclusion...")
                            .font(font.clone())
                            .desired_width(f32::INFINITY)
                            .show(ui);

                        if p
                            .response
                            .on_hover_text("Proof premises (comma-separated)")
                            .changed() 
                        {
                            self.new_proof.premises = parse::normalize_ops(&self.new_proof.premises)
                        }

                        if c
                            .response
                            .on_hover_text("Proof conclusion")
                            .changed() 
                        {
                            self.new_proof.conclusion = parse::normalize_ops(&self.new_proof.conclusion)
                        }

                        ui.label(&self.new_proof.error);
                    });
                });

                ui.separator();
                
                if ui.button("Create proof").clicked() {
                    if let Some(ui) = self.new_proof.try_create() {
                        self.proof = ui;
                    }
                }
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