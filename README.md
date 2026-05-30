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
| <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/a7a42a2f-cadd-492f-b682-f79aac064964" /> | <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/d2f62f10-d835-4b1f-938f-3d21ffadac8d" /> | **0.976** |
| <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/bd71c257-3583-4cd3-bda3-bc5f62b4e9a9" /> | <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/6e701b29-4348-40f1-9cb1-9ae680d4e9ff" /> | **0.956** |
| <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/20946749-d78a-4318-9b4c-8e4c2ee4d884" /> | <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/b70a7f03-78ab-4556-8c9d-db72391fd763" /> | **0.974** |
| <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/d73c4822-6e6b-46c6-ac6e-d3686d626f2c" /> | <img width="256" height="256" alt="image" src="https://github.com/user-attachments/assets/4171ffc6-f773-4638-8e85-b7b3c8501a91" /> | **0.983** |
