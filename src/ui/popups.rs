use egui::*;
use serde::{Serialize, Deserialize};

use crate::check::*;
use crate::parse::Sentence;
use crate::parse::normalize_ops;

use super::UI_ZOOM_FACTORS;
use super::proof::*;


#[derive(Debug, Default)]
pub struct Visibility {
    pub new_proof : bool,
    pub shortcuts : bool,
    pub settings  : bool,
    pub about     : bool,
}

#[derive(Debug)]
pub struct NewProof {
    pub conclusion : String,
    pub premises   : String,
    pub error      : String,
    pub rules      : [bool; 6],
    pub ready      : bool,
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
            .filter(|s| !s.is_empty() )
            .collect();

        if !self.premises.trim().is_empty() {
            for (i, premise) in premises.iter().enumerate() {
                if let Err(e) = Sentence::parse(premise) {
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
        } else {
            lines.push(
                LineUi::new(true, 1)
            );
        }

        if let Err(e) = Sentence::parse(&self.conclusion) {
            self.error = format!("Conclusion is not well formed ({e})");
            return None;
        }

        for (i, rule) in self.rules.iter().enumerate() {
            if *rule {
                checker.add_ruleset(rulesets::ALL_RULESETS[i])
            }
        }

        let new_ui = ProofUi {
            premises: premises.clone(),
            conclusion: self.conclusion.clone(),
            checker,
            lines,
            ..Default::default()
        };

        Some(new_ui)
    }

    pub fn reset(&mut self) {
        self.premises.clear();
        self.conclusion.clear();
        self.error.clear();
    }
}

impl Default for NewProof {
    fn default() -> Self {
        Self {
            conclusion: String::new(),
            premises: String::new(),
            error: String::new(),
            rules: [true, false, false, false, false, false],
            ready: false,
        }
    }
}

impl Widget for &mut NewProof {
    fn ui(self, ui: &mut Ui) -> Response {
        self.rules[0] = true;

        let font = FontId::new(
            15.0,
            FontFamily::Name( "math".into() )
        );

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.checkbox(&mut self.rules[0], "Basic TFL");
                ui.checkbox(&mut self.rules[1], "Derived TFL");
                ui.checkbox(&mut self.rules[2], "System K");
            });

            ui.vertical(|ui| {
                ui.checkbox(&mut self.rules[3], "System T");
                ui.checkbox(&mut self.rules[4], "System S4");
                ui.checkbox(&mut self.rules[5], "System S5");
            });

            let highest = self
                .rules
                .iter()
                .enumerate()
                .rev()
                .find(|(_, v)| **v)
                .map(|(i, _)| i)
                .unwrap_or_default();

            for i in 0..highest {
                self.rules[i] = true;
            }

            ui.separator();

            ui.vertical(|ui| {
                let p = TextEdit::singleline(&mut self.premises)
                    .hint_text("Premises...")
                    .font(font.clone())
                    .desired_width(f32::INFINITY)
                    .show(ui);

                let c = TextEdit::singleline(&mut self.conclusion)
                    .hint_text("Conclusion...")
                    .font(font.clone())
                    .desired_width(f32::INFINITY)
                    .show(ui);

                if p
                    .response
                    .on_hover_text("Proof premises (comma-separated)")
                    .changed() 
                {
                    self.premises = normalize_ops(&self.premises)
                }

                if c
                    .response
                    .on_hover_text("Proof conclusion")
                    .changed() 
                {
                    self.conclusion = normalize_ops(&self.conclusion)
                }

                ui.label(&self.error);
            });
        });

        ui.separator();
        
        if ui.button("Create proof").clicked() {
            self.ready = true;
        }

        super::dummy_response(ui)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub dark_mode : bool,
    pub ui_scale  : usize,
}

impl Widget for &mut Preferences {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("Theme: ");
            egui::global_dark_light_mode_buttons(ui);
        });

        self.dark_mode = ui.visuals().dark_mode;

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("UI Scale: ");
            
            let r = egui::ComboBox::new("ui_scale_combo", "")
                .show_index(
                    ui,
                    &mut self.ui_scale,
                    UI_ZOOM_FACTORS.len(),
                    |i| format!("{}%", UI_ZOOM_FACTORS[i] * 100.0)
                );

            if r.changed() {
                ui.ctx().set_zoom_factor(UI_ZOOM_FACTORS[self.ui_scale]);
            }
        });

        super::dummy_response(ui)
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self { dark_mode: true, ui_scale: 0 }
    }
}