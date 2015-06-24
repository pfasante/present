/**
 * Bit-Slice Implementation of PRESENT in pure standard C.
 * v1.5 26/08/2011
 *
 * The authors are
 *  Martin Albrecht <martinralbrecht@googlemail.com>
 *  Nicolas T. Courtois <firstinitial.family_name@cs.ucl.ac.uk>
 *  Daniel Hulme <firstname@satalia.com>
 *  Guangyan Song <firstname.lastname@gmail.com>
 * This work was partly funded by the Technology Strategy Board
 * in the United Kingdom under Project No 9626-58525.
 *
 * NEW FEATURES in this version:
 * - it contains an optimized sbox() using 15 only gates, instead of 39
 *   previously
 * - it now supports both 80-bit and 128-bit PRESENT
 * - it contains test vectors for both versions
 *
 * This is a simple and straightforward implementation
 * it encrypts at the speed of
 *   59 cycles per byte on Intel Xeon 5130 1.66 GHz
 * this can be compared to for example
 *   147 cycles per byte for optimized triple DES on the same CPU
 *
 * To compile try:
 *   gcc -O2 -g -std=c99 present_bitslice.c -o present_bitslice
 * will also compile with any version of Microsoft Visual Studio
 *
 * TODO:
 *  - improve performance
 *     It could easily be optimised quite a bit by using SSE2,
 *     pushing critical paths down to assembly
 *  - make it safer, right now we assume sizeof(uint64_t) == 8
 */

#include <string.h>
#include <stdint.h>

static inline void present_sbox(uint64_t *Y0, uint64_t *Y1, uint64_t *Y2, uint64_t *Y3,
		const uint64_t X0, const uint64_t X1, const uint64_t X2, const uint64_t X3) {
	register uint64_t T1, T2, T3, T4;
	T1 = X2 ^ X1;
	T2 = X1 & T1;
	T3 = X0 ^ T2;
	*Y3 = X3 ^ T3;
	T2 = T1 & T3;
	T1 ^= (*Y3);
	T2 ^= X1;
	T4 = X3 | T2;
	*Y2 = T1 ^ T4;
	T2 ^= (~X3);
	*Y0 = (*Y2) ^ T2;
	T2 |= T1;
	*Y1 = T3 ^ T2;
}

void transpose(uint64_t *out, uint64_t const* inp,
		size_t const out_size, size_t const inp_size) {
	for (size_t j = 0; j < out_size; j++) {
		out[j] = 0;
		for (size_t i = 0; i < inp_size; i++) {
			out[j] |= ((inp[i] >> (j & 63)) & 1) << (i & 63);
		}
	}
}

uint64_t mirror64(uint64_t x) {
	uint64_t reverse = 0;
	for (int i = 0; i < 64; i++)
		reverse |= ((x >> i) & 1) << (63 - i);
	return reverse;
}

/** Encryption **/

void sBoxLayer(uint64_t *Y, uint64_t const* X) {
	present_sbox(Y+ 0,Y+ 1,Y+ 2,Y+ 3, X[ 0],X[ 1],X[ 2],X[ 3]);
	present_sbox(Y+ 4,Y+ 5,Y+ 6,Y+ 7, X[ 4],X[ 5],X[ 6],X[ 7]);
	present_sbox(Y+ 8,Y+ 9,Y+10,Y+11, X[ 8],X[ 9],X[10],X[11]);
	present_sbox(Y+12,Y+13,Y+14,Y+15, X[12],X[13],X[14],X[15]);
	present_sbox(Y+16,Y+17,Y+18,Y+19, X[16],X[17],X[18],X[19]);

	present_sbox(Y+20,Y+21,Y+22,Y+23, X[20],X[21],X[22],X[23]);
	present_sbox(Y+24,Y+25,Y+26,Y+27, X[24],X[25],X[26],X[27]);
	present_sbox(Y+28,Y+29,Y+30,Y+31, X[28],X[29],X[30],X[31]);
	present_sbox(Y+32,Y+33,Y+34,Y+35, X[32],X[33],X[34],X[35]);
	present_sbox(Y+36,Y+37,Y+38,Y+39, X[36],X[37],X[38],X[39]);

	present_sbox(Y+40,Y+41,Y+42,Y+43, X[40],X[41],X[42],X[43]);
	present_sbox(Y+44,Y+45,Y+46,Y+47, X[44],X[45],X[46],X[47]);
	present_sbox(Y+48,Y+49,Y+50,Y+51, X[48],X[49],X[50],X[51]);
	present_sbox(Y+52,Y+53,Y+54,Y+55, X[52],X[53],X[54],X[55]);
	present_sbox(Y+56,Y+57,Y+58,Y+59, X[56],X[57],X[58],X[59]);

	present_sbox(Y+60,Y+61,Y+62,Y+63, X[60],X[61],X[62],X[63]);
}

