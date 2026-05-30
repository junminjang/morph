mod canvas;
mod dsl;
mod fitness;
mod interpreter;
mod search;
mod target;

use dsl::active_len;

const SEED: u64 = 42;

fn parse_checkpoint_secs() -> Vec<f64> {
    let Ok(raw) = std::env::var("MORPH_CHECKPOINT_SECS") else {
        return Vec::new();
    };

    let mut checkpoints: Vec<f64> = raw
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter_map(|part| part.parse::<f64>().ok())
        .filter(|secs| secs.is_finite() && *secs > 0.0)
        .collect();
    checkpoints.sort_by(|a, b| a.total_cmp(b));
    checkpoints.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
    checkpoints
}

fn checkpoint_label(secs: f64) -> String {
    if secs.fract().abs() < f64::EPSILON {
        format!("{secs:.0}")
    } else {
        format!("{secs:.1}").replace('.', "p")
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let image_path = args.get(1).map(|s| s.as_str());
    let time_secs: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(60.0);
    let chains: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);

    let mut cfg = search::SearchConfig::default();
    cfg.time_budget_secs = time_secs;
    cfg.chains = chains.max(1);

    let stem = image_path
        .and_then(|p| std::path::Path::new(p).file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();
    let target_out = format!("{}_target.png", stem);
    let result_out = format!("{}_result.png", stem);

    let target_canvas: Vec<bool> = match image_path {
        Some(path) => {
            println!("Loading: {}", path);
            let canvas = canvas::load_png(path);
            canvas::save_png(&canvas, &target_out);
            canvas
        }
        None => {
            panic!("No image given")
        }
    };

    println!("Target saved → {}", target_out);
    println!(
        "Running for {:.0}s with {} chain(s) ...\n",
        cfg.time_budget_secs, cfg.chains
    );

    let checkpoint_secs = parse_checkpoint_secs();
    let (synthesized, fitness) = if checkpoint_secs.is_empty() {
        search::synthesize(&target_canvas, &cfg, SEED)
    } else {
        let result_stem = stem.clone();
        let start_x = cfg.start_x;
        let start_y = cfg.start_y;
        println!("Saving checkpoints at {:?}s", checkpoint_secs);
        search::synthesize_with_checkpoints(
            &target_canvas,
            &cfg,
            SEED,
            checkpoint_secs,
            move |secs, program, score| {
                let label = checkpoint_label(secs);
                let result_out = format!("{}_result_{}s.png", result_stem, label);
                let result_canvas = target::render_program(program, start_x, start_y);
                canvas::save_png(&result_canvas, &result_out);
                println!(
                    "Checkpoint {}s best={:.4} saved → {}",
                    label, score, result_out
                );
            },
        )
    };

    println!("\nFinal fitness: {:.4}", fitness);
    println!(
        "Program ({}/{} active cmds): {:?}",
        active_len(&synthesized),
        synthesized.len(),
        synthesized
    );

    let result_canvas = target::render_program(&synthesized, cfg.start_x, cfg.start_y);
    canvas::save_png(&result_canvas, &result_out);
    println!("Result saved → {}", result_out);
}
