//! Collection of function for LINear Cryptanalysis
//!
//! Every part of a SPN cipher is modeled with traits:
//! Sbox, BitPerm, KeySchedule and State allows us to generically implement a SPN
//! Cipher with its round function. Using this traits, we also implement
//! functions for Walsh-transformation (to examine S-box'es) and to find linear
//! trails through the cipher.
//!
//! The present module contains implementations of these traits for the PRESENT
//! Cipher.

pub mod present;

extern crate generic_matrix;
extern crate num;
extern crate rand;

use generic_matrix::Matrix;
use num::traits::PrimInt;
use rand::{Rand};
use std::fmt;
use std::ops::Index;

/// KeySchedule is generic over the initial CipherKey T and the RoundKey U
pub trait KeySchedule<T, U>: Index<usize, Output=U> {
    fn new(key: T, rounds: usize) -> Self;
    //fn index(i: usize) -> T;
}

pub trait Sbox<T>: Index<usize> {
    fn new() -> Self;
    fn len(&self) -> usize;
    fn lookup_state(&self, in_state: T) -> T;
    // could be substituted by call(&self, in_state: T)
    //fn index(i: usize) -> T;
}

pub trait BitPerm: Index<usize, Output=usize> {
    //fn index(i: usize) -> T;
    //fn call(&self, in_state: T) -> T;
}

/// the ciphers State is generic over the inital state and the ciphers key
pub trait Cipher<T, C, S: Sbox<T>, P: BitPerm> {
    fn new(cipher_key: C, rounds: usize) -> Self;
    fn enc(&self, init: T, rounds: usize) -> T;
    fn dec(&self, init: T, rounds: usize) -> T;
    fn state_size() -> usize;
    fn sbox() -> &'static S;
    fn permutation() -> &'static P;
}

/// compute the walsh transformation of the given sbox, return it as a LAT
pub fn walsh_transform<T, S>(s: &S) -> LAT
where
    S: Sbox<T> + Index<usize, Output=usize>
{
    let range = s.len();
    let mut lat = LAT::new();
    for i in 0..range {
        let row = walsh_transform_row(s, i);
        lat.table.push(row);
    }
    // the LAT is transposed somehow.. revert this here
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
where
    S: Sbox<T> + Index<usize, Output=usize>
{
    let range = s.len();
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
    // return it
    row
}

fn dot_prod_f2<T>(a: T, b: T) -> T
where T: PrimInt {
    let n = (a.count_ones() + a.count_zeros()) as usize;
    assert!(n == (b.count_ones() + b.count_zeros()) as usize,
               "a and b does not have same bit-length?!");
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

        // write header row
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
    /// returns a new, empty LAT
    fn new() -> LAT {
        LAT {table: Vec::new()}
    }
}

/// computes a vector holding the biased one bit input output masks
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

fn state_graph<T, U, K, S, P>(rounds: usize) -> Matrix<u64>
where
    S: 'static + Sbox<U> + Index<usize, Output=usize>,
    P: 'static + BitPerm,
    T: Cipher<U, K, S, P>
{
    // state is an adjacency-matrix for the ciphers state,
    // it has edges for every one bit trail of the ciphers sbox
    let mut state = Matrix::zero(T::state_size(), T::state_size());
    for i in 0..T::state_size() {
        for (a, b, _) in biased_one_bit(&walsh_transform(T::sbox())) {
            let active_sbox = i as usize / 4;
            if a == 2.pow(i as u32 % 4) {
                let idx = ((b as f64).log(2.0) as usize) + active_sbox * 4;
                let c = T::permutation()[idx];
                state[(i, c)] = 1;
            }
        }
    }

    // reimplement pow here, as matrix does not support One Trait
    // (and I have no idea how to implement it)
    let mut acc = Matrix::one(T::state_size(), T::state_size());
    let mut exp = rounds;
    while exp > 0 {
        if (exp & 1) == 1 {
            acc = acc * state.clone();
        }
        state = state.clone() * state.clone();
        exp >>= 1;
    }

    acc
}

/// returns the number of maximal trails for given round, for every
/// one bit biased linear hull through the cipher.
pub fn number_one_bit_trails<T, U, K, S, P>(rounds: usize) -> u64
where
    S: 'static + Sbox<U> + Index<usize, Output=usize>,
    P: 'static + BitPerm,
    T: Cipher<U, K, S, P>
{
    let mat = state_graph::<T, U, K, S, P>(rounds);

    // find biggest entry in acc matrix
    // generic-matrix does not implement iterators,
    // so access every element per index
    let mut max = 0;
    let (row, column) = mat.size();
    for i in 0..row {
        for j in 0..column {
            if mat[(i, j)] > max {
                max = mat[(i, j)];
            }
        }
    }
    max
}

// return all combinations for given state length and masks
//
// TODO could be written nicer with functional tools: filter and map
fn all_combinations<T, U, K, S, P>(rounds: usize) -> Vec<(usize, usize)>
where
    S: 'static + Sbox<U> + Index<usize, Output=usize>,
    P: 'static + BitPerm,
    T: Cipher<U, K, S, P>
{
    let mut result = vec![];
    let mat = state_graph::<T, U, K, S, P>(rounds);
    let (row, column) = mat.size();
    for i in 0..row {
        for j in 0..column {
            if mat[(i, j)] != 0 {
                result.push((i, j));
            }
        }
    }

    result
}

/// encrypt #n_p random plaintexts for r rounds and
/// and #n_r random keys
/// return the distribution of the linear biases
//pub fn distribution<T, U, K, S, P>(iterations: usize, rounds: usize) -> Vec<(i32, i32)>
pub fn distribution<T, K, S, P>(n_p: usize, n_k: usize, rounds: usize) -> Vec<(f64, i32)>
where
// TODO how to keep this generic over the ciphers state?
    //U: Rand,
    K: Rand,
    S: 'static + Sbox<u64> + Index<usize, Output=usize>,
    P: 'static + BitPerm,
    T: Cipher<u64, K, S, P>
{
    // initialize counters for every trail
    let mut rng = rand::thread_rng();
    let biased_masks = all_combinations::<T, u64, K, S, P>(rounds);
    let mut counter = vec![];
    for _ in 0..biased_masks.len() {
        counter.push(0);
    }

    // count for #iterations, how often the trail holds, i.e. its bias
    for _ in 0..n_k {
        let cipher = T::new(K::rand(&mut rng), rounds);
        for _ in 0..n_p {
            let m = u64::rand(&mut rng);
            let c = cipher.enc(m, rounds);
            for i in 0..biased_masks.len() {
                let (a, b) = biased_masks[i];
                let bit_in = (m & (a as u64)) != 0;
                let bit_out = (c & (b as u64)) != 0;
                if bit_in == bit_out {
                    counter[i] += 1;
                }
            }
        }
    }

    // sort biases and create histogram
    counter.sort();
    let mut histo = vec![];
    let mut prev = -1;
    let mut idx = -1;
    for i in counter {
        if i == prev {
            let (corr, c) = histo[idx];
            histo[idx] = (corr, c + 1);
        } else {
            // round correlation to fixed precision
            let corr = (2.0 * ((i as f64) / (n_p.pow(2) as f64) - 0.5)
                        * 100000.0).round() / 10000.0;
            histo.push((corr, 1));
            idx += 1;
            prev = i;
        }
    }
    histo
}
