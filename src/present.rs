extern crate rand;

use super::*;
use std::ops;
//use rand::{Rand, Rng};

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

#[allow(dead_code)]
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

    fn state_size() -> usize {
        64
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

#[allow(dead_code)]
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

impl ops::Index<usize> for PresentPermutation {
    type Output = usize;
    fn index<'a>(&'a self, idx: usize) -> &'a usize {
        assert!(idx < 64, "index out of range");
        // TODO hack: how to return the computed index without
        // using this dumb array referencing?
        &self.idx[(idx % 4) * 16 + (idx / 4)]
    }
}

impl BitPerm<u64> for PresentPermutation {
    fn new() -> Self {
        PRESENTPERMUTATION.clone()
    }

    fn lookup_state(&self, in_state: u64) -> u64 {
        let mut out_state = 0;
        for i in 0..16 {
            let lookup = self[((in_state >> (i * 4)) & 0xf) as usize] as u64;
            out_state |= lookup << (i * 4);
        }
        out_state
    }

    fn state_size() -> usize {
        64
    }
}

