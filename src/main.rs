extern crate linc;
use linc::present::*;
use linc::*;

fn main() {
    let sbox = PresentSbox::new();

    let lat = walsh_transform(&sbox);
    println!("PRESENT sbox {}", lat);

    let biased_masks = biased_one_bit(&lat);
    println!("biased masks {:?}", biased_masks);

    let rounds = 31;
    let number_trails = number_one_bit_trails::<Present<PresentKeySchedule>, u64, PresentCipherKey,
        PresentSbox, PresentPermutation>(rounds);
    println!("computing trails for {} rounds: {:?}", rounds, number_trails);
}

