use clap::Parser;
use esr_lib::server::rp::{RPArgs, RP};


fn main() {

    let args = RPArgs::parse();
    
    let rp = RP::new(args);
    
    rp.run();
}