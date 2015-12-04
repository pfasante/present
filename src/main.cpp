#include <cstdint>

#include <iostream>
#include <iomanip>

#include "present_bitslice.h"
#include "cmdline.h"

using namespace std;

long args_nkeys;
long args_nplains;
int args_nthreads;

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

	cmdline_parser_free (&args_info); // release allocated memory
	return 0;
}
