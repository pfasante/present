extern crate linc;
use linc::present;

fn main() {
    let rounds = 31;
    let k = present::KeyT::new(0x0, 0x0, rounds);
    let mut s = present::StateT::new(0x0);

    for i in 0..rounds {
        s = s.round(&k, i);
        println!("{} round {}", i, s);
    }
    s = s.kxor_layer(&k, rounds);
    println!("final {}", s);
}

