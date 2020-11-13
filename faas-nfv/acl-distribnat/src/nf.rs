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

pub fn nat<T: 'static + Batch<Header = NullHeader>>(
    parent: T,
    nat_ip: &Ipv4Addr,
) -> CompositionBatch {
    let ip = u32::from(*nat_ip);
    let mut port_hash = HashMap::<Flow, Flow, FnvHash>::with_capacity_and_hasher(65536, Default::default());
    let mut flow_vec: Vec<FlowUsed> = (MIN_PORT..65535).map(|_| Default::default()).collect();
    let mut next_port = 1024;
    const MIN_PORT: u16 = 1024;
    const MAX_PORT: u16 = 65535;
    let pipeline = parent.parse::<MacHeader>().transform(box move |pkt| {
        // let hdr = pkt.get_mut_header();
        let payload = pkt.get_mut_payload();
        if let Some(flow) = ipv4_extract_flow(payload) {
            let found = match port_hash.get(&flow) {
                Some(s) => {
                    s.ipv4_stamp_flow(payload);
                    true
                }
                None => false,
            };
            if !found {
                if next_port < MAX_PORT {
                    let assigned_port = next_port; //FIXME.
                    next_port += 1;
                    flow_vec[assigned_port as usize].flow = flow;
                    flow_vec[assigned_port as usize].used = true;
                    let mut outgoing_flow = flow.clone();
                    outgoing_flow.src_ip = ip;
                    outgoing_flow.src_port = assigned_port;
                    let rev_flow = outgoing_flow.reverse_flow();

                    port_hash.insert(flow, outgoing_flow);
                    port_hash.insert(rev_flow, flow.reverse_flow());

                    outgoing_flow.ipv4_stamp_flow(payload);
                }
            }
        }
    });
    pipeline.compose()
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

pub fn acl_match<T: 'static + Batch<Header = NullHeader>>(parent: T, acls: Vec<Acl>) -> CompositionBatch {
    let mut flow_cache = HashSet::<Flow, FnvHash>::with_hasher(Default::default());
    // take delay for URLFilter from controller/profile.go
    let url_filter_delay: u64 = 6900;
    let chacha_delay: u64 = 6800;
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
	.parse::<TcpHeader>()
        .transform(box move |_p| {
	    delay_loop(url_filter_delay);
	    delay_loop(chacha_delay);  // TODO: implement chacha
        })
	.compose()
}

pub fn acl_nat<T: 'static + Batch<Header = NullHeader>>(parent: T, acls: Vec<Acl>) -> CompositionBatch {
    let mut pipeline = acl_match(parent, acls);
    pipeline = nat(pipeline, &Ipv4Addr::new(10, 0, 0, 1));
    pipeline.compose()
}
