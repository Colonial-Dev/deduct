use egui::*;

use crate::check::Checker;

use crate::parse::Proof;
use crate::parse::normalize_ops;

const LINE_NUMBER_FONT_SIZE : f32 = 15.0;
const SENTENCE_FONT_SIZE    : f32 = 15.0;
const LINE_NUMBER_VERT_PAD  : f32 = 10.0;
const LINE_NUMBER_HORI_PAD  : f32 = 0.0;
const LEFT_LINE_HORI_PAD    : f32 = LINE_NUMBER_HORI_PAD + 5.0;
const SUBPROOF_INDENTATION  : f32 = 15.0;
const SUBPROOF_LINE_PAD     : f32 = 5.0;
const SENTENCE_CITATION_PAD : f32 = 10.0;

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
    pub conclusion : String,
    pub premises   : Vec<String>,
    pub lines      : Vec<LineUi>,
    pub output     : Vec<String>,
    pub focus_to   : Option<usize>,
    pub current    : Option<usize>,
    pub checker    : Checker,
    pub updated    : bool,
    pub transform  : emath::TSTransform,
}

impl ProofUi {
    fn draw_surroundings(&mut self, ui: &mut Ui, p: &Painter) -> (f32, f32) {
        // Prefetch TeX mathematics font.
        let font = FontId::new(
            SENTENCE_FONT_SIZE,
            FontFamily::Name( "math".into() )
        );

        let text_color = ui.visuals().strong_text_color();
        
        // Compute the text layout of the largest line number.
        let max = p.layout_no_wrap(
            format!( "{}", self.lines.len() ),
            FontId::monospace(LINE_NUMBER_FONT_SIZE),
            text_color
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
            text_color
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
                    text_color
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
                y + (h + LINE_NUMBER_VERT_PAD) * self.premises.len() as f32 - LINE_NUMBER_VERT_PAD,
                Stroke::new(1.0, text_color)
            );
        }

        // Render the line numbers down the left side of the proof body.
        for (i, _) in self.lines.iter().enumerate() {
            let mut text = text::LayoutJob::simple_singleline(
                format!("{}", i + 1),
                FontId::monospace(15.0),
                text_color
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
            Stroke::new(1.0, text_color)
        );

        // Return the computed line number width and height for use in rendering the proof body.
        (w, h)
    }

    fn draw_linectl(&mut self, n: usize, ui: &mut Ui) {
        let premise = self.lines[n].premise;
        let depth   = self.lines[n].depth;

        // The delete line button is available everywhere except the starting premises.
        #[allow(clippy::nonminimal_bool)]
        if !(premise && depth == 0) && !(self.premises.is_empty() && n == 0) && ui.button("X")
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

                if end == n {
                    self.lines.remove(n);
                } else {
                    self.lines.drain(n..=end); 
                }
            }
            else {
                self.lines.remove(n);
            }

