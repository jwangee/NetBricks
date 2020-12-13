use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::utils::{Flow, Ipv4Prefix};
use fnv::FnvHasher;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use chacha::{ChaCha, KeyStream};
use std::arch::x86_64::_rdtsc;

static mut prev_ts: i64 = 0;
static mut switch_overhead: i64 = 0;

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

impl Acl {
    pub fn matches(&self, flow: &Flow, connections: &HashSet<Flow, FnvHash>) -> bool {
        if (self.src_ip.is_none() || self.src_ip.unwrap().in_range(flow.src_ip))
            && (self.dst_ip.is_none() || self.dst_ip.unwrap().in_range(flow.dst_ip))
            && (self.src_port.is_none() || flow.src_port == self.src_port.unwrap())
            && (self.dst_port.is_none() || flow.dst_port == self.dst_port.unwrap())
        {
            if let Some(established) = self.established {
                let rev_flow = flow.reverse_flow();
                (connections.contains(flow) || connections.contains(&rev_flow)) == established
            } else {
                true
            }
        } else {
            false
        }
    }
}

#[inline]
fn lat() {
    unsafe {
        asm!("nop"
             :
             :
             :
             : "volatile");
    }
}

#[inline]
fn delay_loop(delay: u64) {
    let mut d = 0;
    while d < delay {
        lat();
        d += 1;
    }
}

pub fn urlfilter<T: 'static + Batch<Header = NullHeader>>(parent: T, delay: u64) -> CompositionBatch {
    parent.transform(box move |_p| {
	delay_loop(delay);
    }).compose()
}

const CHACHA_SECRET_KEY: [u8; 32] = [0u8; 32];
const CHACHA_NONCE: [u8; 8] = [0u8; 8];
// add a 20-byte offset to keep timestamp in TCP payload intact
const CHACHA_OFFSET: usize = 20;

pub fn chacha<T: 'static + Batch<Header = NullHeader>>(parent: T) -> CompositionBatch {
    let mut stream = ChaCha::new_chacha20(&CHACHA_SECRET_KEY, &CHACHA_NONCE);
    parent
	.parse::<MacHeader>()
	.parse::<IpHeader>()
	.parse::<TcpHeader>()
	.transform(box move |p| {
	    let payload = p.get_mut_payload();
	    if payload.len() > CHACHA_OFFSET {
		stream.xor_read(&mut payload[CHACHA_OFFSET..])
		    .expect("hit end of stream far too soon");
	    }
    }).compose()
}


pub fn acl_match<T: 'static + Batch<Header = NullHeader>>(parent: T, acls: Vec<Acl>) -> CompositionBatch {
    let mut flow_cache = HashSet::<Flow, FnvHash>::with_hasher(Default::default());
    parent
        .parse::<MacHeader>()
        .transform(box move |p| {
            p.get_mut_header().swap_addresses();
        })
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

pub fn acl_urlfilter_chacha<T: 'static + Batch<Header = NullHeader>>(parent: T, acls: Vec<Acl>) -> CompositionBatch {
    // take delay for URLFilter from controller/profile.go
    let urlfilter_delay: u64 = 6900;
    let mut pipeline = acl_match(parent, acls);
    pipeline = urlfilter(pipeline, urlfilter_delay);
    pipeline = chacha(pipeline);
    pipeline.compose()
}

pub fn tag_ts() {
    unsafe {
        let now = _rdtsc();
        switch_overhead = now - prev_ts;
        prev_ts = now;
    }
}
