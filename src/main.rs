use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::VecDeque;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
enum TaskKind {
    CPU,
    IO,
}

#[derive(Clone, Debug)]
struct Task {
    id: usize,
    kind: TaskKind,
    duration_ms: u64,
    created_at: Instant,
}

#[derive(Clone, Copy, Debug)]
enum Policy {
    FIFO,
    Optimized,
}

struct Metrics {
    completed: usize,
    cpu_completed: usize,
    io_completed: usize,
    total_wait_ms: u128,
    total_turnaround_ms: u128,
    max_wait_ms: u128,
    total_worker_busy_ms: u128,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            completed: 0,
            cpu_completed: 0,
            io_completed: 0,
            total_wait_ms: 0,
            total_turnaround_ms: 0,
            max_wait_ms: 0,
            total_worker_busy_ms: 0,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let policy = if args.len() > 1 && args[1].to_lowercase() == "optimized" {
        Policy::Optimized
    } else {
        Policy::FIFO
    };

    run_simulation(policy);
}

fn run_simulation(policy: Policy) {
    let total_tasks = 500;
    let worker_count = 6;
    let start_time = Instant::now();

    let queue: Arc<Mutex<VecDeque<Task>>> = Arc::new(Mutex::new(VecDeque::new()));
    let metrics = Arc::new(Mutex::new(Metrics::new()));
    let generator_done = Arc::new(Mutex::new(false));

    let queue_for_generator = Arc::clone(&queue);
    let done_for_generator = Arc::clone(&generator_done);

    let generator = thread::spawn(move || {
        generate_tasks(queue_for_generator, done_for_generator, total_tasks);
    });

    let mut workers = Vec::new();

    for worker_id in 0..worker_count {
        let queue_for_worker = Arc::clone(&queue);
        let metrics_for_worker = Arc::clone(&metrics);
        let done_for_worker = Arc::clone(&generator_done);

        let worker = thread::spawn(move || {
            worker_loop(
                worker_id,
                queue_for_worker,
                metrics_for_worker,
                done_for_worker,
                policy,
            );
        });

        workers.push(worker);
    }

    generator.join().unwrap();

    for worker in workers {
        worker.join().unwrap();
    }

    let total_runtime_ms = start_time.elapsed().as_millis();
    let final_metrics = metrics.lock().unwrap();

    let avg_wait = final_metrics.total_wait_ms as f64 / final_metrics.completed as f64;
    let avg_turnaround =
        final_metrics.total_turnaround_ms as f64 / final_metrics.completed as f64;

    let average_cpu_usage = (final_metrics.total_worker_busy_ms as f64
        / (total_runtime_ms as f64 * worker_count as f64))
        * 100.0;

    println!("======================================");
    println!("Concurrent Task Dispatcher Results");
    println!("======================================");
    println!("Policy: {:?}", policy);
    println!("Total tasks completed: {}", final_metrics.completed);
    println!("CPU tasks completed: {}", final_metrics.cpu_completed);
    println!("IO tasks completed: {}", final_metrics.io_completed);
    println!("Total runtime / makespan: {} ms", total_runtime_ms);
    println!("Average wait time: {:.2} ms", avg_wait);
    println!("Average turnaround time: {:.2} ms", avg_turnaround);
    println!("Max wait time: {} ms", final_metrics.max_wait_ms);
    println!("Average CPU usage: {:.2}%", average_cpu_usage);
    println!("Worker count: {}", worker_count);
    println!("======================================");
}

fn generate_tasks(
    queue: Arc<Mutex<VecDeque<Task>>>,
    generator_done: Arc<Mutex<bool>>,
    total_tasks: usize,
) {
    let mut rng = StdRng::seed_from_u64(42);

    for id in 0..total_tasks {
        let kind = if rng.gen_bool(0.5) {
            TaskKind::CPU
        } else {
            TaskKind::IO
        };

        let duration_ms = match kind {
            TaskKind::CPU => rng.gen_range(30..80),
            TaskKind::IO => rng.gen_range(10..40),
        };

        thread::sleep(Duration::from_millis(rng.gen_range(1..5)));

        let task = Task {
            id,
            kind,
            duration_ms,
            created_at: Instant::now(),
        };

        let mut locked_queue = queue.lock().unwrap();
        locked_queue.push_back(task);
    }

    let mut done = generator_done.lock().unwrap();
    *done = true;
}

fn worker_loop(
    worker_id: usize,
    queue: Arc<Mutex<VecDeque<Task>>>,
    metrics: Arc<Mutex<Metrics>>,
    generator_done: Arc<Mutex<bool>>,
    policy: Policy,
) {
    loop {
        let task_option = {
            let mut locked_queue = queue.lock().unwrap();

            match policy {
                Policy::FIFO => locked_queue.pop_front(),
                Policy::Optimized => pop_optimized_task(&mut locked_queue),
            }
        };

        match task_option {
            Some(task) => {
                let wait_time_ms = task.created_at.elapsed().as_millis();

                let execution_start = Instant::now();
                thread::sleep(Duration::from_millis(task.duration_ms));
                let execution_time_ms = execution_start.elapsed().as_millis();

                let turnaround_time_ms = task.created_at.elapsed().as_millis();

                let mut locked_metrics = metrics.lock().unwrap();
                locked_metrics.completed += 1;

                match task.kind {
                    TaskKind::CPU => locked_metrics.cpu_completed += 1,
                    TaskKind::IO => locked_metrics.io_completed += 1,
                }

                locked_metrics.total_wait_ms += wait_time_ms;
                locked_metrics.total_turnaround_ms += turnaround_time_ms;
                locked_metrics.total_worker_busy_ms += execution_time_ms;

                if wait_time_ms > locked_metrics.max_wait_ms {
                    locked_metrics.max_wait_ms = wait_time_ms;
                }

                println!(
                    "Worker {} completed task {} ({:?})",
                    worker_id, task.id, task.kind
                );
            }
            None => {
                let done = *generator_done.lock().unwrap();
                let queue_empty = queue.lock().unwrap().is_empty();

                if done && queue_empty {
                    break;
                }

                thread::sleep(Duration::from_millis(2));
            }
        }
    }
}

fn pop_optimized_task(queue: &mut VecDeque<Task>) -> Option<Task> {
    if queue.is_empty() {
        return None;
    }

    let mut best_index = 0;
    let mut best_duration = queue[0].duration_ms;

    for i in 1..queue.len() {
        if queue[i].duration_ms < best_duration {
            best_duration = queue[i].duration_ms;
            best_index = i;
        }
    }

    queue.remove(best_index)
}