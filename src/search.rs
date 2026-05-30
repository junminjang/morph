use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::canvas::{Canvas, render};
use crate::dsl::{
    PROGRAM_LEN, Program, active_len, random_command, random_program, tune_parameter,
};
use crate::fitness::jaccard;
use crate::interpreter::{TurtleState, execute};

const INITIAL_ACTIVE_COMMANDS: usize = 4;
const T0: f64 = 0.1;
const TEMP_END_RATIO: f64 = 1e-3;
const LOG_EVERY: usize = 10_000;
type ProgressCallback = Arc<dyn Fn(usize, &Program, f64) + Send + Sync>;

#[derive(Clone, Copy)]
pub struct SearchConfig {
    pub time_budget_secs: f64,
    pub chains: usize,
    pub start_x: f64,
    pub start_y: f64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            time_budget_secs: 60.0,
            chains: 1,
            start_x: 0.0,
            start_y: 0.0,
        }
    }
}

const LOCAL_MOVE_PROB: f64 = 0.7;

fn mutate<R: Rng>(prog: &Program, rng: &mut R, temp: f64) -> Program {
    debug_assert_eq!(prog.len(), PROGRAM_LEN);
    let mut p = prog.clone();
    let idx = rng.gen_range(0..PROGRAM_LEN);
    // local refinement
    if rng.gen_bool(LOCAL_MOVE_PROB) {
        let frac = (temp / T0).clamp(0.0, 1.0);
        let max_delta = (1.0 + frac * 31.0).round() as i32; // 1..=32
        p[idx] = tune_parameter(&p[idx], rng, max_delta);
    } else {
        p[idx] = random_command(rng);
    }
    p
}

fn temperature(elapsed: f64, budget: f64) -> f64 {
    let t_min = (T0 * TEMP_END_RATIO).max(1e-7);
    let cooling_rate = (t_min / T0).ln() / budget;
    T0 * (cooling_rate * elapsed).exp()
}

fn evaluate(prog: &Program, target: &Canvas, cfg: &SearchConfig) -> f64 {
    let mut state = TurtleState::new(cfg.start_x, cfg.start_y);
    let segments = execute(prog, &mut state);
    jaccard(&render(&segments), target)
}

fn synthesize_chain(
    target: &Canvas,
    cfg: &SearchConfig,
    rng: &mut impl Rng,
    chain_id: usize,
    chains: usize,
    progress: Option<&ProgressCallback>,
) -> (Program, f64, usize) {
    let start = Instant::now();
    let budget = cfg.time_budget_secs;
    let prefix = if chains > 1 {
        format!("[chain {}/{}] ", chain_id + 1, chains)
    } else {
        String::new()
    };

    let t_min = (T0 * TEMP_END_RATIO).max(1e-7);
    println!("{prefix}T0 = {:.6}  T_min = {:.8}", T0, t_min);

    let mut best: Program = random_program(rng, INITIAL_ACTIVE_COMMANDS);
    let mut best_score = evaluate(&best, target, cfg);
    if let Some(progress) = progress {
        progress(chain_id, &best, best_score);
    }
    let mut current = best.clone();
    let mut current_score = best_score;
    let mut iter = 0usize;
    let mut accepted_since_log = 0usize;

    loop {
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed >= budget {
            break;
        }

        let temp = temperature(elapsed, budget);
        let candidate = mutate(&current, rng, temp);
        let candidate_score = evaluate(&candidate, target, cfg);

        let log_a = (candidate_score - current_score) / temp;
        if log_a >= 0.0 || rng.gen_range(0.0f64..1.0).ln() < log_a {
            current = candidate;
            current_score = candidate_score;
            accepted_since_log += 1;
        }

        if current_score > best_score {
            best = current.clone();
            best_score = current_score;
            if let Some(progress) = progress {
                progress(chain_id, &best, best_score);
            }
        }

        iter += 1;

        if iter % LOG_EVERY == 0 {
            let remaining = budget - start.elapsed().as_secs_f64();
            println!(
                "{prefix}iter={:>7}  T={:.6}  score={:.4}  best={:.4}  active={:>3}/{}  acc={:.2}  remaining={:.1}s",
                iter,
                temp,
                current_score,
                best_score,
                active_len(&best),
                PROGRAM_LEN,
                accepted_since_log as f64 / LOG_EVERY as f64,
                remaining
            );
            accepted_since_log = 0;
        }

        if best_score >= 0.99 {
            println!("{prefix}Converged at iter={}", iter);
            break;
        }
    }

    println!(
        "\n{prefix}Done - {} iters in {:.1}s",
        iter,
        start.elapsed().as_secs_f64()
    );
    (best, best_score, iter)
}

