fn main() {
    println!("Hello, world!");
}

fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, n| acc.max(*n)))
    }
}

