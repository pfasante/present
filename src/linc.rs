//! Collection of functions for LINear Cryptanalysis
//!
//! In order to run linear cryptanalysis experiments, we model the relevant
//! parts of a symmmetric cipher with traits. Our analysis build on SPN constructions.
//! Thus we interpret a cipher as a SPN, consisting of a key addition, substitution
//! and permutation layer. These layers are then iterated over several rounds.
//!
//! We assume, that the substitution layer uses one n to n bit sbox, that is applied in
//! parallel to the whole block. The permutation layer applies a bit permutation, and
//! thus we can identify permuted indices of the individual block bits.
//! These properties are used to build a state transition matrix, which models the
//! linear bias over one round. Rising this matrix to the r'th power gives us the linear
//! bias over r rounds. This description also allows to easily include the key dependecy.

pub mod present;

extern crate generic_matrix;
extern crate num;
extern crate rand;

use generic_matrix::Matrix;
use num::traits::PrimInt;
use std::fmt;
use std::ops::Index;

pub trait Sbox<T>: Index<usize> {
    fn new() -> Self;
    fn len(&self) -> usize;
    fn lookup_state(&self, in_state: T) -> T;
    // could be substituted by call(&self, in_state: T)
    //fn index(i: usize) -> T;
    fn state_size() -> usize;
}

pub trait BitPerm<T>: Index<usize, Output=usize> {
    fn new() -> Self;
    fn lookup_state(&self, in_state: T) -> T;
    //fn index(i: usize) -> T;
    //fn call(&self, in_state: T) -> T;
    fn state_size() -> usize;
}

/// compute the walsh transformation of the given sbox, return it as a LAT
pub fn walsh_transform<T, S>() -> LAT
where
    S: Sbox<T> + Index<usize, Output=usize>
{
    let s = &S::new();
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

fn state_graph<U, S, P>(rounds: usize) -> Matrix<u64>
where
    S: 'static + Sbox<U> + Index<usize, Output=usize>,
    P: 'static + BitPerm<U>,
{
    let p = P::new();
    // state is an adjacency-matrix for the ciphers state,
    // it has edges for every one bit trail of the ciphers sbox
    let mut state = Matrix::zero(S::state_size(), S::state_size());
    for i in 0..S::state_size() {
        for (a, b, _) in biased_one_bit(&walsh_transform::<U, S>()) {
            let active_sbox = i as usize / 4;
            if a == 2.pow(i as u32 % 4) {
                let idx = ((b as f64).log(2.0) as usize) + active_sbox * 4;
                let c = p[idx];
                state[(i, c)] = 1;
            }
        }
    }

    // reimplement pow here, as matrix does not support One Trait
    // (and I have no idea how to implement it)
    let mut acc = Matrix::one(S::state_size(), S::state_size());
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
pub fn number_one_bit_trails<U, S, P>(rounds: usize) -> u64
where
    S: 'static + Sbox<U> + Index<usize, Output=usize>,
    P: 'static + BitPerm<U>,
{
    let mat = state_graph::<U, S, P>(rounds);

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
#[allow(dead_code)]
fn all_combinations<U, S, P>(rounds: usize) -> Vec<(usize, usize)>
where
    S: 'static + Sbox<U> + Index<usize, Output=usize>,
    P: 'static + BitPerm<U>,
{
    let mut result = vec![];
    let mat = state_graph::<U, S, P>(rounds);
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

