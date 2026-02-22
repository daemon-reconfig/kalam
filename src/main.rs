use eframe::egui::{self, Color32, Pos2, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("OpenPen")
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top()
        .with_fullscreen(true);

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "OpenPen",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::<OpenPenApp>::default()
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Mouse,
    Pen,
    Eraser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StrokePath {
    points: Vec<[f32; 2]>,
    rgba: [u8; 4],
    thickness: f32,
}

impl StrokePath {
    fn color(&self) -> Color32 {
        Color32::from_rgba_premultiplied(self.rgba[0], self.rgba[1], self.rgba[2], self.rgba[3])
    }

    fn as_pos2(&self) -> Vec<Pos2> {
        self.points.iter().map(|p| Pos2::new(p[0], p[1])).collect()
    }
}

struct OpenPenApp {
    palette: Vec<Color32>,
    active_color: usize,
    thickness: f32,
    drawing: Vec<Pos2>,
    paths: Vec<StrokePath>,
    redo_stack: Vec<StrokePath>,
    tool: Tool,
    last_passthrough: bool,
    eraser_size: f32,
}

impl OpenPenApp {
    fn set_passthrough_for_tool(&mut self, ctx: &egui::Context) {
        let passthrough = self.tool == Tool::Mouse;
        if passthrough != self.last_passthrough {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(passthrough));
            self.last_passthrough = passthrough;
        }
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::Area::new("bottom_toolbar".into())
            .anchor(egui::Align2::CENTER_BOTTOM, Vec2::new(0.0, -18.0))
            .show(ctx, |ui| {
                egui::Frame::window(ui.style())
                    .rounding(egui::Rounding::same(14.0))
                    .show(ui, |ui| {
                        let drag_bar = ui.add(
                            egui::Label::new(
                                RichText::new("⠿ Drag toolbar / window").color(Color32::LIGHT_GRAY),
                            )
                            .sense(egui::Sense::click_and_drag()),
                        );
                        if drag_bar.dragged() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }

                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(self.tool == Tool::Mouse, "🖱 Mouse")
                                .on_hover_text("Pass input through overlay")
                                .clicked()
                            {
                                self.tool = Tool::Mouse;
                            }

                            let pen_text =
                                RichText::new("✏ Pen").color(self.palette[self.active_color]);
                            ui.menu_button(pen_text, |ui| {
                                self.tool = Tool::Pen;
                                ui.set_min_width(220.0);
                                ui.label("Pen color");

                                ui.horizontal_wrapped(|ui| {
                                    for (idx, color) in self.palette.iter().copied().enumerate() {
                                        let mut button = egui::Button::new(" ")
                                            .fill(color)
                                            .min_size(Vec2::splat(22.0));
                                        if self.active_color == idx {
                                            button =
                                                button.stroke(Stroke::new(2.0, Color32::WHITE));
                                        }
                                        if ui.add(button).clicked() {
                                            self.active_color = idx;
                                            ui.close_menu();
                                        }
                                    }
                                });

                                ui.add(
                                    egui::Slider::new(&mut self.thickness, 1.0..=24.0)
                                        .text("Thickness"),
                                );
                            });

                            if ui
                                .selectable_label(self.tool == Tool::Eraser, "🧽 Eraser")
                                .on_hover_text("Erase strokes")
                                .clicked()
                            {
                                self.tool = Tool::Eraser;
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Undo").clicked() {
                                if let Some(path) = self.paths.pop() {
                                    self.redo_stack.push(path);
                                }
                            }
                            if ui.button("Redo").clicked() {
                                if let Some(path) = self.redo_stack.pop() {
                                    self.paths.push(path);
                                }
                            }
                            if ui.button("Clear").clicked() {
                                self.paths.clear();
                                self.redo_stack.clear();
                            }
                            if self.tool == Tool::Eraser {
                                ui.add(
                                    egui::Slider::new(&mut self.eraser_size, 8.0..=80.0)
                                        .text("Eraser size"),
                                );
                            }
                        });
                    });
            });
    }

    fn erase_near(&mut self, center: Pos2) {
        self.paths.retain(|path| {
            !path
                .as_pos2()
                .iter()
                .any(|point| point.distance(center) <= self.eraser_size)
        });
    }

    fn draw_canvas(&mut self, ctx: &egui::Context) {
        let mut to_commit = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let response = ui.allocate_rect(rect, egui::Sense::drag());
                let painter = ui.painter_at(rect);

                for path in &self.paths {
                    painter.add(egui::Shape::line(
                        path.as_pos2(),
                        Stroke::new(path.thickness, path.color()),
                    ));
                }

                if self.tool == Tool::Pen {
                    if response.drag_started() {
                        self.drawing.clear();
                    }
                    if response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.drawing.push(pos);
                        }
                    }
                    if response.drag_stopped() && self.drawing.len() > 1 {
                        to_commit = Some(StrokePath {
                            points: self.drawing.iter().map(|p| [p.x, p.y]).collect(),
                            rgba: self.palette[self.active_color].to_array(),
                            thickness: self.thickness,
                        });
                    }
                    if self.drawing.len() > 1 {
                        painter.add(egui::Shape::line(
                            self.drawing.clone(),
                            Stroke::new(self.thickness, self.palette[self.active_color]),
                        ));
                    }
                } else {
                    self.drawing.clear();
                }

                if self.tool == Tool::Eraser {
                    if response.dragged() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.erase_near(pos);
                        }
                    }
                    if let Some(pos) = response.hover_pos() {
                        painter.circle_stroke(
                            pos,
                            self.eraser_size,
                            Stroke::new(2.0, Color32::WHITE),
                        );
                    }
                }
            });

        if let Some(stroke) = to_commit {
            self.paths.push(stroke);
            self.redo_stack.clear();
            self.drawing.clear();
        }
    }
}

impl Default for OpenPenApp {
    fn default() -> Self {
        Self {
            palette: vec![
                Color32::from_rgb(255, 77, 77),
                Color32::from_rgb(80, 220, 100),
                Color32::from_rgb(85, 170, 255),
                Color32::from_rgb(255, 230, 90),
                Color32::from_rgb(245, 245, 245),
                Color32::from_rgb(204, 128, 255),
            ],
            active_color: 0,
            thickness: 4.0,
            drawing: Vec::new(),
            paths: Vec::new(),
            redo_stack: Vec::new(),
            tool: Tool::Pen,
            last_passthrough: false,
            eraser_size: 24.0,
        }
    }
}

impl eframe::App for OpenPenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.set_passthrough_for_tool(ctx);
        self.draw_canvas(ctx);
        self.toolbar(ctx);
        ctx.request_repaint();
    }
}
