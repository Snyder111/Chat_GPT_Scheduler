use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
struct Process {
    name: String,
    arrival: i32,
    burst: i32,
    remaining: i32,
    finish_time: i32,
    start_time: Option<i32>,
    has_finished: bool,
}

struct Config {
    process_count: usize,
    run_for: i32,
    algorithm: String,
    quantum: Option<i32>,
    processes: Vec<Process>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: scheduler <input file>");
        return;
    }

    let input_path = &args[1];
    if !input_path.ends_with(".in") {
        println!("Error: Input file must have .in extension.");
        return;
    }

    let config = parse_input(input_path);
    let output_path = input_path.replace(".in", ".out");
    run_simulation(config, &output_path);
}

fn parse_input(filename: &str) -> Config {
    let file = File::open(filename).expect("Error: Cannot open file.");
    let reader = BufReader::new(file);

    let mut process_count = 0;
    let mut run_for = 0;
    let mut algorithm = String::new();
    let mut quantum = None;
    let mut processes = Vec::new();

    for line in reader.lines() {
        let l = line.unwrap();
        let parts: Vec<&str> = l.split_whitespace().collect();
        if parts.is_empty() || parts[0] == "end" || parts[0].starts_with('#') { continue; }

        match parts[0] {
            "processcount" => process_count = parts[1].parse().unwrap(),
            "runfor" => run_for = parts[1].parse().unwrap(),
            "use" => algorithm = parts[1].to_string(),
            "quantum" => quantum = Some(parts[1].parse().unwrap()),
            "process" => {
                processes.push(Process {
                    name: parts[2].to_string(),
                    arrival: parts[4].parse().unwrap(),
                    burst: parts[6].parse().unwrap(),
                    remaining: parts[6].parse().unwrap(),
                    finish_time: 0,
                    start_time: None,
                    has_finished: false,
                });
            }
            _ => {}
        }
    }

    // Validation
    if algorithm == "rr" && quantum.is_none() {
        panic!("Error: Missing quantum parameter when use is 'rr'");
    }

    Config { process_count, run_for, algorithm, quantum, processes }
}

fn run_simulation(mut config: Config, output_path: &str) {
    let mut out = File::create(output_path).unwrap();
    
    writeln!(out, "{} processes", config.process_count).unwrap();
    let algo_name = match config.algorithm.as_str() {
        "fcfs" => "First Come First-Served",
        "sjf" => "preemptive Shortest Job First",
        "rr" => "Round-Robin",
        _ => "Unknown",
    };
    writeln!(out, "Using {}", algo_name).unwrap();
    if let Some(q) = config.quantum { writeln!(out, "Quantum {}", q).unwrap(); }

    let mut current_proc_idx: Option<usize> = None;
    let mut ready_queue: VecDeque<usize> = VecDeque::new();
    let mut rr_time_slice = 0;

    for t in 0..config.run_for {
        // 1. Handle Arrivals
        for i in 0..config.processes.len() {
            if config.processes[i].arrival == t {
                writeln!(out, "Time {}: {} arrived", t, config.processes[i].name).unwrap();
                ready_queue.push_back(i);
            }
        }

        // 2. Selection Logic
        if config.algorithm == "sjf" {
            // Preemption: Check if a shorter job arrived
            if let Some(best_idx) = ready_queue.iter().min_by_key(|&&i| config.processes[i].remaining).cloned() {
                if current_proc_idx.is_none() || config.processes[best_idx].remaining < config.processes[current_proc_idx.unwrap()].remaining {
                    if let Some(old) = current_proc_idx { ready_queue.push_back(old); }
                    current_proc_idx = Some(best_idx);
                    ready_queue.retain(|&x| x != best_idx);
                    start_process(&mut config.processes[current_proc_idx.unwrap()], t, &mut out);
                }
            }
        } else if config.algorithm == "rr" {
            if current_proc_idx.is_none() || rr_time_slice >= config.quantum.unwrap() {
                if let Some(old) = current_proc_idx { ready_queue.push_back(old); }
                current_proc_idx = ready_queue.pop_front();
                rr_time_slice = 0;
                if let Some(idx) = current_proc_idx {
                    start_process(&mut config.processes[idx], t, &mut out);
                }
            }
        } else if config.algorithm == "fcfs" && current_proc_idx.is_none() {
            current_proc_idx = ready_queue.pop_front();
            if let Some(idx) = current_proc_idx {
                start_process(&mut config.processes[idx], t, &mut out);
            }
        }

        // 3. Execution
        if let Some(idx) = current_proc_idx {
            let p = &mut config.processes[idx];
            p.remaining -= 1;
            rr_time_slice += 1;

            if p.remaining == 0 {
                p.has_finished = true;
                p.finish_time = t + 1;
                writeln!(out, "Time {}: {} finished", t + 1, p.name).unwrap();
                current_proc_idx = None;
                rr_time_slice = 0;
            }
        } else {
            writeln!(out, "Time {}: Idle", t).unwrap();
        }
    }

    writeln!(out, "Finished at time {}\n", config.run_for).unwrap();

    // 4. Reporting
    for p in &config.processes {
        if p.has_finished {
            let turnaround = p.finish_time - p.arrival;
            let wait = turnaround - p.burst;
            let response = p.start_time.unwrap() - p.arrival;
            writeln!(out, "{} wait {} turnaround {} response {}", p.name, wait, turnaround, response).unwrap();
        } else {
            writeln!(out, "{} did not finish", p.name).unwrap();
        }
    }
}

fn start_process(p: &mut Process, t: i32, out: &mut File) {
    if p.start_time.is_none() { p.start_time = Some(t); }
    writeln!(out, "Time {}: {} selected (burst {})", t, p.name, p.remaining).unwrap();
}