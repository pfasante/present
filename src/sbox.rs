//! TODO add sbox module documentation

use std::fmt;
use std::ops;

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
    pub fn new(ary: [u8; 16]) -> Self {
        SboxT {s: ary}
    }

    /// sbox look up, use both nibbles of idx (i.e. performs two sbox lookups)
    pub fn look_up_byte(&self, idx: u8) -> u8 {
        let lsb = self[idx as usize];
        let msb = self[(idx >> 4) as usize] << 4;
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
                    let (a, b) = (row[left as usize], row[right as usize]);
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
