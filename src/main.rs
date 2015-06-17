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
    let number_trails = number_one_bit_trails::<Present, u64, PresentCipherKey,
        PresentSbox, PresentPermutation>(rounds);
    println!("computing trails for {} rounds: {:?}", rounds, number_trails);
    let rounds = 31;
    let iterations = 1_000;
    let histo = distribution::<Present, PresentCipherKey, PresentSbox,
        PresentPermutation>(iterations, rounds);
    //println!("histogram of trails for {} rounds, {} entries:\nbias\tabsolute prob"
    //         , rounds, histo.len());
    for (p, c) in histo {
        println!("{}\t{}", p, c);
    }
}

