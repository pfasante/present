#include <cmath>
#include <cstdint>
#include <cstdlib>

#include <iostream>
#include <iomanip>
#include <future>
#include <map>
#include <mutex>
#include <thread>
#include <utility>
#include <vector>

#include "cmdline.h"
#include "keyschedule.h"
#include "present_bitslice.h"

using namespace std;

mutex mut_cout;

const int NROUNDS = 5;
long args_nkeys;
long args_nplains;
int args_nthreads;

pair<map<double, double>, map<double, double>> check_keys(uint64_t alpha, uint64_t beta);

template<template<size_t> class KEY_T, size_t NR>
map<double, double> experiment(uint64_t alpha, uint64_t beta);

void check_old();

int main(int argc, char **argv) {
	gengetopt_args_info args_info;
	if (cmdline_parser(argc, argv, &args_info) != 0)
	{
		cerr << "failed parsing command line arguments" << endl;
		return EXIT_FAILURE;
	}
	args_nkeys = args_info.nkeys_arg;
	args_nplains = args_info.nplains_arg;
	args_nthreads = args_info.nthreads_arg;
	cmdline_parser_free (&args_info);

	cout << "Start random experiments for correlations with " << args_nkeys << " keys." << endl;
	cout << "Each correlation is experimentally computed over " << args_nplains << " plain/ciphertext pairs." << endl;
	cout << "Use " << args_nthreads << " threads for experiments." << endl;
	cout << endl;

	uint64_t alpha = 0, beta = 0;

	vector<future<pair<map<double, double>, map<double, double>>>> future_maps;
	for (int i=0; i<args_nthreads; ++i) {
		future_maps.push_back(async(launch::async, check_keys, alpha, beta));
	}

	map<double, double> histoindp, histoconst;
	for (auto & maps : future_maps) {
		auto mappair = maps.get();
		for (auto const& entry : mappair.first) {
			histoindp[entry.first] += entry.second;
		}
		for (auto const& entry : mappair.second) {
			histoindp[entry.first] += entry.second;
		}
	}

	cout << "histoindp:" << endl;
	for (auto const& entry : histoindp) {
		cout << entry.first << ": " << entry.second << endl;
	}

	cout << "histoconst:" << endl;
	for (auto const& entry : histoconst) {
		cout << entry.first << ": " << entry.second << endl;
	}

	// TODO write histograms

	return EXIT_SUCCESS;
}

pair<map<double, double>, map<double, double>> check_keys(uint64_t alpha, uint64_t beta) {
	return make_pair<map<double, double>, map<double, double>>(
			experiment<Independent_Key, NROUNDS>(alpha, beta),
			experiment<Constant_Key, NROUNDS>(alpha, beta)
		);
}

template<template<size_t> class KEY_T, size_t NR>
map<double, double> experiment(uint64_t alpha, uint64_t beta) {
	std::random_device rd;
	static thread_local std::mt19937 prng(rd());
	std::uniform_int_distribution<uint64_t> dist;

	map<double, double> histo;

	for (size_t i=0; i<ceil(args_nkeys/(double) args_nthreads); ++i) {
		KEY_T<NR> expanded_key;
		double ctr = 0;
		for (size_t j=0; j<ceil(args_nplains/64.0); ++j) {
			// TODO generate random plaintexts
			array<uint64_t, 64> plains;
			for (auto & p : plains)
				p = dist(prng);
			array<uint64_t, 64> cipher(plains);
			//present_encrypt(cipher.data(), expanded_key.data(), NR);
			ctr += __builtin_popcount(!((plains[alpha] ^ cipher[beta]) >> 32));
			ctr += __builtin_popcount(!((plains[alpha] ^ cipher[beta]) && 0xffffffff));
		}
		histo[2 * (ctr / args_nplains - 0.5)] += 1;
	}

	return histo;
}

void check_old() {
	// TODO
	// remove old present test code
	const size_t ntrials = 1;

	uint64_t plaintexts[64];
	uint64_t ciphertexts[64];
	uint64_t tmp[64];
	uint64_t key[80];

	size_t nr = 31;

	for (size_t i=0;i<80;i++)
		key[i] = 0;
	for (size_t i=0; i<64; i++)
		plaintexts[i] = 0;

	uint64_t *subkeys = (uint64_t *)calloc(64 * (nr+1), sizeof(uint64_t));
	present_keyschedule(subkeys, key, nr);

	transpose(tmp, plaintexts, 64, 64);

	for (size_t i=0; i<ntrials; ++i)
		present_encrypt(tmp, subkeys, nr);

	transpose(ciphertexts, tmp, 64, 64);

	cout << "We expect to get the same result in every encryption:" << endl;;
	if (ntrials==1)
		cout << "0000000000000000 5579C1387B228445" << endl << endl;
	for (size_t i=0; i<2; i++)
	{
		cout << hex << setfill('0') << setw(16)
			 << mirror64(plaintexts[i]) << " " << mirror64(ciphertexts[i]) << endl;
	}

	free(subkeys);
	// TODO old present test code end
}

