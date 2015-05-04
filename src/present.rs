use super::*;
use std::ops;

/// PresentSbox implements the PRESENT sbox, by implementing the Sbox trait
/// a Sbox lookup can than be accomplished by indexing the sbox.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentSbox {
    size: usize,
    elems: Vec<usize>
}

/// sbox lookup, ignores the higher nibble (returns 0x0 in it)
impl ops::Index<usize> for PresentSbox {
    type Output = usize;
    fn index<'a>(&'a self, idx: usize) -> &'a usize {
        &self.elems[idx & 0xf]
    }
}

impl Sbox<u64> for PresentSbox {
    fn new() -> Self {
        PresentSbox {size: 16, elems:
            vec![0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
                 0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2]}
    }

    fn length(&self) -> usize {
        self.size
    }

    fn lookup_state(&self, in_state: u64) -> u64 {
        let mut out_state = 0;
        for i in 0..16 {
            let lookup = self[((in_state >> (i * 4)) & 0xf) as usize] as u64;
            out_state |= lookup << (i * 4);
        }
        out_state
    }
}

/// present permutation
/// TODO missing Fn Trait implementation to permute whole present state
pub struct PresentPermutation {
    // FIXME hack for Index implementation
    idx: Vec<usize>
}

// FIXME hack for Index implementation
impl PresentPermutation {
    fn new() -> Self {
        let mut v = Vec::new();
        for i in 0..64 {
            v.push(i);
        }
        PresentPermutation {idx: v}
    }
}

impl ops::Index<usize> for PresentPermutation {
    type Output = usize;
    fn index<'a>(&'a self, idx: usize) -> &'a usize {
        assert!(idx < 64, "index out of range");
        // TODO hack: how to return the computed index without
        // using this dumb vector referencing?
        &self.idx[(idx % 5) * 16 + (idx / 4)]
    }
}

impl BitPerm for PresentPermutation {}

/// PresentCipherKey is the initial 80bit key
#[derive(Clone, Debug, PartialEq)]
pub struct PresentCipherKey {
    high: u64,
    low: u16
}

impl PresentCipherKey {
    pub fn new(high: u64, low: u16) -> Self {
        PresentCipherKey {high: high, low: low}
    }
}

/// PresentRoundKey is a 64bit round key
#[derive(Clone, Debug, PartialEq)]
pub struct PresentRoundKey {
    key: u64
}

impl PresentRoundKey {
    pub fn new(k: u64) -> Self {
        PresentRoundKey {key: k}
    }
}

pub struct PresentKeySchedule {
    size: usize,
    elems: Vec<PresentRoundKey>
}

impl ops::Index<usize> for PresentKeySchedule {
    type Output = PresentRoundKey;
    fn index<'a>(&'a self, idx: usize) -> &'a PresentRoundKey {
        assert!(idx < self.size, "index out of range");
        &self.elems[idx]
    }
}

impl KeySchedule<PresentCipherKey, PresentRoundKey> for PresentKeySchedule {
    fn new(key: PresentCipherKey, rounds: usize) -> Self {
        let sbox = PresentSbox::new();
        let mut keys = vec![PresentRoundKey {key: key.high}];

        // run keyschedule
        let mut old_low = key.low;
        let mut old_high = key.high;
        for i in 1..(rounds + 1) {

            // rotate the key register by 61 bit positions to the left
            // or rotate the key register by 19 bit positions to the right
            let mut low = ((old_high >> 3) & 0xffff) as u16;
            let mut high = (old_high >> 19)
                 | ((old_low as u64) << 45)
                 | ((old_high & 0b111) << 61);

            // pass the leftmost nibble through the sbox
            high = (high & 0x0fff_ffff_ffff_ffff) |
                   ((sbox[(high >> 60) as usize] as u64) << 60);

            // xor the round_counter to bits 19-15
            low = low ^ (((i & 0b1) << 15) as u16);
            high = high ^ ((i >> 1) as u64);

            old_low = low;
            old_high = high;
            keys.push(PresentRoundKey {key: high});
        }
        PresentKeySchedule {size: rounds + 1, elems: keys}
    }
}

/// Present implements the Cipher Trait, to tie everything together
pub struct Present {
    state: u64,
    sbox: PresentSbox,
    perm: PresentPermutation,
    keys: PresentKeySchedule
}

impl Cipher<u64, PresentCipherKey> for Present {
    fn enc(init: u64, key: PresentCipherKey, rounds: usize) -> u64 {
        let s = PresentSbox::new();
        let b = PresentPermutation::new();
        let k = PresentKeySchedule::new(key, rounds);
        let mut c = Present {state: init, sbox: s, perm: b, keys: k};
        
        for i in 0..rounds {
            c.kxor_layer(i);
            c.sbox_layer();
            c.perm_layer();
        }
        c.kxor_layer(rounds);
        c.state
    }
}

impl Present {
    fn kxor_layer(&mut self, round: usize) {
        self.state = self.keys[round].key ^ self.state;
    }

    fn sbox_layer(&mut self) {
        self.state = self.sbox.lookup_state(self.state);
    }

    fn perm_layer(&mut self) {
        //self.state = self.perm(self.state);
        let mut perm_state = 0;
        for j in 0..16 {
            for i in 0..4 {
                let old_idx = j *  4 + i;
                let new_idx = i * 16 + j;
                perm_state |= (self.state >> old_idx & 0x1) << new_idx;
            }
        }
        self.state = perm_state;
    }
}