            self.updated = true;
        }

        // The new line below button is universal.
        if ui.button("NL")
            .on_hover_text("Create a new line below this one")
            .clicked() 
            {
                self.insert_line(n, false, depth);
            }
        
        // The new subproof below button is universal.
        if ui.button("NS")
            .on_hover_text("Create a new subproof below this line")
            .clicked() 
            {
                self.insert_line(n, true, depth + 1);
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
                    self.insert_line(n, false, depth - 1);
                }

            if ui.button("NSO")
                .on_hover_text("Create a new subproof below/outside this one")
                .clicked()
                {
                    self.insert_line(n, true, depth);
                }
        }
    }

    pub fn insert_line(&mut self, idx: usize, premise: bool, depth: u16) {
        self.lines.insert(
            idx + 1,
            LineUi::new(premise, depth)
        );

        self.focus_to = Some(idx + 1);
    }

    pub fn draw(&mut self, ui: &mut Ui) {          
        let p = ui.painter().to_owned();

        let font = FontId::new(
            SENTENCE_FONT_SIZE,
            FontFamily::Name( "math".into() )
        );

        let text_color = ui.visuals().strong_text_color();
        
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
                text_color
            );

            let c = p.layout_no_wrap(
                line.citation.clone(),
                font.clone(),
                text_color
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
        let linectl_x_end   = ui.ctx().input(|i| {
            let r = i.screen_rect().x_range();
            r.max
        }) * 0.70;

        for (i, line) in self.lines.iter_mut().enumerate() {
            if line.premise && line.depth == 0 {
                let text = p.layout_no_wrap(
                    line.sentence.clone(),
                    font.clone(),
                    text_color
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
                .text_color(text_color)
                .frame(false)
                .margin(Margin::symmetric(0.0, 0.0))
                .id_source((i, 1));

            let mut x_start = x;
            x_start += SUBPROOF_INDENTATION * line.depth as f32;
            x_start += 2.0;

            let mut x_end = x_start;
            x_end += sentence_max_width;
            
            let res = ui.put(
                Rect::from_two_pos(Pos2::new(x_start, y), Pos2::new(x_end, y + h)),
                te
            );

            if res.changed() {
                line.sentence = normalize_ops(&line.sentence);
                self.updated = true;
            }

            if res.has_focus() {
                self.current = Some(i);
            }

            if Some(i) == self.current && res.lost_focus() {
                self.current = None;
            }
            
            if Some(i) == self.focus_to {
                self.focus_to = None;
                res.request_focus();
            }

            // This is a premise, so no citation is needed - 
            // just some fancy lines.
            if line.premise {
                let y_end = y + (h + LINE_NUMBER_VERT_PAD / 2.0);

                p.vline(
                    x + (SUBPROOF_INDENTATION * line.depth as f32) - SUBPROOF_LINE_PAD,
                    y..=y_end,
                    Stroke::new(1.0, text_color)
                );
                
                let mut x_start = x;
                x_start += SUBPROOF_INDENTATION * line.depth as f32;
                x_start -= SUBPROOF_LINE_PAD;

                let mut x_end = citation_x_start;
                x_end -= SENTENCE_CITATION_PAD;

                p.hline(
                    x_start..=x_end,
                    y + (h + LINE_NUMBER_VERT_PAD / 2.0),
                    Stroke::new(1.0, text_color)
                );
            }
            // We're making a deduction, so a citation is needed.
            else {
                // Create field for citation.
                let te = TextEdit::singleline(&mut line.citation)
                    .font(font.clone())
                    .text_color(text_color)
                    .frame(false)
                    .margin(Margin::symmetric(0.0, 0.0))
                    .id_source((i, 2));

                let res = ui.put(
                    Rect::from_two_pos(Pos2::new(citation_x_start, y), Pos2::new(citation_x_end, y + h)),
                    te
                );

                if res.changed() {
                    line.citation = normalize_ops(&line.citation);
                    self.updated = true;
                }

                if res.has_focus() {
                    self.current = Some(i);
                }

                if Some(i) == self.current && res.lost_focus() {
                    self.current = None;
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
                        Stroke::new(1.0, text_color)
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
    
            if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos() ) {
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

impl Widget for &mut ProofUi {
    fn ui(self, ui: &mut Ui) -> Response {
        let (w, h) = super::window_size(ui);

        let (id, rect) = ui.allocate_space(
            Vec2::new(w * 0.70, h * 0.80)
        );

        let transform = &mut self.transform;

        if let Some(pointer) = ui.ctx().input(|i| i.pointer.hover_pos() ) {
            if rect.contains(pointer) {
                let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
                *transform = *transform * emath::TSTransform::from_translation(Vec2::new(0.0, pan_delta.y));
            }
        }

        if transform.translation.y > 0.0 {
            transform.translation.y = 0.0;
        }

        let transform = *transform * emath::TSTransform::from_translation(
            Vec2::new(0.0, ui.min_rect().left_top().y)
        );

        let id = egui::Area::new(id.with("proof_area") )
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                ui.set_clip_rect(transform.inverse() * rect);
                ui.style_mut().wrap = Some(false);
                self.draw(ui)
            })
            .response
            .layer_id;

        ui.ctx().set_transform_layer(id, transform);
        
        ui.separator();

        ui.centered_and_justified( |ui| {
            if self.updated {
                let p: Vec<_> = self
                    .lines
                    .iter()
                    .map(|l| {
                        (l.depth, l.sentence.as_str(), l.citation.as_str())
                    })
                    .collect();

                match Proof::parse(p) {
                    Ok(p) => {
                        if let Err(e) = self.checker.check_proof(&p) {
                            self.output.clear();
                            self.output.push("Invalid proof!".to_string());

                            for (line, err) in e {
                                self.output.push(
                                    format!("line {line}: {err}")
                                )
                            }
                        }
                        else {
                            self.output.clear();
                            if p.reached_conclusion(&self.conclusion) {
                                if p.contains_placeholders() {
                                    self.output.push("You've reached the conclusion, but your proof still contains placeholder citations.".to_string());
                                } else {
                                    self.output.push("This proof is correct!".to_string());
                                }
                            } 
                            else {
                                self.output.push("No errors, but you haven't reached the conclusion.".to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.output.clear();
                        self.output.push("Failed to parse proof!".to_string());

                        for (line, err) in e {
                            self.output.push(
                                format!("line {line}: {err}")
                            )
                        }
                    }
                }

                self.updated = false;
            }

            Frame::group(ui.style())
                .stroke(Stroke::new(1.0, ui.visuals().strong_text_color()))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .max_width(w * 0.75)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                if self.output.is_empty() {
                                    let label = RichText::new("Proof checker idle...")
                                        .italics();

                                    ui.label(label);
                                }

                                let mut lines = self.output.iter().peekable();

                                while let Some(line) = lines.next() {
                                    ui.label(
                                        RichText::new(line).strong()
                                    );

                                    if lines.peek().is_some() {
                                        ui.separator();
                                    }
                                }
                            });
                        });
                });
        });

        super::dummy_response(ui)
    }
}