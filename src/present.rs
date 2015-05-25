use super::*;
use std::ops;

/// PresentSbox implements the PRESENT sbox, by implementing the Sbox trait
/// a Sbox lookup can than be accomplished by indexing the sbox.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentSbox {
    size: usize,
    elems: &'static [usize]
}

static PRESENTSBOX : PresentSbox =
    PresentSbox {size: 16, elems:
        &[0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
          0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2]};

/// sbox lookup, ignores the higher nibble (returns 0x0 in it)
impl ops::Index<usize> for PresentSbox {
    type Output = usize;
    fn index<'a>(&'a self, idx: usize) -> &'a usize {
        &self.elems[idx & 0xf]
    }
}

impl Sbox<u64> for PresentSbox {
    fn new() -> Self {
        PRESENTSBOX.clone()
    }

    fn len(&self) -> usize {
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
#[derive(Clone, Debug, PartialEq)]
pub struct PresentPermutation {
    idx: &'static [usize]
}

static PRESENTPERMUTATION : PresentPermutation =
    PresentPermutation {idx:
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
         16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
         32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
         48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63]};

impl PresentPermutation {
    #[allow(dead_code)]
    fn new() -> Self {
        PRESENTPERMUTATION.clone()
    }
}

impl ops::Index<usize> for PresentPermutation {
    type Output = usize;
    fn index<'a>(&'a self, idx: usize) -> &'a usize {
        assert!(idx < 64, "index out of range");
        // TODO hack: how to return the computed index without
        // using this dumb array referencing?
        &self.idx[(idx % 4) * 16 + (idx / 4)]
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

#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct Present {
    state: u64,
    sbox: &'static PresentSbox,
    perm: &'static PresentPermutation,
    keys: PresentKeySchedule
}


impl Cipher<u64, PresentCipherKey, PresentSbox, PresentPermutation> for Present {
    fn enc(init: u64, key: PresentCipherKey, rounds: usize) -> u64 {
        let k = PresentKeySchedule::new(key, rounds);
        let mut c = Present {state: init,
                             sbox: &PRESENTSBOX,
                             perm: &PRESENTPERMUTATION,
                             keys: k};

        for i in 0..rounds {
            c.kxor_layer(i);
            c.sbox_layer();
            c.perm_layer();
        }
        c.kxor_layer(rounds);
        c.state
    }

    fn state_size() -> usize {
        64
    }

    fn sbox() -> &'static PresentSbox {
        &PRESENTSBOX
    }

    fn permutation() -> &'static PresentPermutation {
        &PRESENTPERMUTATION
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

