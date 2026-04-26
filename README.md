# Final_Project

## Overview

This project is a concurrent task dispatcher simulation written in Rust. It creates a stream of CPU and IO tasks, places them into a manager queue, and uses a fixed-size worker pool to execute tasks concurrently.

The project compares two scheduling policies:

* FIFO scheduling
* Optimized scheduling using shortest-task selection

## Build Instructions

```bash
cargo build
```

For optimized build:

```bash
cargo build --release
```

## Run Instructions

Run FIFO:

```bash
cargo run --release -- fifo
```

Run optimized:

```bash
cargo run --release -- optimized
```

## Example Commands

```bash
cargo run --release -- fifo
cargo run --release -- optimized
```

## Design Summary

The system has three main parts:

1. Task generator
   Creates tasks randomly using a fixed seed.

2. Manager queue
   Stores tasks before execution using `Arc<Mutex<VecDeque<Task>>>`.

3. Worker pool
   Multiple worker threads take tasks, simulate execution, and update metrics.

## Scheduling Policies

### FIFO

Tasks are executed in arrival order.

### Optimized

Tasks with shorter duration are executed first, improving average wait and turnaround time.

## Metrics Collected

* total tasks completed
* total runtime (makespan)
* average wait time
* average turnaround time
* max wait time
* average CPU usage

## Experiment Output

Results are stored in:

* `results/fifo_output.txt`
* `results/optimized_output.txt`

## Tool Use Disclosure

Tools used:

* Rust / Cargo
* GitHub Codespaces
* ChatGPT

Accepted advice:

* comparing FIFO vs optimized scheduling

Rejected/fixed advice:

* initially included `target/` folder, later removed using `.gitignore`
