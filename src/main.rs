use eframe::egui;

use egui::{
    widgets,
    Vec2,
    vec2,
    Color32,
    ColorImage,
    ImageData,
    TextureHandle,
    TextureOptions,
};

use std::sync::{Arc, RwLock};



fn main() -> Result<(), eframe::Error> {
    println!("Hello, World!");

    let outputs = Arc::new(RwLock::new([0.0; 10]));

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Hello egui",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    outputs: Arc<RwLock<[f32; 10]>>,
    drawing_texture: Option<TextureHandle>,
    drawing_data: Arc<RwLock<Vec<Color32>>>,
    prev_brush_pos: Option<Vec2>,
}

impl MyApp {
    fn new(__cc: &eframe::CreationContext<'_>) -> Self {
        let drawing_data = vec![Color32::WHITE; 28 * 28];
    
        Self {
            outputs: Arc::new(RwLock::new([0.0; 10])),
            drawing_texture: None,
            drawing_data: Arc::new(RwLock::new(drawing_data)),
            prev_brush_pos: None,
        }
    }

    fn update_drawing(&mut self, ctx: &egui::Context) {
        let pixels = self.drawing_data.write().unwrap();

        let img_data = ImageData::Color(Arc::new(ColorImage {
            size: [28,28],
            pixels: pixels.clone()
        }));

        if let Some(texture) = &mut self.drawing_texture {
            texture.set(img_data, TextureOptions::NEAREST);
        } else {
            // create a new texture
            let texture = ctx.load_texture("drawing_canvas", img_data, TextureOptions::NEAREST);
            self.drawing_texture = Some(texture);
        }
    }
}

impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_drawing(ctx);

        egui::SidePanel::left("output values")
            .resizable(false)
            .show(ctx, |ui| {

            ui.spacing_mut().item_spacing = Vec2::new(4.0, 10.0);
            let output_values = *self.outputs.read().unwrap();
            ui.vertical_centered_justified(|ui| {
                for n in 0..9 {
                    ui.add(widgets::ProgressBar::new(output_values[n])
                        .desired_width(100.0)
                        .desired_height(50.0)
                        .text(n.to_string())
                        .rounding(egui::Rounding::same(6.0)));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Hello World!");
                if ui.button("Clear").clicked() {
                    let img = &mut self.drawing_data.write().unwrap();
                    img.fill(Color32::WHITE);
                }

                let response = ui.add(widgets::Image::from_texture(
                        &self.drawing_texture.clone().unwrap()
                ).maintain_aspect_ratio(true).fit_to_fraction([0.8,0.8].into()));

                if response.contains_pointer() {
                    if ui.input(|i| i.pointer.primary_down()) {
                        if let Some(mouse_pos) = ctx.pointer_latest_pos() {
                            let min = response.rect.min;
                            let max = response.rect.max;
                            let uv = 28.0 * (mouse_pos - min) / (max - min);
                            let mut rounded = uv.floor();

                            //println!("{:?}",&self.drawing_data);
                            //
                            println!("{}, {}; {}, {}",
                                uv.x,
                                uv.y,
                                self.prev_brush_pos.unwrap_or_else(|| vec2(-1.0,-1.0)).x,
                                self.prev_brush_pos.unwrap_or_else(|| vec2(-1.0,-1.0)).y
                            );


                            let img = &mut self.drawing_data.write().unwrap();
                            //println!("{:?}",img);
                            img[(rounded.x as usize) + 28 * ((rounded.y  as usize))] = Color32::BLACK;
                            self.prev_brush_pos = Some(uv.clone());
                        }
                    } else {
                        // mouse is not clicked; break brush line.
                        self.prev_brush_pos = None;
                    }
                }

            });
        });

    }

}
