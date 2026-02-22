use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};
use serde::{Deserialize, Serialize};

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("OpenPen")
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top()
        .with_inner_size([1280.0, 720.0]);

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
    passthrough: bool,
}

impl OpenPenApp {
    fn tool_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("OpenPen Tools")
            .anchor(egui::Align2::LEFT_TOP, Vec2::new(12.0, 12.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Draw directly on screen");

                ui.horizontal_wrapped(|ui| {
                    for (idx, color) in self.palette.iter().copied().enumerate() {
                        let mut button = egui::Button::new(" ")
                            .fill(color)
                            .min_size(Vec2::splat(22.0));
                        if self.active_color == idx {
                            button = button.stroke(Stroke::new(2.0, Color32::WHITE));
                        }
                        if ui.add(button).clicked() {
                            self.active_color = idx;
                        }
                    }
                });

                ui.add(egui::Slider::new(&mut self.thickness, 1.0..=20.0).text("Thickness"));

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
                });

                if ui
                    .checkbox(&mut self.passthrough, "Click-through mode")
                    .changed()
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(
                        self.passthrough,
                    ));
                }

                ui.small("Esc to close the app.");
            });
    }

    fn draw_canvas(&mut self, ctx: &egui::Context) {
        let mut to_commit = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::drag());

                if response.drag_started() {
                    self.drawing.clear();
                }

                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.drawing.push(pos);
                    }
                }

                if response.drag_stopped() && self.drawing.len() > 1 {
                    let stroke = StrokePath {
                        points: self.drawing.iter().map(|p| [p.x, p.y]).collect(),
                        rgba: self.palette[self.active_color].to_array(),
                        thickness: self.thickness,
                    };
                    to_commit = Some(stroke);
                }

                for path in &self.paths {
                    painter.add(egui::Shape::line(
                        path.as_pos2(),
                        Stroke::new(path.thickness, path.color()),
                    ));
                }

                if self.drawing.len() > 1 {
                    painter.add(egui::Shape::line(
                        self.drawing.clone(),
                        Stroke::new(self.thickness, self.palette[self.active_color]),
                    ));
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
            ],
            active_color: 0,
            thickness: 4.0,
            drawing: Vec::new(),
            paths: Vec::new(),
            redo_stack: Vec::new(),
            passthrough: false,
        }
    }
}

impl eframe::App for OpenPenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        self.tool_panel(ctx);
        self.draw_canvas(ctx);
        ctx.request_repaint();
    }
}
