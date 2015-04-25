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
    pub fn new(high: u64, low: u16) -> Self {
        KeyT {keys: vec!(SubkeyT::new(high, low))}
    }

    /// for a given key, run the PRESENT keyschedule and return subkeys for n rounds
    pub fn keyschedule(&mut self, rounds : usize)
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

/// sbox struct, implements methods for look up
/// can be constructed with the present_sbox function
pub struct SboxT {
    s : [u8; 16]
}

/// sbox look up, use only lower nibble of idx and return 0x0 in the higher nibble
impl ops::Index<usize> for SboxT {
    type Output = u8;
    fn index<'a>(&'a self, idx: usize) -> &'a u8 {
        &self.s[idx & 0xf]
    }
}

impl SboxT {
    fn new(ary: [u8; 16]) -> Self {
        SboxT {s: ary}
    }

    /// sbox look up, use both nibbles of idx (i.e. performs two sbox lookups)
    pub fn look_up_byte(&self, idx: u8) -> u8 {
        let lsb = self.s[idx as usize];
        let msb = self.s[(idx >> 4) as usize] << 4;
        msb | lsb
    }

    /// sbox look up, use both nibbles of idx (i.e. performs two sbox lookups)
    pub fn look_up_state(&self, idx: u64) -> u64 {
        let mut output = 0;
        output |= (self.look_up_byte(((idx >> 56) & 0xff) as u8) as u64) << 56;
        output |= (self.look_up_byte(((idx >> 48) & 0xff) as u8) as u64) << 48;
        output |= (self.look_up_byte(((idx >> 40) & 0xff) as u8) as u64) << 40;
        output |= (self.look_up_byte(((idx >> 32) & 0xff) as u8) as u64) << 32;
        output |= (self.look_up_byte(((idx >> 24) & 0xff) as u8) as u64) << 24;
        output |= (self.look_up_byte(((idx >> 16) & 0xff) as u8) as u64) << 16;
        output |= (self.look_up_byte(((idx >>  8) & 0xff) as u8) as u64) <<  8;
        output |  (self.look_up_byte(((idx >>  0) & 0xff) as u8) as u64) <<  0
    }

    /// compute the walsh transformation of the sbox and return it as a
    /// one row of the linear approximation table
    fn walsh_transform_row(&self, beta: u8) -> [i32; 16] {
        // TODO fix
        let mut row = [0; 16];
        // initialize the row array
        for i in 0..16 {
            row[i] =(-1i32).pow(dot_product_f2(beta, self[i as usize]) as u32);
        }
        let mut step = 1;
        while step < 16 {
            let mut left = 0;
            let blocks = 16 / (step * 2);
            for _ in 0..blocks {
                let mut right = left + step;
                for _ in 0..step {
                    let a = row[left as usize];
                    let b = row[right as usize];
                    row[left as usize] = a + b;
                    row[right as usize] = a - b;
                    left += 1;
                    right += 1;
                }
                left = right;
            }
            step *= 2;
        }
        // scale row by 0.5, we want to compute biases
        for i in 0..16 {
            row[i] /= 2;
        }
        // return it
        row
    }

    /// compute the walsh transformation of the sbox and return it as a
    /// linear approximation table
    pub fn walsh_transform(&self) -> LAT {
        let mut lat = LAT::new();
        for i in 0..16 {
            lat.table[i] = self.walsh_transform_row(i as u8);
        }
        lat
    }
}

fn dot_product_f2(a: u8, b: u8) -> u8 {
    let mut x = 0;
    for i in 0..8 {
        x ^= ((a >> i) & 1) * ((b >> i) & 1);
    }
    x
}

/// returns the PRESENT sbox
pub fn present_sbox() -> SboxT {
    SboxT::new([0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
                0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2])
}

pub struct LAT {
    table: [[i32; 16]; 16]
}

impl fmt::Display for LAT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut res = write!(f, "LAT:");
        for i in 0..16 {
            for j in 0..16 {
                res = res.and(write!(f, "{} ", self.table[i][j]))
            }
            res = res.and(write!(f, "\n"));
        }
        res
    }
}

impl LAT {
    fn new() -> LAT {
        LAT {table: [[0; 16]; 16]}
    }
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
    fn kxor_layer(&self, key: &KeyT, round: usize) -> Self {
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
