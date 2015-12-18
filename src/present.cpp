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

template<typename Sbox>
pair<map<double, double>, map<double, double>>
check_keys(uint64_t alpha, uint64_t beta);

template<class Sbox, template<size_t> class KEY_T, size_t NR>
map<double, double>
experiment(uint64_t alpha, uint64_t beta);

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

	cout << "Start random experiments for correlations." << endl;
	cout << "Parameter:" << endl;
	cout << "\t" << args_nkeys << " keys are tested" << endl;
	cout << "\t" << args_nplains << " plaintexts used for correlation computation" << endl;
	cout << "\t" << args_nthreads << " threads for splitting workload of experiments" << endl;
	cout << endl;

	uint64_t alpha = 21, beta = 21;

	// start threads to run experiments
	vector<future<pair<map<double, double>, map<double, double>>>> future_maps;
	for (int i=0; i<args_nthreads; ++i) {
		future_maps.push_back(async(launch::async, check_keys<Sbox_Present>, alpha, beta));
	}

	// get histograms from threads and accumulate them
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

	// output accumulated histograms
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

/**
 * check_keys
 * \brief runs experiments for indpendent and constant round keys
 */
template<typename Sbox>
pair<map<double, double>, map<double, double>> check_keys(uint64_t alpha, uint64_t beta) {
	return make_pair<map<double, double>, map<double, double>>(
			experiment<Sbox, Independent_Key, NROUNDS>(alpha, beta),
			experiment<Sbox, Constant_Key, NROUNDS>(alpha, beta)
		);
}

template<typename Sbox, template<size_t> class KEY_T, size_t NR>
map<double, double> experiment(uint64_t alpha, uint64_t beta) {
	std::random_device rd;
	static thread_local std::mt19937 prng(rd());
	std::uniform_int_distribution<uint64_t> dist;

	map<double, double> histo;

	for (size_t i=0; i<ceil(args_nkeys/(double) args_nthreads); ++i) {
		KEY_T<NR> expanded_key;
		double ctr = 0;
		for (size_t j=0; j<ceil(args_nplains/64.0); ++j) {
			array<uint64_t, 64> plains;
			for (auto & p : plains)
				p = dist(prng);
			array<uint64_t, 64> cipher(plains);

			present_encrypt<Sbox>(cipher, expanded_key.data(), NR);

			ctr += __builtin_popcount(!((plains[alpha] ^ cipher[beta]) >> 32));
			ctr += __builtin_popcount(!((plains[alpha] ^ cipher[beta]) && 0xffffffff));
		}
		histo[2 * (ctr / args_nplains - 0.5)] += 1;
	}

	return histo;
}

