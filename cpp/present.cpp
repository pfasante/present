#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <functional>
#include <iostream>
#include <map>
#include <memory>
#include <random>
#include <sstream>
#include <thread>

#include "Encryption_8bit.inc"

using namespace std;

static const uint64_t NUM_MASKS  = sizeof(input_indices) / sizeof(unsigned);
static const uint64_t NUM_PLAIN  = 20000,
                      NUM_KEYS   = 100,
                      NUM_ROUNDS = 10;

typedef array<uint64_t, NUM_ROUNDS+1> round_keys_t;
typedef array<map<double, int>, NUM_MASKS> bias_histo_t;

void eval_present(
        shared_ptr<bias_histo_t> histo,
        shared_ptr<array<round_keys_t, NUM_KEYS>> subkey);
uint64_t present_enc(uint64_t const& plain, round_keys_t const& subkey);
void print_histo(shared_ptr<bias_histo_t> histo, string name);

int main(int argc, char *argv[])
{
    random_device device;
    mt19937_64 generator(device());

    shared_ptr<bias_histo_t> p_histo_indp(new bias_histo_t);
    shared_ptr<bias_histo_t> p_histo_id(new bias_histo_t);
    shared_ptr<bias_histo_t> p_histo_no(new bias_histo_t);

    shared_ptr<array<round_keys_t, NUM_KEYS>> p_indp_subkeys(new array<round_keys_t, NUM_KEYS>);
    shared_ptr<array<round_keys_t, NUM_KEYS>> p_id_subkeys(new array<round_keys_t, NUM_KEYS>);
    shared_ptr<array<round_keys_t, NUM_KEYS>> p_no_subkeys(new array<round_keys_t, NUM_KEYS>);

    cout << "generating random keys...";
    generate(p_indp_subkeys->begin(), p_indp_subkeys->end(), [&generator]() mutable {
                array<uint64_t, NUM_ROUNDS+1> key;
                for (auto &k : key) {
                    k = generator();
                }
                return key;
            });
    generate(p_id_subkeys->begin(), p_id_subkeys->end(), [&generator]() mutable {
                uint64_t r = generator();
                array<uint64_t, NUM_ROUNDS+1> key;
                for (auto &k : key) {
                    k = r;
                }
                return key;
            });
    cout << "\t\t\t[ok]" << endl;

    thread independent(eval_present, p_histo_indp, p_indp_subkeys);
    thread identical(eval_present, p_histo_id, p_id_subkeys);
    thread no_key(eval_present, p_histo_no, p_no_subkeys);

    independent.join();
    identical.join();
    no_key.join();

    print_histo(p_histo_indp, "data/histo_indp");
    print_histo(p_histo_id, "data/histo_id");
    print_histo(p_histo_no, "data/histo_no");

    return 0;
}

void eval_present(
        shared_ptr<bias_histo_t> p_histo,
        shared_ptr<array<round_keys_t, NUM_KEYS>> p_round_keys)
{
    random_device device;
    mt19937_64 generator(device());

    uint64_t keys_done = 0;
    for (auto const& key : *p_round_keys) {
        uint64_t key_counter[NUM_MASKS] = { 0 };
        for (uint64_t p = 0; p < NUM_PLAIN; ++p) {
            uint64_t plain = generator();
            // encrypt plaintext
            uint64_t cipher = present_enc(plain, key);

            // check, if mask hold
            for (unsigned mask_idx = 0; mask_idx < NUM_MASKS; ++mask_idx)
            {
                uint64_t input_mask  = (1 << input_indices[mask_idx]);
                uint64_t output_mask = (1 << output_indices[mask_idx]);

                int bit_in  = (plain  & input_mask)  != 0;
                int bit_out = (cipher & output_mask) != 0;

                if (bit_in == bit_out) {
                    key_counter[mask_idx] += 1;
                }
            }
        }

        // compute experimental bias for every mask and this key,
        // update bias_histogram accordingly
        for (unsigned i = 0; i < NUM_MASKS; ++i)
        {
            double bias = 0.5 - (key_counter[i]) / (double)(NUM_PLAIN);
            (*p_histo)[i][bias]++;
            key_counter[i] = 0;
        }
        keys_done++;
        cout << " " << keys_done; cout.flush();
    }
}

uint64_t present_enc(uint64_t const& plain, round_keys_t const& subkey) {
    uint64_t state = plain;
    for (int round = 0; round < NUM_ROUNDS; round++) {
        // Add round key.
        state ^= subkey[round];

        // Permutation, SBox.
        uint64_t temp_0 = pBox8_0[ state       &0xFF];
        uint64_t temp_1 = pBox8_1[(state >>  8)&0xFF];
        uint64_t temp_2 = pBox8_2[(state >> 16)&0xFF];
        uint64_t temp_3 = pBox8_3[(state >> 24)&0xFF];
        uint64_t temp_4 = pBox8_4[(state >> 32)&0xFF];
        uint64_t temp_5 = pBox8_5[(state >> 40)&0xFF];
        uint64_t temp_6 = pBox8_6[(state >> 48)&0xFF];
        uint64_t temp_7 = pBox8_7[(state >> 56)&0xFF];

        state = temp_0 | temp_1 | temp_2 | temp_3 | temp_4 | temp_5 | temp_6 | temp_7;
    }

    // Add round key.
    state ^= subkey[NUM_ROUNDS];
    return state;
}

void print_histo(shared_ptr<bias_histo_t> p_histo, string name) {
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
