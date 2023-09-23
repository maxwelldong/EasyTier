use dashmap::DashSet;
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::common::global_ctx::ArcGlobalCtx;

pub mod icmp_proxy;
pub mod tcp_proxy;

#[derive(Debug)]
struct CidrSet {
    global_ctx: ArcGlobalCtx,
    cidr_set: Arc<DashSet<cidr::IpCidr>>,
    tasks: JoinSet<()>,
}

impl CidrSet {
    pub fn new(global_ctx: ArcGlobalCtx) -> Self {
        let mut ret = Self {
            global_ctx,
            cidr_set: Arc::new(DashSet::new()),
            tasks: JoinSet::new(),
        };
        ret.run_cidr_updater();
        ret
    }

    fn run_cidr_updater(&mut self) {
        let global_ctx = self.global_ctx.clone();
        let cidr_set = self.cidr_set.clone();
        self.tasks.spawn(async move {
            let mut last_cidrs = vec![];
            loop {
                let cidrs = global_ctx.get_proxy_cidrs();
                if cidrs != last_cidrs {
                    last_cidrs = cidrs.clone();
                    cidr_set.clear();
                    for cidr in cidrs.iter() {
                        cidr_set.insert(cidr.clone());
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    pub fn contains_v4(&self, ip: std::net::Ipv4Addr) -> bool {
        let ip = ip.into();
        return self.cidr_set.iter().any(|cidr| cidr.contains(&ip));
    }
}
