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

#include "present_bitsliced.h"

using namespace std;

static const uint64_t NUM_TRAILS[] = {1, 1, 1, 3, 9, 27, 72, 192, 512, 1344, 3528, 9261, 24255, 63525, 166375, 435600, 1140480, 2985984, 7817472, 20466576, 53582633, 140281323, 367261713, 961504803, 2517252696, 6590254272, 17253512704, 45170283840, 118257341400, 309601747125, 810547899975};
static const uint64_t input_indices[] = {21};
static const uint64_t output_indices[] = {21};

static const uint64_t NUM_MASKS  = sizeof(input_indices) / sizeof(unsigned);
static const uint64_t NUM_ROUNDS = 5;
static const uint64_t NUM_KEYS   = 10000;

// NUM_PLAINS = 1/c^2 * 2 * N_T
static const uint64_t NUM_PLAIN  = (1 << (4*NUM_ROUNDS)) * 2 * NUM_TRAILS[NUM_ROUNDS-1];

typedef array<uint64_t, NUM_ROUNDS+1> round_keys_t;
typedef array<map<double, int>, NUM_MASKS> bias_histo_t;

void eval_present(
        shared_ptr<bias_histo_t> histo,
        shared_ptr<array<round_keys_t, NUM_KEYS>> subkey);
uint64_t present_enc(uint64_t const& plain, round_keys_t const& subkey);
void print_estimated_mean_var(string name);
void print_mean_var(shared_ptr<bias_histo_t> histo, string name);
void print_histo(shared_ptr<bias_histo_t> histo, string name);

int main(int argc, char *argv[])
{
    shared_ptr<bias_histo_t> p_histo_indp(new bias_histo_t);
    shared_ptr<bias_histo_t> p_histo_id(new bias_histo_t);

    shared_ptr<array<round_keys_t, NUM_KEYS>> p_indp_subkeys(new array<round_keys_t, NUM_KEYS>);
    shared_ptr<array<round_keys_t, NUM_KEYS>> p_id_subkeys(new array<round_keys_t, NUM_KEYS>);

    cout << "generating random keys...";
    random_device device;
    mt19937_64 generator(device());

    generate(p_indp_subkeys->begin(), p_indp_subkeys->end(), [&generator]() mutable {
                round_keys_t key;
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

    independent.join();
    identical.join();

    print_estimated_mean_var("data/data_est");
    print_mean_var(p_histo_indp, "data/data_indp");
    print_mean_var(p_histo_id, "data/data_id");

    print_histo(p_histo_indp, "data/histo_indp");
    print_histo(p_histo_id, "data/histo_id");

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
        for (uint64_t p = 0; p < NUM_PLAIN; p += 64) {
            // generate 64 plaintexts
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
                uint64_t input_masked  = plain[input_indices[mask_idx]];
                uint64_t output_masked = cipher[output_indices[mask_idx]];
                uint64_t equal = ~(input_masked ^ output_masked);

                key_counter[mask_idx] += __builtin_popcount(equal & 0xffffffff);
                key_counter[mask_idx] += __builtin_popcount(equal >> 32);
                //for (size_t i = 0; i < 64; ++i) {
                //    if (!(xor_diff & 1)) {
                //        key_counter[mask_idx] += 1;
                //    }
                //    xor_diff >>= 1;
                //}
            }
        }

        // compute experimental bias for every mask and this key,
        // update bias_histogram accordingly
        for (unsigned i = 0; i < NUM_MASKS; ++i)
        {
            double bias = (key_counter[i]) / (double)(NUM_PLAIN) - 0.5;
            (*p_histo)[i][2.0 * bias]++;
            key_counter[i] = 0;
        }
        keys_done++;
        cout << " " << keys_done; cout.flush();
    }
}

void print_estimated_mean_var(string name) {
    double mean = 0.0;
    double var = 2.0;
    var = pow(var, -4.0 * NUM_ROUNDS) * NUM_TRAILS[NUM_ROUNDS-1];

    ofstream output(name);
    if (!output.is_open()) {
        cout << "fatal error: could not open " << name << endl;
        return;
    }

    output << "mean " << mean << endl;
    output << "var  " << var << endl;
    output.close();
}

double weight(map<double, int> const& data) {
    double weight_sum = 0;
    for (auto const& x : data) {
        weight_sum += x.second;
    }
    return weight_sum;
}

double weighted_mean(map<double, int> const& data) {
    double mean = 0.0;
    double weight_sum = weight(data);
    for (auto const& x : data) {
        mean += x.first * x.second;
    }
    return mean / weight_sum;
}

double weighted_var(map<double, int> const& data) {
    double var = 0.0;
    double mean = weighted_mean(data);
    double weight_sum = weight(data);
    for (auto const& x : data) {
        var += pow(x.first - x.second * mean, 2.0);
        weight_sum += x.second;
    }
    return var / weight_sum;
}

void print_mean_var(shared_ptr<bias_histo_t> p_histo, string name) {
    double mean = 0.0;
    for (auto const& histo : (*p_histo)) {
        mean += weighted_mean(histo);
    }
    mean /= p_histo->size();

    double var = 0.0;
    for (auto const& histo : (*p_histo)) {
        var += weighted_var(histo);
    }
    var /= p_histo->size();

    ofstream output(name);
    if (!output.is_open()) {
        cout << "fatal error: could not open " << name << endl;
        return;
    }

    output << "mean " << mean << endl;
    output << "var  " << var << endl;
    output.close();
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

