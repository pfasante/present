#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <functional>
#include <future>
#include <iostream>
#include <map>
#include <memory>
#include <random>
#include <sstream>
#include <thread>

#include "present_bitsliced.h"
#include "present.h"

using namespace std;

// internal functions
p_corr_histo_t eval_present(p_corr_histo_t histo, p_keys_t keys);
uint64_t present_enc(uint64_t const& plain, round_keys_t const& subkeys);
void print_estimated_mean_var(string name);
void print_mean_var(p_corr_histo_t histo, string name);
void print_histo(p_corr_histo_t histo, string name);

int main(int argc, char *argv[]) {
	cout << "starting evalutation of PRESENT for" << endl;
	cout << NUM_KEYS << " keys" << endl;
	cout << NUM_PLAIN << " plaintexts" << endl;
	cout << NUM_ROUNDS << " rounds" << endl;
	cout << NUM_THREADS << " threads" << endl;
	cout << endl;

	random_device device;
	mt19937_64 generator(device());
	vector<future<p_corr_histo_t>> result_p_histos_indp;
	vector<future<p_corr_histo_t>> result_p_histos_id;

	for (size_t i = 0; i < NUM_THREADS; ++i) {
		cout << "generating random keys...";
		p_corr_histo_t p_histo_indp(new corr_histo_t);
		p_corr_histo_t p_histo_id(new corr_histo_t);


		p_keys_t p_indp_subkeys(new keys_t);
		p_keys_t p_id_subkeys(new keys_t);

		// save key in bitsliced format
		// actually this should be reversed,
		// but as we only need random keys we ignore this
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

		// save key in bitsliced format
		generate(p_id_subkeys->begin(), p_id_subkeys->end(), [&generator]() mutable {
			round_keys_t key;
			// generate one random key and use it in every round
			uint64_t r = generator();
				for (size_t k = 0; k < NUM_ROUNDS+1; ++k) {
					for (size_t i = 0; i < 64; ++i) {
						if (r & (1 << i))
							key[k*64 + i] = 0xffffffffffffffff;
						else
							key[k*64 + i] = 0;
					}
				}
			return key;
		});
		cout << "\t\t\t[ok]" << endl;

		cout << "spawning threads...";
		result_p_histos_indp.push_back(async(launch::async, eval_present, p_histo_indp, p_indp_subkeys));
		result_p_histos_id.push_back(async(launch::async, eval_present, p_histo_id, p_id_subkeys));
		cout << "\t\t\t\t[ok]" << endl;
	}

	p_corr_histo_t p_histo_indp(new corr_histo_t);
	for (auto & p_histo : result_p_histos_indp) {
		corr_histo_t const& histo = *p_histo.get();
		for (size_t i = 0; i < NUM_MASKS; ++i) {
			for (auto const& data : histo[i]) {
				(*p_histo_indp)[i][data.first] += data.second;
			}
		}
	}

	p_corr_histo_t p_histo_id(new corr_histo_t);
	for (auto & p_histo : result_p_histos_id) {
		corr_histo_t const& histo = *p_histo.get();
		for (size_t i = 0; i < NUM_MASKS; ++i) {
			for (auto const& data : histo[i]) {
				(*p_histo_id)[i][data.first] += data.second;
			}
		}
	}

	//print_estimated_mean_var("data/data_est");
	//print_mean_var(p_histo_indp, "data/data_indp");
	//print_mean_var(p_histo_id, "data/data_id");

	print_histo(p_histo_indp, "data/histo_indp");
	print_histo(p_histo_id, "data/histo_id");

	return 0;
}

p_corr_histo_t eval_present(p_corr_histo_t p_histo, p_keys_t p_round_keys) {
	random_device device;
	mt19937_64 generator(device());

	uint64_t keys_done = 0;
	for (auto const& key : *p_round_keys) {
		uint64_t key_counter[NUM_MASKS] = { 0 };
		for (uint64_t p = 0; p < NUM_PLAIN; p += 64) {
			// generate 64 plaintexts in bitsliced format
			// we generate the plaintexts bitwise, so we can save transforming
			// forth and back. this should be in reverse, too. but we can also
			// ignore this, as both, plain- and ciphertexts are in reverse
			uint64_t plain[64];
			uint64_t cipher[64];
			for (size_t i = 0; i < 64; ++i) {
				plain[i] = generator();
				cipher[i] = plain[i];
			}

			// encrypt bitsliced 64 plaintexts
			encrypt(cipher, key.data(), NUM_ROUNDS);

			// check all 64 plaintext/ciphertext pairs if mask holds
			for (unsigned mask_idx = 0; mask_idx < NUM_MASKS; ++mask_idx)
			{
				// get masked in and output bit
				// each is only one bit, as we are only considering one bit masks
				uint64_t input_masked  = plain[input_indices[mask_idx]];
				uint64_t output_masked = cipher[output_indices[mask_idx]];
				uint64_t equal = ~(input_masked ^ output_masked);

				// popcount returns the number of bits set in its 32bit intput
				key_counter[mask_idx] += __builtin_popcount(equal & 0xffffffff);
				key_counter[mask_idx] += __builtin_popcount(equal >> 32);
				//for (size_t i = 0; i < 64; ++i) {
				//	if (!(xor_diff & 1)) {
				//		key_counter[mask_idx] += 1;
				//	}
				//	xor_diff >>= 1;
				//}
			}
		}

		// compute experimental bias for mask and this key,
		// update bias_histogram accordingly
		for (unsigned i = 0; i < NUM_MASKS; ++i)
		{
			double correlation = 2 * ((key_counter[i]) / (double)(NUM_PLAIN) - 0.5);
			// round bias to PRECISION
			correlation = (double)((int)(correlation * pow(10, PRECISION))) / pow(10, PRECISION);
			(*p_histo)[i][correlation]++;
			key_counter[i] = 0;
		}
		keys_done++;
		cout << " " << keys_done; cout.flush();
	}
	return p_histo;
}

void print_histo(p_corr_histo_t p_histo, string name) {
	for (uint64_t i = 0; i < NUM_MASKS; ++i) {
		stringstream ss;
		ss << name << i;
		ofstream output(ss.str ());

		if (!output.is_open ()) {
			cout << "fatal error: could not open " << name << endl;
			return;
		}

		for (const auto &data : (*p_histo)[i]) {
			output << data.first << " " << data.second << "\n";
		}
		output.close();
	}
}

