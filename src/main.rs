extern crate linc;
use linc::present::*;
use linc::*;

fn main() {
    let lat = walsh_transform::<u64, PresentSbox>();
    println!("PRESENT sbox {}", lat);

    let biased_masks = biased_one_bit(&lat);
    println!("biased masks {:?}", biased_masks);

    for rounds in 1..10 { //32 {
        let trail_mat = count_trails::<u64, PresentSbox, PresentPermutation>(rounds);
        println!("trails for {:2} rounds: {:12?}", rounds, max_entry(trail_mat));
    }

}

