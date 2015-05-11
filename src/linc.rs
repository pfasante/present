//! collection of function for LINear Cryptanalysis
//!
//! every part of a spn cipher is modeled with traits:
//! Sbox, BitPerm, KeySchedule and State allows us to genericly implement a spn
//! cipher with its round function. Using this traits, we also implement
//! functions for walsh-transformation (to examine Sbox'es) and to find linear
//! trails through the cipher.
//!
//! the present module contains implementations of these traits for the PRESENT
//! cipher.

pub mod present;

extern crate num;

use num::traits::PrimInt;
use std::fmt;
use std::ops::Index;

pub trait Sbox<T>: Index<usize> {
    fn new() -> Self;
    fn length(&self) -> usize;
    fn lookup_state(&self, in_state: T) -> T; 
    // could be substituted by call(&self, in_state: T)
    //fn index(i: usize) -> T;
}

pub trait BitPerm: Index<usize> {
    //fn index(i: usize) -> T;
    //fn call(&self, in_state: T) -> T;
}

/// KeySchedule is generic over the initial CipherKey T and the RoundKey U
pub trait KeySchedule<T, U>: Index<usize, Output=U> {
    fn new(key: T, rounds: usize) -> Self;
    //fn index(i: usize) -> T;
}

/// the ciphers State is generic over the inital state and the ciphers key
pub trait Cipher<T, C> {
    fn enc(init: T, cipher_key: C, rounds: usize) -> u64;
}

/// compute the walsh transformation of the given sbox, return it as a LAT
pub fn walsh_transform<T, S>(s: &S) -> LAT
where S: Sbox<T> + Index<usize, Output=usize> {
    let range = s.length();
    let mut lat = LAT::new();
    for i in 0..range {
        let row = walsh_transform_row(s, i);
        lat.table.push(row);
    }
    // the LAT is transposed somehow.. so revert this here
    for i in 0..lat.table.len()-1 {
        for j in 0..i+1 {
            let a = lat[i][j];
            lat.table[i][j] = lat[j][i];
            lat.table[j][i] = a;
        }
    }
    lat
}

fn walsh_transform_row<T, S>(s: &S, beta: usize) -> Vec<i32>
where S: Sbox<T> + Index<usize, Output=usize> {
    let range = s.length();
    let mut row = Vec::new();
    // initialize the row
    for i in 0..range {
        row.push((-1i32).pow(dot_prod_f2(beta, s[i]) as u32));
    }
    let mut step = 1;
    while step < range {
        let mut left = 0;
        let blocks = range / (step * 2);
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
    for i in 0..range {
        row[i] /= 2;
    }
    // return it
    row
}

fn dot_prod_f2<T>(a: T, b: T) -> T 
where T: PrimInt {
    let n = (a.count_ones() + a.count_zeros()) as usize;
    assert!(n == (b.count_ones() + b.count_zeros()) as usize,
               "a and b does not have same bitlength?!");
    let mut x = T::zero();
    for i in 0..n {
        x = x ^ (((a >> i) & T::one()) * ((b >> i) & T::one()));
    }
    x
}

pub struct LAT {
    table: Vec<Vec<i32>>
}

impl Index<usize> for LAT {
    type Output = Vec<i32>;
    fn index<'a>(&'a self, idx: usize) -> &'a Vec<i32> {
        &self.table[idx]
    }
}

impl fmt::Display for LAT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut cnt = 0;
        let mut res = write!(f, "LAT:\n");

        // writen header row
        res = res.and(write!(f, "   "));
        for i in 0..self.table.len() {
            res = res.and(write!(f, "{:>2} ", i));
        }
        res = res.and(write!(f, "\n"));

        for i in self.table.iter() {
            // write header column
            res = res.and(write!(f, "{:>2} ", cnt));
            cnt += 1;
            // write table content
            for &j in i.iter() {
                res = res.and(
                    if j == 0 {
                        write!(f, "   ")
                    } else {
                        write!(f, "{:>2} ", j)
                    });
            }
            res = res.and(write!(f, "\n"));
        }
        res
    }
}

impl LAT {
    fn new() -> LAT {
        LAT {table: Vec::new()}
    }
}

pub fn biased_one_bit(lat: &LAT) -> Vec<(usize, usize, i32)> {
    let mut biased_masks = Vec::new();
    let mut i = 1;
    while i < lat.table.len() {
        let mut j = 1;
        while j < lat[0].len() {
            if (*lat)[i][j] != 0 {
                biased_masks.push((i, j, (*lat)[i][j]));
            }
            j *= 2;
        }
        i *= 2;
    }
    biased_masks
}

