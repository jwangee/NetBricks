#![feature(box_syntax)]
#![feature(asm)]
#![feature(const_vec_new)]
extern crate e2d2;
extern crate getopts;
use e2d2::scheduler::*;
use e2d2::utils;
use getopts::Options;
use std::env;
use std::process;
use std::cmp;
use self::stats::*;
mod stats;

// Default number of measurements:
// Used when no -N <number> is given
const DEFAULT_NUM_MEAS: u64 = 1000;

pub static mut TIMESTAMPS: Vec<u64> = Vec::new();

fn nf1(){
    unsafe {
    	TIMESTAMPS.push(utils::rdtsc_unsafe());
    }
}

fn nf2(){
    unsafe {
    	TIMESTAMPS.push(utils::rdtsc_unsafe());
    }
}

fn main() {
    // Read CLI args
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "verbose mode (prints data)");
    opts.optopt("N", "num_meas", "Number of measurements", "NUM_MEAS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        let program = env::args().next().unwrap();
        print!("{}", opts.usage(&format!("Usage: {} [options]", program)));
        process::exit(0)
    }
    let num_measurements: u64 = matches
        .opt_str("num_meas")
        .unwrap_or_else(|| DEFAULT_NUM_MEAS.to_string())
        .parse()
        .expect("Could not parse number of measurements");

    // Init Scheduler
    let mut sched = StandaloneScheduler::new();
    sched.add_task(|| nf1()).unwrap();
    sched.add_task(|| nf2()).unwrap();

    // Run measurement
    for _ in 0..num_measurements {
	sched.execute_one();
	sched.execute_one();
    }

    // Collect results
    let mut ts_diffs: Vec<u64> = {
	unsafe {
	    TIMESTAMPS
		.chunks_exact(2)
		.map(|x| {
		    if x[0] > x[1] {
			0
		    }
		    else {
			x[1]-x[0]
		    }
		})
		.collect()
	}
    };

    // Calculate and print statistics
    if matches.opt_present("v") {
	println!("data: {:?}", ts_diffs);
    }
    let num_ts_diffs = ts_diffs.len();
    println!("avg: {:.3}", mean(&ts_diffs).unwrap_or_else(|| -1.0));
    println!("stdev: {:.3}", std_deviation(&ts_diffs).unwrap_or_else(|| -1.0));

    ts_diffs.sort();
    // if matches.opt_present("v") {
    // 	println!("sorted data: {:?}", ts_diffs);
    // }

    println!("min: {}", ts_diffs[0]);
    let percentiles = [25, 50, 75, 90, 95, 99];
    for p in &percentiles {
	let idx = (num_ts_diffs as f32 * (*p as f32) / 100.0) as usize;
	println!("p{}: {}", p, ts_diffs[cmp::min(cmp::max(0, idx-1), num_ts_diffs-1)]);
    }
    println!("max: {}", ts_diffs[num_ts_diffs-1]);
}