void addRoundKey(uint64_t *X, uint64_t const* K) {
	X[ 0] ^= K[ 0],  X[ 1] ^= K[ 1],  X[ 2] ^= K[ 2],  X[ 3] ^= K[ 3];
	X[ 4] ^= K[ 4],  X[ 5] ^= K[ 5],  X[ 6] ^= K[ 6],  X[ 7] ^= K[ 7];
	X[ 8] ^= K[ 8],  X[ 9] ^= K[ 9],  X[10] ^= K[10],  X[11] ^= K[11];
	X[12] ^= K[12],  X[13] ^= K[13],  X[14] ^= K[14],  X[15] ^= K[15];
	X[16] ^= K[16],  X[17] ^= K[17],  X[18] ^= K[18],  X[19] ^= K[19];
	X[20] ^= K[20],  X[21] ^= K[21],  X[22] ^= K[22],  X[23] ^= K[23];
	X[24] ^= K[24],  X[25] ^= K[25],  X[26] ^= K[26],  X[27] ^= K[27];
	X[28] ^= K[28],  X[29] ^= K[29],  X[30] ^= K[30],  X[31] ^= K[31];
	X[32] ^= K[32],  X[33] ^= K[33],  X[34] ^= K[34],  X[35] ^= K[35];
	X[36] ^= K[36],  X[37] ^= K[37],  X[38] ^= K[38],  X[39] ^= K[39];
	X[40] ^= K[40],  X[41] ^= K[41],  X[42] ^= K[42],  X[43] ^= K[43];
	X[44] ^= K[44],  X[45] ^= K[45],  X[46] ^= K[46],  X[47] ^= K[47];
	X[48] ^= K[48],  X[49] ^= K[49],  X[50] ^= K[50],  X[51] ^= K[51];
	X[52] ^= K[52],  X[53] ^= K[53],  X[54] ^= K[54],  X[55] ^= K[55];
	X[56] ^= K[56],  X[57] ^= K[57],  X[58] ^= K[58],  X[59] ^= K[59];
	X[60] ^= K[60],  X[61] ^= K[61],  X[62] ^= K[62],  X[63] ^= K[63];
}

void pLayer(uint64_t *X, uint64_t const* Y) {
	X[ 0] = Y[ 0],  X[ 1] = Y[ 4],  X[ 2] = Y[ 8],  X[ 3] = Y[12];
	X[ 4] = Y[16],  X[ 5] = Y[20],  X[ 6] = Y[24],  X[ 7] = Y[28];
	X[ 8] = Y[32],  X[ 9] = Y[36],  X[10] = Y[40],  X[11] = Y[44];
	X[12] = Y[48],  X[13] = Y[52],  X[14] = Y[56],  X[15] = Y[60];
	X[16] = Y[ 1],  X[17] = Y[ 5],  X[18] = Y[ 9],  X[19] = Y[13];
	X[20] = Y[17],  X[21] = Y[21],  X[22] = Y[25],  X[23] = Y[29];
	X[24] = Y[33],  X[25] = Y[37],  X[26] = Y[41],  X[27] = Y[45];
	X[28] = Y[49],  X[29] = Y[53],  X[30] = Y[57],  X[31] = Y[61];
	X[32] = Y[ 2],  X[33] = Y[ 6],  X[34] = Y[10],  X[35] = Y[14];
	X[36] = Y[18],  X[37] = Y[22],  X[38] = Y[26],  X[39] = Y[30];
	X[40] = Y[34],  X[41] = Y[38],  X[42] = Y[42],  X[43] = Y[46];
	X[44] = Y[50],  X[45] = Y[54],  X[46] = Y[58],  X[47] = Y[62];
	X[48] = Y[ 3],  X[49] = Y[ 7],  X[50] = Y[11],  X[51] = Y[15];
	X[52] = Y[19],  X[53] = Y[23],  X[54] = Y[27],  X[55] = Y[31];
	X[56] = Y[35],  X[57] = Y[39],  X[58] = Y[43],  X[59] = Y[47];
	X[60] = Y[51],  X[61] = Y[55],  X[62] = Y[59],  X[63] = Y[63];
}

void encrypt(uint64_t *X, uint64_t const* subkeys, const size_t nr) {
	static uint64_t Y[64];
	for (size_t i = 0; i < nr; i++) {
		addRoundKey(X, subkeys + (i*64));
		sBoxLayer(Y, X);
		pLayer(X, Y);
	}
	addRoundKey(X, subkeys + (nr*64));
}


/** Key Schedule **/

void rotate(uint64_t *k) {
	uint64_t temp[80];
	memcpy(temp, k, 80*sizeof(uint64_t));
	for (size_t i = 0; i < 80; i++) {
		k[i] = temp[(i+61) % 80];
	}
}

static inline void round_constant(uint64_t *rc, size_t i) {
	static uint64_t lookup[2] = {
		(uint64_t)0x0000000000000000,((uint64_t)0xffffffffffffffff)
	};
	rc[4] = lookup[(i&(1<<0))>>0];
	rc[3] = lookup[(i&(1<<1))>>1];
	rc[2] = lookup[(i&(1<<2))>>2];
	rc[1] = lookup[(i&(1<<3))>>3];
	rc[0] = lookup[(i&(1<<4))>>4];
}

void key_schedule(uint64_t *subkeys, uint64_t *key, const size_t nr) {
	/** TODO: The key schedule isn't optimised at all */
	uint64_t *ki = subkeys;
	uint64_t S[8];
	uint64_t rc[5];
	for(size_t i = 0; i <= nr; i++) {
		for(size_t j=0; j < 64; j++)
			ki[j] = key[j];
		ki += 64;

		rotate(key);

		present_sbox(&S[0], &S[1], &S[2], &S[3], key[0], key[1], key[2], key[3]);
		key[0] = S[0], key[1] = S[1], key[2] = S[2], key[3] = S[3];
		round_constant(rc, i+1);

		key[60] ^= rc[0];
		key[61] ^= rc[1];
		key[62] ^= rc[2];
		key[63] ^= rc[3];
		key[64] ^= rc[4];
	}
}

