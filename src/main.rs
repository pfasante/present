extern crate linc;
use linc::present;

fn main() {
    let k = present::KeyT::new(0x0, 0x0, 31);
    let mut s = present::StateT::new(0x0);

    for i in 0..30 {
        s = s.round(&k, i);
        println!("{} round {}", i, s);
    }
    s.kxor_layer(&k, 30);
    println!("final {}", s);
}

