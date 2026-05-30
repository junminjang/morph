# morph

Synthesizes a **turtle program** that redraws a black-and-white image. MCMC search over
a tiny DSL — no neural net. The render improves the longer it runs, converging toward the target.

## How it works

1. Load the target PNG, resize to 256×256, binarize (on if luma < 128).
2. Search for the turtle program whose rendering best overlaps the target.
3. Save the binarized target and the best result.

A program is a fixed vector of 512 commands, each carrying two `u8` params:
`Forward` (stroke), `Turn`/`SetAngle` (heading), `MoveTo` (jump), `SetWidth`, `SetColor`,
`NoOp`. Running the program draws strokes onto a 256×256 1-bit canvas; fitness is the **Jaccard
index** (overlap / union) between that render and the target.

Search is **MCMC** with an exponentially cooled temperature: each step proposes a mutation of
one command (70% small parameter tweak, 30% full random replacement) and accepts it via the
Metropolis rule. Several independent chains run in parallel — best chain wins.

## Usage

```bash
cargo build --release

# morph <image> [time_secs=60] [chains=1]
target/release/morph kiwi.png 60 4
```

Writes `kiwi_target.png` (binarized target) and `kiwi_result.png` (best rendering).

Set `MORPH_CHECKPOINT_SECS="60,600,1200"` to dump best-so-far snapshots at those times.
See [scripts/](scripts/) for batch and checkpoint runners.

## Results

4 chains, 3h (10800s) budget

| Target | Result | Jaccard |
|:------:|:------:|:-------:|
| ![](kiwi_target.png)        | ![](kiwi_result.png)        | **0.976** |
| ![](psl_logo_target.png)    | ![](psl_logo_result.png)    | **0.956** |
| ![](github_logo_target.png) | ![](github_logo_result.png) | **0.974** |
| ![](yinyang_target.png)     | ![](yinyang_result.png)     | **0.983** |
