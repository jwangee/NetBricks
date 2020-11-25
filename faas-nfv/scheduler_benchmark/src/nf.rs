use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::utils;
use std::cmp;


pub static mut TIMESTAMPS: Vec<u64> = Vec::new();



pub fn nf1<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    parent.transform(box move |_p| {
	unsafe {
    	    TIMESTAMPS.push(utils::rdtsc_unsafe());
	}
    }).compose()
}

pub fn nf2<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    parent.transform(box move |_p| {
	unsafe {
    	    TIMESTAMPS.push(utils::rdtsc_unsafe());
	}
    }).compose()
}

// Construct pipeline
pub fn test_chain<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    let mut pipeline = nf1(parent);
    pipeline = nf2(pipeline);
    pipeline.compose()
}

// Print benchmarking results
pub fn print_results() {
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
    let num_ts_diffs = ts_diffs.len();
    println!("num: {}", num_ts_diffs);
    if num_ts_diffs == 0 {
	return;
    }
    println!("avg: {:.3}", mean(&ts_diffs).unwrap_or_else(|| -1.0));
    println!("stdev: {:.3}", std_deviation(&ts_diffs).unwrap_or_else(|| -1.0));

    ts_diffs.sort();

    println!("min: {}", ts_diffs[0]);
    let percentiles = [25, 50, 75, 90, 95, 99];
    for p in &percentiles {
	let idx = (num_ts_diffs as f32 * (*p as f32) / 100.0) as usize;
	println!("p{}: {}", p, ts_diffs[cmp::min(cmp::max(0, idx-1), num_ts_diffs-1)]);
    }
    println!("max: {}", ts_diffs[num_ts_diffs-1]);
}


// based on https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html
pub fn mean(data: &Vec<u64>) -> Option<f64> {
    let sum = data.iter().sum::<u64>() as f64;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f64),
        _ => None,
    }
}

// based on https://rust-lang-nursery.github.io/rust-cookbook/science/mathematics/statistics.html
pub fn std_deviation(data: &Vec<u64>) -> Option<f64> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data.iter().map(|value| {
                let diff = data_mean - (*value as f64);

                diff * diff
            }).sum::<f64>() / count as f64;

            Some(variance.sqrt())
        },
        _ => None
    }
}
