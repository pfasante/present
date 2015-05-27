use super::*;
use std::ops;
use rand::{Rand, Rng};

/// PresentSbox implements the PRESENT sbox, by implementing the Sbox trait
/// a Sbox lookup can than be accomplished by indexing the sbox.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentSbox {
    size: usize,
    elems: &'static [usize]
}

static PRESENTSBOX: PresentSbox =
    PresentSbox {size: 16, elems:
        &[0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
          0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2]};

static PRESENTSBOX_INV: PresentSbox =
    PresentSbox {size: 16, elems:
        &[0x5, 0xe, 0xf, 0x8, 0xc, 0x1, 0x2, 0xd,
          0xb, 0x4, 0x6, 0x3, 0x0, 0x7, 0x9, 0xa]};

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

static PRESENTPERMUTATION: PresentPermutation =
    PresentPermutation {idx:
        &[ 0,16,32,48, 1,17,33,49, 2,18,34,50, 3,19,35,51,
           4,20,36,52, 5,21,37,53, 6,22,38,54, 7,23,39,55,
           8,24,40,56, 9,25,41,57,10,26,42,58,11,27,43,59,
          12,28,44,60,13,29,45,61,14,30,46,62,15,31,47,63]};

static PRESENTPERMUTATION_INV: PresentPermutation =
    PresentPermutation {idx:
        &[0x0, 0x4, 0x8, 0xc, 0x10, 0x14, 0x18, 0x1c,
          0x20, 0x24, 0x28, 0x2c, 0x30, 0x34, 0x38, 0x3c,
          0x1, 0x5, 0x9, 0xd, 0x11, 0x15, 0x19, 0x1d,
          0x21, 0x25, 0x29, 0x2d, 0x31, 0x35, 0x39, 0x3d,
          0x2, 0x6, 0xa, 0xe, 0x12, 0x16, 0x1a, 0x1e,
          0x22, 0x26, 0x2a, 0x2e, 0x32, 0x36, 0x3a, 0x3e,
          0x3, 0x7, 0xb, 0xf, 0x13, 0x17, 0x1b, 0x1f,
          0x23, 0x27, 0x2b, 0x2f, 0x33, 0x37, 0x3b, 0x3f
         ]};

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

impl Rand for PresentCipherKey {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        PresentCipherKey {
            high: u64::rand(rng),
            low: u16::rand(rng),
        }
    }
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
    sbox: &'static PresentSbox,
    perm: &'static PresentPermutation,
    keys: PresentKeySchedule
}


impl Cipher<u64, PresentCipherKey, PresentSbox, PresentPermutation> for Present {
    fn new(key: PresentCipherKey, rounds: usize) -> Self {
        let k = PresentKeySchedule::new(key, rounds);
        Present {
            sbox: &PRESENTSBOX,
            perm: &PRESENTPERMUTATION,
            keys: k
        }
    }

    fn enc(&self, init: u64, rounds: usize) -> u64 {
        let mut state = init;
        for i in 0..rounds {
            state = self.kxor_layer(state, i);
            state = self.sbox_layer(state);
            state = self.perm_layer(state);
        }
        self.kxor_layer(state, rounds)
    }

    fn dec(&self, init: u64, rounds: usize) -> u64 {
        let mut state = init;
        for i in (1..rounds+1).rev() {
            state = self.kxor_layer(state, i);
            state = self.perm_inv_layer(state);
            state = self.sbox_inv_layer(state);
        }
        self.kxor_layer(state, 0)
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
    fn kxor_layer(&self, state: u64, round: usize) -> u64 {
        self.keys[round].key ^ state
    }

    fn sbox_layer(&self, state: u64) -> u64 {
        PRESENTSBOX.lookup_state(state)
    }

    fn sbox_inv_layer(&self, state: u64) -> u64 {
        PRESENTSBOX_INV.lookup_state(state)
    }

    fn perm_layer(&self, state: u64) -> u64 {
        //self.state = self.perm(self.state);
        let mut perm_state = 0;
        for old_idx in 0..64 {
            let new_idx = PRESENTPERMUTATION.idx[old_idx];
            perm_state |= (state >> old_idx & 0x1) << new_idx;
        }
        perm_state
    }

    fn perm_inv_layer(&self, state: u64) -> u64 {
        //self.state = self.perm(self.state);
        let mut perm_state = 0;
        for old_idx in 0..64 {
            let new_idx = PRESENTPERMUTATION_INV.idx[old_idx];
            perm_state |= (state >> old_idx & 0x1) << new_idx;
        }
        perm_state
    }
}

