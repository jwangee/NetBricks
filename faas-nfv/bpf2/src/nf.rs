use e2d2::headers::*;
use e2d2::operators::*;
use e2d2::utils::*;
use fnv::FnvHasher;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

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
    pub fn matches(&self, flow: &Flow, _connections: &HashSet<Flow, FnvHash>) -> bool {
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
                    return !acl.drop;
                    /*
                    if !acl.drop {
                        flow_cache.insert(f);
                    }
                    return !acl.drop;
                    */
                }
            }
	    // drop packet
            return false;
        })
	.compose()
}

pub fn bpf2<T: 'static + Batch<Header = NullHeader>>(
    parent: T,
    bpf1: Vec<Acl>,
    bpf2: Vec<Acl>,
) -> CompositionBatch {
    let mut pipeline = acl_match(parent, bpf1);
    pipeline = acl_match(pipeline, bpf2);
    pipeline = macswap(pipeline);
    pipeline.compose()
}
