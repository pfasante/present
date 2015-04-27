extern crate linc;
use linc::present::*;

fn main() {
    let lat = present_sbox().walsh_transform();
    println!("{}", lat);
}

