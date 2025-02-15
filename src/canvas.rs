
use eframe::egui::{Vec2, vec2, Color32};

#[derive(Debug)]
pub struct Canvas {
        pixels: Vec<Color32>,
        size: [usize; 2],

        brush_type: BrushType,
        pub brush_size: f32,
        pub brush_smoothness: f32, //should range from 0 to 100
        pub brush_intensity: f32, // should range from 0.5 to 5

        blank_color: Color32,
        paint_color: Color32,
}

#[derive(Debug)]
pub enum BrushType {
    Hard,
    Smooth,
}

#[derive(Debug)]
pub enum CanvasError {
    OutOfBoundsError,
}

impl Canvas {
    pub fn new(blank_color: Color32, paint_color: Color32, size: [usize; 2]) -> Self {
        let pixels = vec![blank_color; size[0] * size[1]];

        Self {
            pixels,
            size,

            brush_type: BrushType::Smooth,
            brush_size: 1.5,
            brush_smoothness: 50.0, // should range from 0 to 1500
            brush_intensity: 10.4, // should range from 0 to 1

            blank_color,
            paint_color,
        }
    }

    pub fn set_brush_type(&mut self, brush_type: BrushType) {
        self.brush_type = brush_type;
    }

    pub fn set_brush_size(&mut self, brush_size: f32) {
        self.brush_size = brush_size;
    }

    pub fn fill(&mut self, val: Color32) {
        self.pixels.fill(val);
    }

    pub fn get_pixels(&self) -> Vec<Color32> {
        self.pixels.clone()
    }

    pub fn draw_point(&mut self, point: Vec2) -> Result<(), CanvasError> {
        match self.brush_type {
            BrushType::Smooth => {
                for p_w_d in PointPxIter::new(self.brush_size, point) {
                    if p_w_d.px[0] >= 0 && (p_w_d.px[0] as usize) < self.size[0] && p_w_d.px[1] >= 0 {
                        let index = p_w_d.px[0] as usize + self.size[0] * p_w_d.px[1] as usize;

                        if let Some(pixel) = self.pixels.get_mut(index) {

                            let r2_inv = 1.0 / (self.brush_size.powi(2));

                            let intensity = 
                                (0.1 + self.brush_intensity)
                                * (1.0 / (self.brush_smoothness * p_w_d.d2 * r2_inv))
                                * (1.0 - p_w_d.d2 * r2_inv);

                            let pix_value = Color32::from_rgba_premultiplied(
                                self.paint_color.r(),
                                self.paint_color.g(),
                                self.paint_color.b(),
                                (intensity * 255.0) as u8,
                            );
                            *pixel = pixel.clone().blend(pix_value);
                        }
                    }
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }
    
    pub fn draw_line(&mut self, start: Vec2, end: Vec2) -> Result<(), CanvasError> {
        if let Err(e) = self.check_point_bounds(start) {
            return Err(e);
        }
        if let Err(e) = self.check_point_bounds(end) {
            return Err(e);
        }

        let line_length = (end - start).length();
        
        if line_length < 0.001 {
            self.draw_point(start);
            return Ok(());
        }

        let grad = vec2((end.x - start.x) / line_length, (end.y - start.y) / line_length);

        let mut pos = start.clone();

        pos += grad;
        loop {
            self.draw_point(pos);

            if (pos.x - start.x).abs() >= (line_length * grad.x).abs() {
                return Ok(());
            }
            pos += grad;
        }
    }

    fn check_point_bounds(&self, pt: Vec2) -> Result<(), CanvasError> {
        if pt.x < 0.0 || pt.y < 0.0 || pt.x > self.size[0] as f32 || pt.y > self.size[1] as f32 {
            return Err(CanvasError::OutOfBoundsError);
        }
        
        Ok(())
    }
}

type Loc = [i32; 2];

#[derive(Debug)]
struct PxWithD {
    px: Loc, // pixel location
    d2: f32, // distance to brush/line squared
}


#[derive(Debug)]
struct PointPxIter {
    point: Vec2,
    origin_loc: Loc,
    loc: Option<Loc>,
    radius: f32,
    radius2: f32,
}

impl PointPxIter {
    fn new(radius: f32, point: Vec2) -> Self {
        test_radius(radius);
        
        Self {
            point,
            origin_loc: [point.x.trunc() as i32, point.y.trunc() as i32],
            loc: None,
            radius,
            radius2: radius.powi(2),
        }
    }


    fn d2_loc(&self, loc: Loc) -> f32 {
        (self.point.x - 0.5 - loc[0] as f32).powi(2) + (self.point.y - 0.5 - loc[1] as f32).powi(2)
    }

    // must have non None loc.
    fn d2(&self) -> f32 {
        self.d2_loc(self.loc.unwrap())
    }

    fn next_point(&self, loc: Option<Loc>) -> Loc {
        match loc {
            None => {
                let mut px = [(self.point.x - self.radius).trunc() as i32, (self.point.y).trunc() as i32];
                for _i in 0..3 {
                    if self.d2_loc(px) <= self.radius2 {
                        return px;
                    }
                    px = [px[0] + 1, px[1]];
                }
                panic!("Something's gone horribly wrong...");
            },
            Some(loc) if loc[1] >= self.origin_loc[1] => [loc[0], loc[1] + 1],
            Some(loc) if loc[1] <  self.origin_loc[1] => [loc[0], loc[1] - 1],
            Some(_) => panic!("This should never happen lol."),
        }
    }

    fn next_section(&self, loc: Option<Loc>) -> Loc {
        match loc {
            Some(loc) if loc[1] >= self.origin_loc[1] => [loc[0], self.origin_loc[1] - 1],
            Some(loc) if loc[1] < self.origin_loc[1] => [loc[0] + 1, self.origin_loc[1]],
            None => self.next_point(None),
            Some(_) => panic!("This should never happen lol"),
        }
    }
}

impl Iterator for PointPxIter {
    type Item = PxWithD;

    fn next(&mut self) -> Option<Self::Item> {
        self.loc = Some(self.next_point(self.loc));
        for i in 0..3 {
            let d2 = self.d2();
            if d2 < self.radius2 {
                return Some(PxWithD {px: self.loc.unwrap(), d2});
            }
            self.loc = Some(self.next_section(self.loc))
        }


        None
    }
}

fn test_radius(radius: f32) {
    if radius <= 0.0 {
        panic!("Invalid Radius!");
    }
}
