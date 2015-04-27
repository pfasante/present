use sbox::*;

use std::fmt;
use std::ops;

/// subkey struct, that holds the roundkey and the additional key bits for one state
/// of the keyschedule (80 bits alltogether)
pub struct SubkeyT {
    high : u64,
    low : u16
}

impl Clone for SubkeyT {
    fn clone(&self) -> Self {
        SubkeyT::new(self.high, self.low)
    }
}

impl PartialEq for SubkeyT {
    fn eq(&self, other: &SubkeyT) -> bool {
        (self.high == other.high) && (self.low == other.low)
    }
}

impl fmt::Display for SubkeyT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Subkey: 0x{:x}, 0x{:x}", self.high, self.low)
    }
}

impl fmt::Debug for SubkeyT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Subkey: 0x{:x}, 0x{:x}", self.high, self.low)
    }
}

impl SubkeyT {
    pub fn new(high: u64, low: u16) -> Self {
        SubkeyT {high: high, low: low}
    }

    pub fn round_key(&self) -> u64 {
        self.high
    }
}

/// key struct, that holds all subkeys
pub struct KeyT {
    keys: Vec<SubkeyT>
}

impl ops::Index<usize> for KeyT {
    type Output = SubkeyT;
    fn index<'a>(&'a self, idx: usize) -> &'a SubkeyT {
        &self.keys[idx]
    }
}

impl KeyT {
    pub fn new(high: u64, low: u16, rounds: usize) -> Self {
        let mut k = KeyT {keys: vec!(SubkeyT::new(high, low))};
        k.keyschedule(rounds);
        k
    }

    /// for a given key, run the PRESENT keyschedule and return subkeys for n rounds
    fn keyschedule(&mut self, rounds: usize)
    {
        // generate rounds + 1 keys, from key[0]
        for i in 1..(rounds + 1)
        {
            // rotate the key register by 61 bit positions to the left
            // or rotate the key register by 19 bit positions to the right
            let mut low = (((*self)[i - 1].high >> 3) & 0xffff) as u16;
            let mut high = ((*self)[i - 1].high >> 19) |
                           (((*self)[i - 1].low as u64) << 45) |
                           (((*self)[i - 1].high & 0b111) << 61);
            // pass the leftmost nibble through the sbox
            high = (high & 0x0fff_ffff_ffff_ffff) |
                   ((present_sbox().look_up_byte((high >> 60) as u8) as u64) << 60);
            // xor the round_counter to bits 19-15
            low = low ^ (((i & 0b1) << 15) as u16);
            high = high ^ ((i >> 1) as u64);
            (*self).keys.push(SubkeyT::new(high, low));
        }
    }
}

/// returns the PRESENT sbox
pub fn present_sbox() -> SboxT {
    SboxT::new([0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
                0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2])
}


/// the state struct, holding the PRESENT state (64 bit) and implements
/// each layer
pub struct StateT(u64);

impl fmt::Display for StateT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let StateT(state) = *self;
        write!(f, "State: 0x{:x}", state)
    }
}

impl StateT {
    pub fn new(s: u64) -> StateT {
        StateT(s)
    }

    pub fn kxor_layer(&self, key: &KeyT, round: usize) -> Self {
        let StateT(in_state) = *self;
        StateT(in_state ^ key[round].round_key())
    }

    fn sbox_layer(&self) -> Self {
        let StateT(in_state) = *self;
        StateT(present_sbox().look_up_state(in_state))
    }

    fn perm_layer(&self) -> Self {
        let StateT(in_state) = *self;
        let mut perm_state = 0;
        for j in 0..16 {
            for i in 0..4 {
                let old_idx = j *  4 + i;
                let new_idx = i * 16 + j;
                perm_state |= (in_state >> old_idx & 0x1) << new_idx;
            }
        }
        StateT(perm_state)
    }

    /// computes one PRESENT round
    pub fn round(&self, key : &KeyT, round: usize) -> Self {
        self.kxor_layer(key, round).sbox_layer().perm_layer()
    }
}
