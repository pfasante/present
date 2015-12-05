#include <cstdint>
#include <cstdlib>

#include <iostream>
#include <iomanip>
#include <future>
#include <mutex>
#include <thread>
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

template<template<size_t> class KEY_T, size_t NR>
void check_keys(size_t alpha, size_t beta);

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

	cout << "called with args:" << endl;
	cout << "\tnkeys = " << args_nkeys << endl;
	cout << "\tnplains = " << args_nplains << endl;
	cout << "\tnthreads = " << args_nthreads << endl;

	size_t alpha = 0, beta = 0;

	vector<future<void>> future_indpmaps;
	for (int i=0; i<args_nthreads; ++i) {
		future_indpmaps.push_back(async(launch::async,
			check_keys<Independent_Key, NROUNDS>, alpha, beta
			));
	}

	vector<future<void>> future_constmaps;
	for (int i=0; i<args_nthreads; ++i) {
		future_constmaps.push_back(async(launch::async,
			check_keys<Constant_Key, NROUNDS>, alpha, beta
			));
	}

	for (auto & map : future_indpmaps)
		map.get();
	for (auto & map : future_constmaps)
		map.get();

	// TODO
	// alpha, beta, nkeys_perthread = ceil(nkeys / nthreads)
	// for nthreads do in parallel
		// generate nkeys_perthread independent/constant expanded keys
			// => class expanded_key
			// independent_expanded_key quasi array<uint64_t, nkeys*nrounds>
			// constant_expanded_key quasi uint64_t
			// both overloads operator[] for convenient access
		// for every key
			// encrypt ceil(nplains/64)*64 random plain/ciphertext pairs to compute bias
			// update histogram map accordingly
		// return histogram
	// join histograms
	//
	// static thread_local std::mt19937 generator;
	// std::uniform_int_distribution<int> distribution(min,max);
	// return distribution(generator);

	cmdline_parser_free (&args_info); // release allocated memory
	return 0;
}

template<template<size_t> class KEY_T, size_t NR>
void check_keys(size_t alpha, size_t beta) {
	KEY_T<NR> expanded_key;

	{	// TODO debug output
		lock_guard<mutex> lock(mut_cout);
		cout << this_thread::get_id() << ":" << endl;
		for (size_t i=0; i<NROUNDS+1; ++i) {
			cout << "round " << i << ": key = ";
			cout << hex << setfill('0') << setw(16) << expanded_key[i] << endl;
		}
		cout << endl;
	}
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

