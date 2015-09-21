extern crate linc;
use linc::present::*;
use linc::*;

fn main() {
    let lat = walsh_transform::<u64, PresentSbox>();
    println!("PRESENT sbox {}", lat);

    let biased_masks = biased_one_bit(&lat);
    println!("biased masks {:?}", biased_masks);

    let rounds = 31;
    let number_trails = number_one_bit_trails::<u64, PresentSbox, PresentPermutation>(rounds);
    println!("computing trails for {} rounds: {:?}", rounds, number_trails);
}