fn synthesize_inner(
    target: &Canvas,
    cfg: &SearchConfig,
    seed: u64,
    progress: Option<ProgressCallback>,
) -> (Program, f64) {
    let chains = cfg.chains.max(1);
    if chains == 1 {
        let mut rng = StdRng::seed_from_u64(seed);
        let (best, best_score, _) =
            synthesize_chain(target, cfg, &mut rng, 0, 1, progress.as_ref());
        return (best, best_score);
    }

    println!("Running {} independent chains", chains);

    let target = Arc::new(target.clone());
    let mut handles = Vec::with_capacity(chains);
    for chain_id in 0..chains {
        let target = Arc::clone(&target);
        let cfg = *cfg;
        let chain_seed = seed.wrapping_add(chain_id as u64);
        let progress = progress.clone();
        handles.push(thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(chain_seed);
            synthesize_chain(
                target.as_ref(),
                &cfg,
                &mut rng,
                chain_id,
                chains,
                progress.as_ref(),
            )
        }));
    }

    let mut best: Option<(Program, f64, usize, usize)> = None;
    for (chain_id, handle) in handles.into_iter().enumerate() {
        let (program, score, iter) = handle.join().expect("search chain panicked");
        if best
            .as_ref()
            .map_or(true, |(_, best_score, _, _)| score > *best_score)
        {
            best = Some((program, score, iter, chain_id));
        }
    }

    let (program, score, iter, chain_id) = best.expect("at least one chain must run");
    println!(
        "Selected chain {} with best={:.4} after {} iters",
        chain_id + 1,
        score,
        iter
    );
    (program, score)
}

pub fn synthesize(target: &Canvas, cfg: &SearchConfig, seed: u64) -> (Program, f64) {
    synthesize_inner(target, cfg, seed, None)
}

pub fn synthesize_with_checkpoints<F>(
    target: &Canvas,
    cfg: &SearchConfig,
    seed: u64,
    checkpoints: Vec<f64>,
    on_checkpoint: F,
) -> (Program, f64)
where
    F: Fn(f64, &Program, f64) + Send + Sync + 'static,
{
    if checkpoints.is_empty() {
        return synthesize(target, cfg, seed);
    }

    let best_seen = Arc::new(Mutex::new(None::<(Program, f64)>));
    let progress_best = Arc::clone(&best_seen);
    let progress: ProgressCallback = Arc::new(move |_chain_id, program, score| {
        let mut best = progress_best
            .lock()
            .expect("checkpoint progress lock poisoned");
        if best
            .as_ref()
            .map_or(true, |(_, best_score)| score > *best_score)
        {
            *best = Some((program.clone(), score));
        }
    });

    let started_at = Instant::now();
    let done = Arc::new(AtomicBool::new(false));
    let watcher_done = Arc::clone(&done);
    let watcher_best = Arc::clone(&best_seen);
    let on_checkpoint: Arc<dyn Fn(f64, &Program, f64) + Send + Sync> = Arc::new(on_checkpoint);

    let watcher = thread::spawn(move || {
        for checkpoint in checkpoints {
            while started_at.elapsed().as_secs_f64() < checkpoint
                && !watcher_done.load(Ordering::Relaxed)
            {
                let elapsed = started_at.elapsed().as_secs_f64();
                let sleep_secs = (checkpoint - elapsed).clamp(0.05, 1.0);
                thread::sleep(Duration::from_secs_f64(sleep_secs));
            }

            if started_at.elapsed().as_secs_f64() < checkpoint {
                break;
            }

            let snapshot = watcher_best
                .lock()
                .expect("checkpoint snapshot lock poisoned")
                .clone();
            if let Some((program, score)) = snapshot {
                on_checkpoint(checkpoint, &program, score);
            }
        }
    });

    let result = synthesize_inner(target, cfg, seed, Some(progress));
    done.store(true, Ordering::Relaxed);
    watcher.join().expect("checkpoint watcher thread panicked");
    result
}
