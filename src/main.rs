
mod canvas;
mod neural_net;
mod data_reader;

use eframe::egui;

use egui_plot::{Plot, Line, PlotPoints};

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

use canvas::Canvas;

use std::sync::{Arc, RwLock};


fn main() -> Result<(), eframe::Error> {
    println!("Hello, World!");

    let mut nn = neural_net::NeuralNet::new(vec![28 * 28, 5, 5, 10]);

    nn.populate_random_weights();

    let outputs = Arc::new(RwLock::new([0.0; 10]));

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Hello egui",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

enum View {
    Draw,
    Train,
    InspectData,
}

struct MyApp {
    outputs: Arc<RwLock<[f32; 10]>>,
    drawing_texture: Option<TextureHandle>,

    data_view_texture: Option<TextureHandle>,
    data_view_index: usize,
    training_images: Vec<Vec<u8>>,
    training_labels: Vec<u8>,

    drawing_data: Arc<RwLock<Canvas>>,
    prev_brush_pos: Option<Vec2>,
    view: View,
}

impl MyApp {
    fn new(__cc: &eframe::CreationContext<'_>) -> Self {
        let drawing_data = vec![Color32::WHITE; 28 * 28];

        let (training_images, training_labels) = data_reader::get_mnist_images("./data/train-images.idx3-ubyte", "./data/train-labels.idx1-ubyte").unwrap();
    
        Self {
            outputs: Arc::new(RwLock::new([0.0; 10])),
            drawing_texture: None,

            data_view_texture: None,
            data_view_index: 0,
            training_images,
            training_labels,

            drawing_data: Arc::new(RwLock::new(Canvas::new(Color32::WHITE, Color32::BLACK, [28, 28]))),

            prev_brush_pos: None,
            view: View::Draw,
        }
    }

    fn update_drawing(&mut self, ctx: &egui::Context) {

        let canvas = self.drawing_data.write().unwrap();

        let img_data = ImageData::Color(Arc::new(ColorImage {
            size: [28,28],
            pixels: canvas.get_pixels()
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

        ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.spacing.item_spacing = vec2(5.0, 5.0);
        });


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
                        .rounding(egui::Rounding::same(2)));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {

            ui.horizontal_top(|ui| {

                if ui.button("Draw").clicked() {
                    self.view = View::Draw;
                }

                if ui.button("Train").clicked() {
                    self.view = View::Train;
                }

                if ui.button("Inspect Data").clicked() {
                    self.view = View::InspectData;
                }
            });

            match self.view {
                View::Train => {
                    let sin: PlotPoints = (0..1000).map(|i| {
                        let x = i as f64 * 0.01;
                        [x, x.sin()]
                    }).collect();

                    let line = Line::new(sin);
                    Plot::new("test_plot").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));
                },
                
                View::InspectData => {
                    

                    ui.vertical_centered(|ui| {
                        let pixels: Vec<Color32> = self.training_images[self.data_view_index].iter()
                            .map(|&x| Color32::from_gray(255 - x))
                            .collect();

                        let img_data = ImageData::Color(Arc::new(ColorImage {
                            size: [28,28],
                            pixels,
                        }));

                        if let Some(texture) = &mut self.data_view_texture {
                            texture.set(img_data, TextureOptions::NEAREST);
                        } else {
                            // create a new texture
                            let texture = ctx.load_texture("drawing_canvas", img_data, TextureOptions::NEAREST);
                            self.data_view_texture = Some(texture);
                        }

                        ui.add(widgets::Image::from_texture(
                                &self.data_view_texture.clone().unwrap())
                                .maintain_aspect_ratio(true).fit_to_fraction([0.8,0.8].into()));

                        ui.label(egui::widget_text::RichText::new(
                                format!("{}", self.training_labels[self.data_view_index]))
                            .size(20.0));

                        ui.add(egui::Slider::new(&mut self.data_view_index, 0..=self.training_images.len() - 1)
                            .clamping(egui::SliderClamping::Edits)
                            .text("Image ID")
                        );

                        ui.horizontal_top(|ui| {
                            if ui.button("<").clicked() {
                                if self.data_view_index > 0 {
                                    self.data_view_index -= 1;
                                }
                            }

                            if ui.button(">").clicked() {
                                if self.data_view_index < self.training_images.len() - 1 {
                                    self.data_view_index += 1;
                                }
                            }
                        });
                    });

                },

                View::Draw => {
                    self.update_drawing(ctx);

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
                                    //println!("{}, {}; {}, {}",
                                    //    uv.x,
                                    //    uv.y,
                                    //    self.prev_brush_pos.unwrap_or_else(|| vec2(-1.0,-1.0)).x,
                                    //    self.prev_brush_pos.unwrap_or_else(|| vec2(-1.0,-1.0)).y
                                    //);


                                    //let img = &mut self.drawing_data.write().unwrap();
                                    //println!("{:?}",img);
                                    //img[(rounded.x as usize) + 28 * ((rounded.y  as usize))] = Color32::BLACK;
                                    let canvas = &mut self.drawing_data.write().unwrap();

                                    match self.prev_brush_pos {
                                        Some(prev_pos) => {
                                            if let Err(e) = canvas.draw_line(prev_pos, uv) {
                                                panic!("Error: {:?}", e);
                                            }
                                        },
                                        None => {
                                            if let Err(e) = canvas.draw_point(uv) {
                                                panic!("Error: {:?}", e);
                                            }
                                        }   
                                    }
                                    self.prev_brush_pos = Some(uv.clone());
                                }
                            } else {
                                // mouse is not clicked; break brush line.
                                self.prev_brush_pos = None;
                            }
                        } else {
                            // mouse is not on drawing area; break brush line.
                            self.prev_brush_pos = None;
                        }

                        let canvas = &mut self.drawing_data.write().unwrap();

                        ui.add(egui::Slider::new(&mut canvas.brush_size, 1.01..=10.0)
                            .clamping(egui::SliderClamping::Edits)
                            .text("Brush Size")
                        );

                        ui.add(egui::Slider::new(&mut canvas.brush_smoothness, 0.0..=75.0)
                            .clamping(egui::SliderClamping::Edits)
                            .text("Brush Smoothness")
                        );

                        ui.add(egui::Slider::new(&mut canvas.brush_intensity, 0.5..=15.0)
                            .clamping(egui::SliderClamping::Edits)
                            .text("Brush Intensity")
                        );

                    });
                }
            }
        });
    }

}
