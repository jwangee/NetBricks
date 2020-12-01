use redis::Commands;
use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::utils::*;
use fnv::FnvHasher;
use std::collections::{HashSet,HashMap};
use std::convert::From;
use std::hash::BuildHasherDefault;
use std::net::Ipv4Addr;

type FnvHash = BuildHasherDefault<FnvHasher>;

#[derive(Clone)]
pub struct Acl {
    pub src_ip: Option<Ipv4Prefix>,
    pub dst_ip: Option<Ipv4Prefix>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub established: Option<bool>,
    // Related not done
    pub drop: bool,
}

#[derive(Clone, Default)]
struct Unit;
#[derive(Clone, Copy, Default)]
struct FlowUsed {
    pub flow: Flow,
    pub time: u64,
    pub used: bool,
}

// https://docs.rs/redis/0.8.0/redis/index.html
// https://github.com/jwangee/FaaS-Flow/blob/master/network_functions/test_dpdk/nf_container/modules/distributed_nat.cc

const REDIS_KEY: &'static str = "NAT";

fn upload_rule(con: &redis::Connection, field: &str, value: &str) -> redis::RedisResult<()> {
    //     std::string field ("");
    //   field += ToIpv4Address(endpoint.addr) + ":" +
    //       std::to_string(endpoint.port.value());

    // std::string value ("");
    // value += ToIpv4Address(entry.endpoint.addr) + ":" +
    //          std::to_string(entry.endpoint.port.value());
    let _: () = try!(con.hset(REDIS_KEY, field, value));
    Ok(())
}

fn remove_rule(con: &redis::Connection, field: &str) -> redis::RedisResult<()> {
    let _: () = try!(con.hdel(REDIS_KEY, field));
    Ok(())
}

fn fetch_rule(con: &redis::Connection, field: &str) -> bool {
    match con.hget(REDIS_KEY, field) {
	Err(_) => return false,
	Ok(v) => {
	    let ip: u32 = v;
	    // TODO: parse ip addr, add to nat
	    return true
	}
    }
}

// fn rules_sync_global(con: &redis::Connection) -> redis::RedisResult<()> {
//     let map: HashMap<String, String> = try!(con.hgetall(REDIS_KEY));
//     // TODO: iterate result, parse entries, add them to nat
//     Ok(())
// }

impl Acl {
    pub fn matches(&self, flow: &Flow, connections: &HashSet<Flow, FnvHash>) -> bool {
        if (self.src_ip.unwrap().in_range(flow.src_ip))
            && (self.dst_ip.unwrap().in_range(flow.dst_ip))
            && (self.src_port.is_none() || flow.src_port == self.src_port.unwrap())
            && (self.dst_port.is_none() || flow.dst_port == self.dst_port.unwrap())
        {
            true
        } else {
            false
        }
    }
}

pub fn macswap<T: 'static + Batch<Header = NullHeader>>(
    parent: T,
) -> CompositionBatch {
    parent
        .parse::<MacHeader>()
        .transform(box move |p| {
            p.get_mut_header().swap_addresses();
        })
    .compose()
}

pub fn acl_match<T: 'static + Batch<Header = NullHeader>>(
    parent: T,
    acls: Vec<Acl>
) -> CompositionBatch {
    let mut flow_cache = HashSet::<Flow, FnvHash>::with_hasher(Default::default());
    parent
        .parse::<MacHeader>()
        .parse::<IpHeader>()
        .filter(box move |p| {
            let flow = p.get_header().flow();
            for acl in &acls {
                if flow.is_none() {
                    return true;
                }
                let f = flow.unwrap();
                if acl.matches(&f, &flow_cache) {
                    if !acl.drop {
                        flow_cache.insert(f);
                    }
                    return !acl.drop;
                }
            }
	    // drop packet
            return false;
        })
	.compose()
}

pub fn bpf6<T: 'static + Batch<Header = NullHeader>>(
    parent: T,
    bpf1: Vec<Acl>,
    bpf2: Vec<Acl>,
    bpf3: Vec<Acl>,
    bpf4: Vec<Acl>,
    bpf5: Vec<Acl>,
    bpf6: Vec<Acl>,
) -> CompositionBatch {
    let mut pipeline = acl_match(parent, bpf1);
    pipeline = acl_match(pipeline, bpf2);
    pipeline = acl_match(pipeline, bpf3);
    pipeline = acl_match(pipeline, bpf4);
    pipeline = acl_match(pipeline, bpf5);
    pipeline = acl_match(pipeline, bpf6);
    pipeline = macswap(pipeline);
    pipeline.compose()
}
