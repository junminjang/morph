use image::{GrayImage, ImageReader, imageops};

pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 256;

pub type Canvas = Vec<bool>;

pub fn new_canvas() -> Canvas {
    vec![false; (WIDTH * HEIGHT) as usize]
}

fn fill_disk(canvas: &mut Canvas, cx: i32, cy: i32, r: i32, black: bool) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                    canvas[(y as u32 * WIDTH + x as u32) as usize] = black;
                }
            }
        }
    }
}

fn draw_line(canvas: &mut Canvas, x1: f64, y1: f64, x2: f64, y2: f64, width: f64, black: bool) {
    let r = (width / 2.0).round().max(1.0) as i32;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let steps = dx.abs().max(dy.abs()).round() as i32;

    if steps == 0 {
        fill_disk(canvas, x1.round() as i32, y1.round() as i32, r, black);
        return;
    }

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let x = x1 + dx * t;
        let y = y1 + dy * t;
        fill_disk(canvas, x.round() as i32, y.round() as i32, r, black);
    }
}

pub fn render(segments: &[(f64, f64, f64, f64, f64, bool)]) -> Canvas {
    let mut canvas = new_canvas();
    for &(x1, y1, x2, y2, w, black) in segments {
        draw_line(&mut canvas, x1, y1, x2, y2, w, black);
    }
    canvas
}

pub fn load_png(path: &str) -> Canvas {
    let img = ImageReader::open(path)
        .unwrap_or_else(|e| panic!("cannot open '{}': {}", path, e))
        .with_guessed_format()
        .unwrap_or_else(|e| panic!("cannot guess format: {}", e))
        .decode()
        .unwrap_or_else(|e| panic!("cannot decode '{}': {}", path, e))
        .resize_exact(WIDTH, HEIGHT, imageops::FilterType::Lanczos3)
        .to_luma8();
    img.pixels().map(|p| p[0] < 128).collect()
}

pub fn save_png(canvas: &Canvas, path: &str) {
    let mut img = GrayImage::new(WIDTH, HEIGHT);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let v = if canvas[(y * WIDTH + x) as usize] {
                0u8
            } else {
                255u8
            };
            img.put_pixel(x, y, image::Luma([v]));
        }
    }
    img.save(path).expect("failed to save PNG");
}
