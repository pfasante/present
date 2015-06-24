#include <algorithm>
#include <array>
#include <iostream>
#include <iomanip>
#include <memory>
#include <random>

#include "present_bitsliced.h"
#include "Encryption_8bit.inc"

using namespace std;

static const uint64_t NUM_ROUNDS = 31;
static const uint64_t NUM_KEYS   = 1;

// we need 64 round keys at the same time,
// because we use a bitsliced present implementation
typedef array<uint64_t, 64*(NUM_ROUNDS+1)> round_keys_t;
typedef array<round_keys_t, NUM_KEYS> keys_t;

typedef shared_ptr<round_keys_t> p_round_keys_t;
typedef shared_ptr<keys_t> p_keys_t;

uint64_t present_enc(uint64_t state, uint64_t *keys, uint64_t rounds);
void present_keys(uint64_t *keys, uint64_t keyhigh, uint64_t keylow, uint64_t rounds);

int main(int argc, char *argv[]) {
	cout << std::hex;
	cout << "test present bitsliced implementation" << endl;
	random_device device;
	mt19937_64 generator(device());

	p_keys_t p_indp_subkeys(new keys_t);

	// save key in bitsliced format
	generate(p_indp_subkeys->begin(), p_indp_subkeys->end(), [&generator]() mutable {
		round_keys_t key;
			for (size_t k = 0; k < NUM_ROUNDS+1; ++k) {
				// generate random key for every round
				uint64_t r = generator();
				for (size_t i = 0; i < 64; ++i) {
					if (r & (1 << i))
						key[k*64 + i] = 0xffffffffffffffff;
					else
						key[k*64 + i] = 0;
				}
			}
		return key;
	});

	uint64_t p_keys[NUM_ROUNDS+1];
	for (size_t i = 0; i < NUM_ROUNDS+1; ++i) {
		p_keys[i] = (*p_indp_subkeys)[0][i*64 + 0] & 1;
		for (size_t j = 1; j < 64; ++j) {
			p_keys[i] |= ((*p_indp_subkeys)[0][i*64 + j] & 1) << j;
		}
		p_keys[i] = mirror64(p_keys[i]);
	}

	uint64_t plain_rev[64];
	uint64_t plain[64];
	uint64_t transposed[64];
	uint64_t cipher[64];
	for (size_t i = 0; i < 64; ++i) {
		plain[i] = generator();
		plain_rev[i] = mirror64(plain[i]);
		//cipher[i] = plain[i];
	}

	transpose(transposed, plain_rev, 64, 64);
	encrypt(transposed, (*p_indp_subkeys)[0].data(), NUM_ROUNDS);
	transpose(cipher, transposed, 64, 64);

	uint64_t c = present_enc(plain[0], p_keys, NUM_ROUNDS);

	cout << "bitsliced encrypt(" << mirror64(plain_rev[0]) << ") = " << mirror64(cipher[0]) << endl;
	cout << "  normal  encrypt(" << plain[0] << ") = " << c << endl;

	cout << "check key schedule" << endl;
	uint64_t bitsliced_keys[64*(NUM_ROUNDS+1)];
	uint64_t key[80*(NUM_ROUNDS+1)];
	uint64_t keys[NUM_ROUNDS+1];
	key_schedule(bitsliced_keys, key, NUM_ROUNDS);
	present_keys(keys, 0, 0, NUM_ROUNDS);

	cout << "generated keys:" << endl
		<< " bitsliced keys  \t  normal keys" << endl;
	for (size_t i = 0; i < NUM_ROUNDS+1; ++i) {
		uint64_t key = 0;
		for (int k = 63; k >= 0; --k) {
			key |= (bitsliced_keys[i*64 + k] & 1) << k;
		}
		cout << setfill('0') << setw(16)
			<< mirror64(key) << " ";
		cout << "\t";
		cout << setfill('0') << setw(16)
			<< keys[i] << " ";
		cout << endl;
	}
	cout << endl;

	for (size_t i = 0; i < 64; ++i) {
		plain[i] = 0x0;
		//cipher[i] = plain[i];
	}

	transpose(transposed, plain, 64, 64);
	encrypt(transposed, bitsliced_keys, NUM_ROUNDS);
	transpose(cipher, transposed, 64, 64);

	cout << "bitsliced encrypt(" << plain[0] << ") = " << mirror64(cipher[0]) << endl;
}

uint64_t present_enc(uint64_t state, uint64_t *keys, uint64_t rounds) {
	for(int round = 0; round < rounds; round++) {
		// Add round key.
		state ^= keys[round];

		// Permutation, SBox.
		uint64_t temp_0 = pBox8_0[ state       &0xFF];
		uint64_t temp_1 = pBox8_1[(state >>  8)&0xFF];
		uint64_t temp_2 = pBox8_2[(state >> 16)&0xFF];
		uint64_t temp_3 = pBox8_3[(state >> 24)&0xFF];
		uint64_t temp_4 = pBox8_4[(state >> 32)&0xFF];
		uint64_t temp_5 = pBox8_5[(state >> 40)&0xFF];
		uint64_t temp_6 = pBox8_6[(state >> 48)&0xFF];
		uint64_t temp_7 = pBox8_7[(state >> 56)&0xFF];

		state=temp_0|temp_1|temp_2|temp_3|temp_4|temp_5|temp_6|temp_7;
	}

	// Add round key.
	return state ^ keys[rounds];
}

void present_keys(uint64_t *keys, uint64_t keyhigh, uint64_t keylow, uint64_t rounds) {
	// Key schedule.
	for(int round = 0; round < rounds+1; round++) {
		keys[round] = keyhigh;
		uint64_t temp = keyhigh;
		keyhigh <<= 61;
		keyhigh |= (keylow << 45);
		keyhigh |= (temp >> 19);
		keylow = (temp >> 3) & 0xFFFF;

		temp = keyhigh >> 60;
		keyhigh &= 0x0FFFFFFFFFFFFFFF;
		temp = sBox4[temp];
		keyhigh |= temp;

		keylow ^= (((round + 1) & 0x01) << 15);
		keyhigh ^= ((round + 1) >> 1);
	}
}

