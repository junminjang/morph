use crate::canvas::Canvas;

// Jaccard index: |A ∩ B| / |A ∪ B|
pub fn jaccard(a: &Canvas, b: &Canvas) -> f64 {
    let mut intersection = 0usize;
    let mut union = 0usize;
    for i in 0..a.len() {
        match (a[i], b[i]) {
            (true, true) => {
                intersection += 1;
                union += 1;
            }
            (true, false) | (false, true) => union += 1,
            (false, false) => {}
        }
    }
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}
