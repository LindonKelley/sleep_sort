use std::cmp::Ordering;
use std::sync::{Arc, Barrier, mpsc};
use std::thread::{sleep, spawn};
use std::time::Duration;
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    const SIZE: usize = 128;
    let mut numbers: Vec<f64> = Vec::with_capacity(SIZE);
    for _ in 0..SIZE {
        numbers.push(rng.gen());
    }
    let numbers_baseline = {
        let mut numbers = numbers.clone();
        numbers.sort_by(|a, b| {
            if let Some(ordering) = a.partial_cmp(b) {
                ordering
            } else {
                Ordering::Equal
            }
        });
        numbers
    };
    // this does pass sometimes
    sleep_sort(&mut numbers, 2.0);
    if numbers == numbers_baseline {
        println!("PASS");
    } else {
        println!("FAIL");
    }
}

/// sorting algorithm that spawns one thread for each item in the provided vector, then each thread
/// will be given an item, and sleep for as long as that item dictates (through the [Sleep] trait)
///
/// higher values for `scale` will result in more accurate results (due to sleep timings being
/// rather inexact), but will make the entire sort take `scale` times as long
fn sleep_sort<T: 'static +  Sleep + Send>(to_sort: &mut Vec<T>, scale: f64) {
    let barrier = Arc::new(Barrier::new(to_sort.len()));
    let (tx, rx) = mpsc::channel();
    for _ in 0..to_sort.len() {
        let barrier = barrier.clone();
        // we're only popping to .len(), so this is safe
        let item = unsafe { to_sort.pop().unwrap_unchecked() };
        let tx = tx.clone();
        spawn(move || {
            barrier.wait();
            let duration = Duration::from_secs_f64(item.get_time() * scale);
            sleep(duration);
            tx.send(item).expect("failed to send");
        });
    }
    drop(tx);
    while let Ok(item) = rx.recv() {
        to_sort.push(item);
    }
}

/// trait to specify how long an item should be held for in a sleep sort
trait Sleep {
    /// returns how long an item should be held for in a sleep sort, in seconds
    /// items with a larger return value will be inserted later in the resulting collection
    fn get_time(&self) -> f64;
}

impl Sleep for f64 {
    fn get_time(&self) -> f64 {
        *self
    }
}
