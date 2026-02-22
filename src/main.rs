use eframe::egui::{self, Align2, Color32, FontId, Pos2, RichText, Shape, Stroke, Vec2};
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
    Polygon,
    Text,
    Eraser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StrokePath {
    points: Vec<[f32; 2]>,
    rgba: [u8; 4],
    thickness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PolygonShape {
    points: Vec<[f32; 2]>,
    rgba: [u8; 4],
    thickness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TextBox {
    pos: [f32; 2],
    text: String,
    rgba: [u8; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CanvasItem {
    Stroke(StrokePath),
    Polygon(PolygonShape),
    Text(TextBox),
}

impl CanvasItem {
    fn draw(&self, painter: &egui::Painter) {
        match self {
            CanvasItem::Stroke(path) => {
                let points: Vec<Pos2> = path.points.iter().map(|p| Pos2::new(p[0], p[1])).collect();
                if points.len() > 1 {
                    painter.add(Shape::line(
                        points,
                        Stroke::new(path.thickness, color_from_rgba(path.rgba)),
                    ));
                }
            }
            CanvasItem::Polygon(poly) => {
                let points: Vec<Pos2> = poly.points.iter().map(|p| Pos2::new(p[0], p[1])).collect();
                if points.len() > 2 {
                    painter.add(Shape::closed_line(
                        points,
                        Stroke::new(poly.thickness, color_from_rgba(poly.rgba)),
                    ));
                }
            }
            CanvasItem::Text(t) => {
                let pos = Pos2::new(t.pos[0], t.pos[1]);
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        pos,
                        Vec2::new((t.text.len() as f32 * 9.0) + 14.0, 30.0),
                    ),
                    6.0,
                    Color32::from_rgba_premultiplied(10, 10, 10, 140),
                );
                painter.text(
                    pos + Vec2::new(7.0, 15.0),
                    Align2::LEFT_CENTER,
                    &t.text,
                    FontId::proportional(18.0),
                    color_from_rgba(t.rgba),
                );
            }
        }
    }
}

fn color_from_rgba(rgba: [u8; 4]) -> Color32 {
    Color32::from_rgba_premultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

struct OpenPenApp {
    palette: Vec<Color32>,
    active_color: usize,
    thickness: f32,
    tool: Tool,
    drawing: Vec<Pos2>,
    polygon_points: Vec<Pos2>,
    items: Vec<CanvasItem>,
    redo_stack: Vec<CanvasItem>,
    eraser_size: f32,
    text_draft: String,
}

impl OpenPenApp {
    fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Num1) || i.key_pressed(egui::Key::F1)) {
            self.set_tool(Tool::Pen);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2) || i.key_pressed(egui::Key::F2)) {
            self.set_tool(Tool::Polygon);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3) || i.key_pressed(egui::Key::F3)) {
            self.set_tool(Tool::Text);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num4) || i.key_pressed(egui::Key::F4)) {
            self.set_tool(Tool::Mouse);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num5) || i.key_pressed(egui::Key::F5)) {
            self.set_tool(Tool::Eraser);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Enter))
            && self.tool == Tool::Polygon
            && self.polygon_points.len() >= 3
        {
            let poly = PolygonShape {
                points: self.polygon_points.iter().map(|p| [p.x, p.y]).collect(),
                rgba: self.palette[self.active_color].to_array(),
                thickness: self.thickness,
            };
            self.items.push(CanvasItem::Polygon(poly));
            self.redo_stack.clear();
            self.polygon_points.clear();
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

                        ui.horizontal_wrapped(|ui| {
                            if ui.selectable_label(self.tool == Tool::Mouse, "🖱 Mouse").clicked() {
                                self.set_tool(Tool::Mouse);
                            }
                            ui.menu_button(
                                RichText::new("✏ Pen").color(self.palette[self.active_color]),
                                |ui| {
                                    self.set_tool(Tool::Pen);
                                    ui.horizontal_wrapped(|ui| {
                                        for (idx, color) in self.palette.iter().copied().enumerate() {
                                            let mut btn =
                                                egui::Button::new(" ").fill(color).min_size(Vec2::splat(22.0));
                                            if self.active_color == idx {
                                                btn = btn.stroke(Stroke::new(2.0, Color32::WHITE));
                                            }
                                            if ui.add(btn).clicked() {
                                                self.active_color = idx;
                                                ui.close_menu();
                                            }
                                        }
                                    });
                                    ui.add(
                                        egui::Slider::new(&mut self.thickness, 1.0..=24.0)
                                            .text("Thickness"),
                                    );
                                },
                            );
                            if ui
                                .selectable_label(self.tool == Tool::Polygon, "⬠ Polygon")
                                .clicked()
                            {
                                self.set_tool(Tool::Polygon);
                            }
                            if ui.selectable_label(self.tool == Tool::Text, "🔤 Text").clicked() {
                                self.set_tool(Tool::Text);
                            }
                            if ui
                                .selectable_label(self.tool == Tool::Eraser, "🧽 Eraser")
                                .clicked()
                            {
                                self.set_tool(Tool::Eraser);
                            }
                        });

                        if self.tool == Tool::Text {
                            ui.horizontal(|ui| {
                                ui.label("Text:");
                                ui.text_edit_singleline(&mut self.text_draft);
                            });
                        }

                        ui.horizontal(|ui| {
                            if ui.button("Undo").clicked() {
                                if let Some(item) = self.items.pop() {
                                    self.redo_stack.push(item);
                                }
                            }
                            if ui.button("Redo").clicked() {
                                if let Some(item) = self.redo_stack.pop() {
                                    self.items.push(item);
                                }
                            }
                            if ui.button("Clear").clicked() {
                                self.items.clear();
                                self.redo_stack.clear();
                                self.polygon_points.clear();
                            }
                            if self.tool == Tool::Eraser {
                                ui.add(
                                    egui::Slider::new(&mut self.eraser_size, 8.0..=80.0)
                                        .text("Eraser size"),
                                );
                            }
                        });

                        ui.small("Hotkeys: 1 Pen · 2 Polygon · 3 Text · 4 Mouse · 5 Eraser · Enter closes polygon");
                    });
            });
    }

    fn erase_near(&mut self, center: Pos2) {
        self.items.retain(|item| match item {
            CanvasItem::Stroke(path) => !path
                .points
                .iter()
                .any(|p| Pos2::new(p[0], p[1]).distance(center) <= self.eraser_size),
            CanvasItem::Polygon(poly) => !poly
                .points
                .iter()
                .any(|p| Pos2::new(p[0], p[1]).distance(center) <= self.eraser_size),
            CanvasItem::Text(t) => {
                Pos2::new(t.pos[0], t.pos[1]).distance(center) > self.eraser_size
            }
        });
    }

    fn draw_canvas(&mut self, ctx: &egui::Context) {
        let mut commit_stroke = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let painter = ui.painter_at(rect);

                for item in &self.items {
                    item.draw(&painter);
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
                        commit_stroke = Some(StrokePath {
                            points: self.drawing.iter().map(|p| [p.x, p.y]).collect(),
                            rgba: self.palette[self.active_color].to_array(),
                            thickness: self.thickness,
                        });
                    }
                    if self.drawing.len() > 1 {
                        painter.add(Shape::line(
                            self.drawing.clone(),
                            Stroke::new(self.thickness, self.palette[self.active_color]),
                        ));
                    }
                } else {
                    self.drawing.clear();
                }

                if self.tool == Tool::Polygon {
                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            self.polygon_points.push(pos);
                        }
                    }
                    if self.polygon_points.len() > 1 {
                        painter.add(Shape::line(
                            self.polygon_points.clone(),
                            Stroke::new(self.thickness, self.palette[self.active_color]),
                        ));
                    }
                    for p in &self.polygon_points {
                        painter.circle_filled(*p, 3.0, self.palette[self.active_color]);
                    }
                }

                if self.tool == Tool::Text && response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let text = if self.text_draft.trim().is_empty() {
                            "Text".to_string()
                        } else {
                            self.text_draft.clone()
                        };
                        self.items.push(CanvasItem::Text(TextBox {
                            pos: [pos.x, pos.y],
                            text,
                            rgba: self.palette[self.active_color].to_array(),
                        }));
                        self.redo_stack.clear();
                    }
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

        if let Some(stroke) = commit_stroke {
            self.items.push(CanvasItem::Stroke(stroke));
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
            tool: Tool::Pen,
            drawing: Vec::new(),
            polygon_points: Vec::new(),
            items: Vec::new(),
            redo_stack: Vec::new(),
            eraser_size: 24.0,
            text_draft: "Text".to_string(),
        }
    }
}

impl eframe::App for OpenPenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.handle_shortcuts(ctx);
        self.draw_canvas(ctx);
        self.toolbar(ctx);
        ctx.request_repaint();
    }
}
