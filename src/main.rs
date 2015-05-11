extern crate linc;
use linc::present::*;
use linc::*;

fn main() {
    let sbox = PresentSbox::new();

    let lat = walsh_transform(&sbox);
    println!("PRESENT sbox {}", lat);

    let biased_masks = biased_one_bit(&lat);
    println!("biased masks {:?}", biased_masks);

    let rounds = 10;
    let state_matrix = number_one_bit_trails::<Present, u64, PresentCipherKey, PresentSbox, PresentPermutation>(rounds);
    println!("computing trails for {} rounds:\n{}", rounds, state_matrix);
}

