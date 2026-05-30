use rand::Rng;

pub const PROGRAM_LEN: usize = 512;
pub const N_COMMAND_TYPES: usize = 7;

/// every command carries the same arity `(u8, u8)`
/// which makes the uniform full-random proposal SYMMETRIC: q(R→R*) = q(R*→R)
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    NoOp(u8, u8),
    Forward(u8, u8),
    Turn(u8, u8),
    SetAngle(u8, u8),
    MoveTo(u8, u8),
    SetWidth(u8, u8),
    SetColor(u8, u8),
}

pub type Program = Vec<Command>;

fn build(kind: usize, a: u8, b: u8) -> Command {
    match kind {
        0 => Command::NoOp(a, b),
        1 => Command::Forward(a, b),
        2 => Command::Turn(a, b),
        3 => Command::SetAngle(a, b),
        4 => Command::MoveTo(a, b),
        5 => Command::SetWidth(a, b),
        _ => Command::SetColor(a, b),
    }
}

pub fn random_command<R: Rng>(rng: &mut R) -> Command {
    let a = rng.gen_range(0u8..=255);
    let b = rng.gen_range(0u8..=255);
    build(rng.gen_range(0..N_COMMAND_TYPES), a, b)
}

pub fn random_active_command<R: Rng>(rng: &mut R) -> Command {
    let a = rng.gen_range(0u8..=255);
    let b = rng.gen_range(0u8..=255);
    // except NoOp
    build(1 + rng.gen_range(0..(N_COMMAND_TYPES - 1)), a, b)
}

pub fn tune_parameter<R: Rng>(cmd: &Command, rng: &mut R, max_delta: i32) -> Command {
    let max_delta = max_delta.max(1).min(127); // < 128 so no offset aliases mod 256
    let mut bump = |p: u8| -> u8 {
        let delta = rng.gen_range(-max_delta..=max_delta);
        (p as i32 + delta).rem_euclid(256) as u8
    };
    match *cmd {
        Command::NoOp(a, b) => Command::NoOp(bump(a), b),
        Command::Forward(a, b) => Command::Forward(bump(a), b),
        Command::Turn(a, b) => Command::Turn(bump(a), b),
        Command::SetAngle(a, b) => Command::SetAngle(bump(a), b),
        Command::MoveTo(a, b) => Command::MoveTo(bump(a), bump(b)),
        Command::SetWidth(a, b) => Command::SetWidth(bump(a), b),
        Command::SetColor(a, b) => Command::SetColor(bump(a), b),
    }
}

pub fn random_program<R: Rng>(rng: &mut R, active: usize) -> Program {
    let mut program = vec![Command::NoOp(0, 0); PROGRAM_LEN];
    for _ in 0..active.min(PROGRAM_LEN) {
        let idx = rng.gen_range(0..PROGRAM_LEN);
        program[idx] = random_active_command(rng);
    }
    program
}

pub fn active_len(prog: &Program) -> usize {
    prog.iter()
        .filter(|cmd| !matches!(cmd, Command::NoOp(_, _)))
        .count()
}
