use super::formatter::format_bytes_count;
use super::formatter::format_bytes_rate_count;

use std::io;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

pub struct State {
    progress: Progress,
    throughput: Throughput,
    fetch_done: bool,
    last_length: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            progress: Progress::new(Instant::now()),
            throughput: Throughput::default(),
            fetch_done: false,
            last_length: 0,
        }
    }
}

pub fn report_clone(state: &mut State, raw: git2::Progress<'_>) {
    let is_finished =
        raw.received_objects() == raw.total_objects() && raw.indexed_deltas() == raw.total_deltas();

    let now = Instant::now();
    if now.duration_since(state.progress.ts) < Duration::from_millis(100) && !is_finished {
        return;
    }

    let progress = Progress::new_with_stats(now, raw);

    if progress.total_objects == 0 {
        return;
    }

    let percent = (100 * progress.received_objects) / progress.total_objects;
    let received = progress.received_objects;
    let total = progress.total_objects;

    let throughput = Throughput {
        bytes: progress.bytes - state.progress.bytes,
        micros: now.duration_since(state.progress.ts).as_micros(),
    };

    let bytes = format_bytes_count(progress.bytes);
    let speed = format_bytes_rate_count(throughput.speed(&state.throughput));

    let mut output = if state.fetch_done {
        if progress.total_deltas == 0 {
            return;
        }

        let percent = (100 * progress.indexed_deltas) / progress.total_deltas;
        let indexed_deltas = progress.indexed_deltas;
        let total_deltas = progress.total_deltas;

        let end = if indexed_deltas == total_deltas {
            ", done.\n"
        } else {
            "\r"
        };

        format!("Resolving deltas: {percent:3}% ({indexed_deltas}/{total_deltas}){end}")
    } else {
        state.fetch_done = received == total;

        let end = if state.fetch_done { ", done.\n" } else { "\r" };

        format!("Receiving objects: {percent:3}% ({received}/{total}), {bytes} | {speed}{end}")
    };

    let current_length = output.len();

    if current_length < state.last_length {
        output.insert_str(
            current_length - 1,
            " ".repeat(state.last_length - current_length + 1).as_str(),
        );
    }

    eprint!("{output}");

    state.progress = progress;
    state.throughput = throughput;
    state.last_length = current_length;

    io::stderr().flush().unwrap();
}

#[derive(Default)]
struct Throughput {
    bytes: usize,
    micros: u128,
}

impl Throughput {
    pub fn speed(&self, other: &Throughput) -> usize {
        (self.bytes + other.bytes) / (self.micros + other.micros) as usize * 1000000usize
    }
}

#[derive(Copy, Clone)]
struct Progress {
    ts: Instant,

    bytes: usize,

    received_objects: usize,
    total_objects: usize,

    indexed_deltas: usize,
    total_deltas: usize,
}

impl Progress {
    pub fn new(ts: Instant) -> Self {
        Self {
            ts,
            bytes: 0,
            received_objects: 0,
            total_objects: 0,
            indexed_deltas: 0,
            total_deltas: 0,
        }
    }

    pub fn new_with_stats(ts: Instant, raw: git2::Progress<'_>) -> Self {
        Self {
            ts,
            bytes: raw.received_bytes(),
            received_objects: raw.received_objects(),
            total_objects: raw.total_objects(),
            indexed_deltas: raw.indexed_deltas(),
            total_deltas: raw.total_deltas(),
        }
    }
}
