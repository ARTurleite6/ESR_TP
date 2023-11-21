use std::net::IpAddr;

#[derive(Debug)]
struct RP {
    content_servers: Vec<IpAddr>,
}

impl RP {
    fn new(bootstraper: IpAddr) -> Self {
        todo!()
    }
}
