use crate::canvas::{HEIGHT, WIDTH};
use crate::dsl::{Command, Program};

pub const MAX_PEN_WIDTH: f64 = 16.0;
pub const MAX_FORWARD: f64 = 64.0;

pub struct TurtleState {
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub pen_width: f64,
    pub pen_black: bool,
}

impl TurtleState {
    pub fn new(x: f64, y: f64) -> Self {
        TurtleState {
            x,
            y,
            angle: 0.0,
            pen_width: 4.0,
            pen_black: true,
        }
    }
}

pub type Segment = (f64, f64, f64, f64, f64, bool);

pub fn execute(program: &Program, state: &mut TurtleState) -> Vec<Segment> {
    let mut segments = Vec::new();
    for cmd in program {
        match cmd {
            Command::NoOp(_, _) => {}
            Command::Forward(p, _) => {
                let dist = 1.0 + *p as f64 * (MAX_FORWARD - 1.0) / 255.0; // 1..=MAX_FORWARD
                let rad = state.angle.to_radians();
                let nx = state.x + dist * rad.cos();
                let ny = state.y + dist * rad.sin();
                segments.push((state.x, state.y, nx, ny, state.pen_width, state.pen_black));
                state.x = nx;
                state.y = ny;
            }
            Command::Turn(p, _) => {
                state.angle = (state.angle + *p as f64 * 360.0 / 256.0).rem_euclid(360.0);
            }
            Command::SetAngle(p, _) => {
                state.angle = *p as f64 * 360.0 / 256.0;
            }
            Command::MoveTo(x, y) => {
                state.x = *x as f64 * (WIDTH as f64 - 1.0) / 255.0;
                state.y = *y as f64 * (HEIGHT as f64 - 1.0) / 255.0;
            }
            Command::SetWidth(p, _) => {
                state.pen_width = 1.0 + *p as f64 * (MAX_PEN_WIDTH - 1.0) / 255.0; // 1..=MAX_PEN_WIDTH
            }
            Command::SetColor(p, _) => {
                state.pen_black = *p >= 128;
            }
        }
    }
    segments
}
