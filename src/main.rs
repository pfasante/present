extern crate linc;
use linc::present::*;
use linc::sbox::*;

fn main() {
    let lat = present_sbox().walsh_transform();
    println!("PRESENT sbox {}", lat);
    let biased_masks = biased_one_bit(&lat);

    print!("biased masks");
    for (alpha, beta, bias) in biased_masks {
        print!(" ({}, {}, {})", alpha, beta, bias);
    }
    println!("");
}

