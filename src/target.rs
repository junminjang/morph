use crate::canvas::{Canvas, render};
use crate::dsl::Program;
use crate::interpreter::{TurtleState, execute};

pub fn render_program(prog: &Program, start_x: f64, start_y: f64) -> Canvas {
    let mut state = TurtleState::new(start_x, start_y);
    let segments = execute(prog, &mut state);
    render(&segments)
}
