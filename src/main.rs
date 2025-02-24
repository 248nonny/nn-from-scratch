
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
use neural_net::{ NeuralNet, NNData };

use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, TryRecvError, Sender};
use std::thread;
use std::iter::zip;

fn main() -> Result<(), eframe::Error> {
    println!("Hello, World!");


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
    ctx: Arc<egui::Context>,

    outputs: [f32; 10],
    drawing_texture: Option<TextureHandle>,

    data_view_texture: Option<TextureHandle>,
    data_view_index: usize,
    training_data: Arc<Vec<neural_net::NNData>>,
    //testing_data: Arc<Vec<neural_net::NNData>>,
    error_data: Arc<RwLock<Vec<f32>>>,
    nn: Arc<RwLock<NeuralNet>>,
    learning_rate: f32,
    training_thread_tx: Option<Sender<()>>,

    drawing_data: Arc<RwLock<Canvas>>,
    prev_brush_pos: Option<Vec2>,
    view: View,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let error_data = Arc::new(RwLock::new(Vec::new()));

        let ctx = Arc::new(cc.egui_ctx.clone());


        let (training_images, training_labels) = data_reader::get_mnist_images("./data/train-images.idx3-ubyte", "./data/train-labels.idx1-ubyte").unwrap();

        //let (testing_images, testing_labels) = data_reader::get_mnist_images("./data/t10k-images.idx3-ubyte", "./data/t10k-labels.idx1-ubyte").unwrap();

        let mut nn = NeuralNet::new();

        nn.populate_random_weights();

        let training_data = Arc::new(zip(training_images, training_labels)
            .map(|(data, label)| NNData { data, label: label as usize }).collect());

        //let testing_data = Arc::new(zip(testing_images, testing_labels)
        //    .map(|(data, label)| NNData { data, label: label as usize }).collect());
    
        Self {
            ctx,

            outputs: [0.0; 10],
            drawing_texture: None,

            data_view_texture: None,
            data_view_index: 0,
            training_data,
            //testing_data,
            error_data,
            nn: Arc::new(RwLock::new(nn)),
            learning_rate: 0.1,
            training_thread_tx: None,
            

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
            ui.vertical_centered_justified(|ui| {
                for n in 0..10 {
                    ui.add(widgets::ProgressBar::new(self.outputs[n])
                        .desired_width(100.0)
                        .desired_height(50.0)
                        .text(n.to_string())
                        .corner_radius(egui::CornerRadius::same(2)));
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
                    //let sin: PlotPoints = (0..1000).map(|i| {
                    //    let x = i as f64 * 0.01;
                    //    [x, x.sin()]
                    //}).collect();

                    //let line = Line::new(sin);

                    let line;
                    {
                        loop {
                            //if let Some(tx) = self.training_thread_tx.as_ref() {
                            //    if let Ok(_) = tx.send(true) {
                            //        thread::sleep(Duration::from_millis(3));
                            //    }
                            //}
                                    

                            let mut i = 0;
                            if let Ok(data_with_lock) = self.error_data.try_read() {

                                let error_points: PlotPoints = data_with_lock.iter()
                                    .map(|y| {
                                        let out = [i as f64, *y as f64];
                                        i += 1;
                                        out
                                    })
                                    .collect();
                                line = Line::new(error_points);
                                break;
                            }
                        }
                    }
                    
                    Plot::new("test_plot").view_aspect(2.0).show(ui, |plot_ui| plot_ui.line(line));

                    if ui.button("Start Training").clicked() {
                        let (tx, rx) = mpsc::channel();

                        self.training_thread_tx = Some(tx);

                        let nn = Arc::clone(&self.nn);
                        let training_data = Arc::clone(&self.training_data);
                        let p_points = Arc::clone(&self.error_data);

                        let ctx_arc = Arc::clone(&self.ctx);

                        let _training_thread = thread::spawn(move || {
                            let mut count = 0;
                            let mut vals = Vec::new();
                            vals.reserve(50);
                            loop {
                                count += 1;
                                if count % 200 == 0 {
                                    match rx.try_recv() {
                                        //Ok(B) if B => thread::sleep(Duration::from_millis(10)),
                                        Ok(_) | Err(TryRecvError::Disconnected) => {
                                            println!("stopping indefinite training thread.");
                                            break;
                                        }
                                        Err(TryRecvError::Empty) => {}
                                    }

                                    p_points.write().unwrap().extend_from_slice(&vals[..]);
                                    vals.clear();
                                    ctx_arc.request_repaint();
                                }
                                
                                vals.push(nn.write().unwrap().train_one(&training_data[rand::random_range(0..training_data.len())]));
                            }
                        });
                    }

                    if ui.button("Stop Training").clicked() {

                        if let Some(tx) = self.training_thread_tx.as_ref() {
                            let _ = tx.send(());
                        }

                        self.training_thread_tx = None;
                    }

                    ui.add(egui::Slider::new(&mut self.learning_rate, 0.0001..=0.4)
                        .clamping(egui::SliderClamping::Edits)
                        .text("Learning Rate")
                    );

                    if let Ok(mut nn) = self.nn.try_write() {
                        nn.set_learning_rate(self.learning_rate);
                    }


                },
                
                View::InspectData => {
                    

                    ui.vertical_centered(|ui| {
                        let pixels: Vec<Color32> = self.training_data[self.data_view_index].data.iter()
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
                                format!("{}", self.training_data[self.data_view_index].label))
                            .size(20.0));

                        ui.add(egui::Slider::new(&mut self.data_view_index, 0..=self.training_data.len() - 1)
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
                                if self.data_view_index < self.training_data.len() - 1 {
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
                            img.clear();
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

                                    if let Ok(nn) = self.nn.try_read() {
                                        let prediction = nn.image_to_prediction(
                                            neural_net::scale_and_normalize_data(
                                                &canvas.get_pixels_as_slice()
                                                .iter()
                                                .map(|color|
                                                    (255.0 - (
                                                        color.r() as f32 +
                                                        color.g() as f32 +
                                                        color.b() as f32
                                                    ) / (3.0)) as u8
                                                ).collect::<Vec<u8>>()
                                            )
                                        );

                                        for i in 0..10 {
                                            self.outputs[i] = prediction[i];
                                        }
                                    }

                                }
                            } else {
                                // mouse is not clicked; break brush line.
                                self.prev_brush_pos = None;
                            }
                        } else {
                            // mouse is not on drawing area; break brush line.
                            self.prev_brush_pos = None;
                        }

                        let mut max = -1.0;
                        let mut final_prediction = 0;
                        for i in 0..10 {
                            if self.outputs[i] > max {
                                max = self.outputs[i];
                                final_prediction = i;
                            }
                        }

                        ui.label(final_prediction.to_string());

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
