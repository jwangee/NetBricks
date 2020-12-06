#![feature(box_syntax)]
#![feature(asm)]
extern crate e2d2;
extern crate fnv;
extern crate getopts;
extern crate rand;
extern crate time;
use self::nf::*;
use e2d2::allocators::CacheAligned;
use e2d2::config::*;
use e2d2::interface::*;
use e2d2::operators::*;
use e2d2::scheduler::*;
use e2d2::utils::Ipv4Prefix;
use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
mod nf;

const CONVERSION_FACTOR: f64 = 1000000000.;

fn test<S: Scheduler + Sized>(
    ports: Vec<CacheAligned<PortQueue>>,
    sched: &mut S,
) {
    for port in &ports {
        println!(
            "Receiving port {} rxq {} txq {}",
            port.port.mac_address(),
            port.rxq(),
            port.txq()
        );
    }
    // Init rules: implement setup_rules() from
    // https://github.com/jwangee/openNetVM/blob/master/examples/faas_runtime/acl.c
    let ip_range_1 = 20;
    let ip_range_2 = 10;
    let num_rules = ip_range_1 * ip_range_2 + 1;
    let mut acls: Vec<Acl> = Vec::new();
    for _ in 0..num_rules {
	acls.push(
        Acl {
        src_ip: Some(Ipv4Prefix::new(u32::from(Ipv4Addr::new(172, 12, 0, 1)), 32)),
        dst_ip: Some(Ipv4Prefix::new(0, 0)),
        src_port: None,
        dst_port: None,
        established: None,
        drop: true,
            })
    }
    acls.push(
    Acl {
            src_ip: Some(Ipv4Prefix::new(0, 0)),
            dst_ip: Some(Ipv4Prefix::new(0, 0)),
            src_port: None,
            dst_port: None,
            established: None,
            drop: false,
        }
    );

    let pipelines: Vec<_> = ports
        .iter()
        .map(|port| bpf6(ReceiveBatch::new(port.clone()), acls.clone(), acls.clone(), acls.clone(), acls.clone(), acls.clone(), acls.clone()).send(port.clone()))
        .collect();
    println!("Running {} pipelines", pipelines.len());
    for pipeline in pipelines {
        sched.add_task(pipeline).unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let opts = basic_opts();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let configuration = read_matches(&matches, &opts);

    let mut config = initialize_system(&configuration).unwrap();
    config.start_schedulers();

    config.add_pipeline_to_run(
	Arc::new(move |p, s: &mut StandaloneScheduler| test(p, s))
    );
    config.execute();

    let mut pkts_so_far = (0, 0);
    let mut last_printed = 0.;
    const MAX_PRINT_INTERVAL: f64 = 30.;
    const PRINT_DELAY: f64 = 15.;
    let sleep_delay = (PRINT_DELAY / 2.) as u64;
    let mut start = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
    let sleep_time = Duration::from_millis(sleep_delay);
    println!("0 OVERALL RX 0.00 TX 0.00 CYCLE_PER_DELAY 0 0 0");
    loop {
        thread::sleep(sleep_time); // Sleep for a bit
        let now = time::precise_time_ns() as f64 / CONVERSION_FACTOR;
        if now - start > PRINT_DELAY {
            let mut rx = 0;
            let mut tx = 0;
            for port in config.ports.values() {
                for q in 0..port.rxqs() {
                    let (rp, tp) = port.stats(q);
                    rx += rp;
                    tx += tp;
                }
            }
            let pkts = (rx, tx);
            let rx_pkts = pkts.0 - pkts_so_far.0;
            if rx_pkts > 0 || now - last_printed > MAX_PRINT_INTERVAL {
                println!(
                    "{:.2} OVERALL RX {:.2} TX {:.2}",
                    now - start,
                    rx_pkts as f64 / (now - start),
                    (pkts.1 - pkts_so_far.1) as f64 / (now - start)
                );
                last_printed = now;
                start = now;
                pkts_so_far = pkts;
            }
        }
    }
}
