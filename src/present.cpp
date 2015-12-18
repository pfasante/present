#include <cmath>
#include <cstdint>
#include <cstdlib>

#include <iostream>
#include <iomanip>
#include <future>
#include <fstream>
#include <map>
#include <thread>
#include <utility>
#include <vector>

#include "cmdline.h"
#include "keyschedule.h"

#include "present_bitslice.h"

using namespace std;

const size_t NROUNDS = 5;
size_t args_nkeys;
size_t args_nplains;
size_t args_nthreads;
size_t args_key_per_thread;

using Histo = map<double, double>;

template<typename Sbox>
pair<Histo, Histo> check_keys(uint64_t alpha, uint64_t beta);

template<class Sbox, template<size_t> class KEY_T, size_t NR>
Histo experiment(uint64_t alpha, uint64_t beta);

void write_histo(string const& filename, Histo const& histo);

int main(int argc, char **argv) {
	gengetopt_args_info args_info;
	if (cmdline_parser(argc, argv, &args_info) != 0)
	{
		cerr << "failed parsing command line arguments" << endl;
		return EXIT_FAILURE;
	}
	args_nthreads = args_info.nthreads_arg;
	args_nplains = ceil((double)args_info.nplains_arg/64.0) * 64;
	args_nkeys = ceil(args_info.nkeys_arg/(double)args_nthreads) * args_nthreads;
	args_key_per_thread = args_nkeys/args_nthreads;
	cmdline_parser_free (&args_info);

	// parameter for  5 rounds: 20000 keys,   16777216 plains and 8 threads
	// parameter for 10 rounds: 20000 keys, 1073741824 plains and 8 threads
	cout << "Start random experiments for correlations." << endl;
	cout << "Parameter:" << endl;
	cout << "\t" << args_nkeys << " keys are tested" << endl;
	cout << "\t" << args_nplains << " plaintexts used for correlation computation" << endl;
	cout << "\t" << args_nthreads << " threads for splitting workload of experiments" << endl;
	cout << endl;

	uint64_t alpha = 21, beta = 21;

	// start threads to run experiments
	vector<future<pair<Histo, Histo>>> future_maps;
	for (size_t i=0; i<args_nthreads; ++i) {
		future_maps.push_back(async(launch::async, check_keys<Sbox_Present>, alpha, beta));
	}

	// get histograms from threads and accumulate them
	Histo histoindp, histoconst;
	for (auto & maps : future_maps) {
		auto mappair = maps.get();
		for (auto const& entry : mappair.first) {
			histoindp[entry.first] += entry.second;
		}
		for (auto const& entry : mappair.second) {
			histoconst[entry.first] += entry.second;
		}
	}

	// output accumulated histograms
	cout << "writing histoindp..." << endl;
	write_histo("histoindp.dat", histoindp);

	cout << "writing histoconst..." << endl;
	write_histo("histoconst.dat", histoconst);

	return EXIT_SUCCESS;
}

/**
 * check_keys
 * \brief runs experiments for indpendent and constant round keys
 */
template<typename Sbox>
pair<Histo, Histo> check_keys(uint64_t alpha, uint64_t beta) {
	return make_pair(
			experiment<Sbox, Independent_Key, NROUNDS>(alpha, beta),
			experiment<Sbox, Constant_Key, NROUNDS>(alpha, beta)
		);
}

template<typename Sbox, template<size_t> class KEY_T, size_t NR>
Histo experiment(uint64_t alpha, uint64_t beta) {
	std::random_device rd;
	static thread_local std::mt19937 prng(rd());
	std::uniform_int_distribution<uint64_t> dist;

	Histo histo;

	for (size_t i=0; i<args_key_per_thread; ++i) {
		KEY_T<NR> expanded_key;
		double ctr = 0;
		for (size_t j=0; j<args_nplains; j+=64) {
			array<uint64_t, 64> plains;
			for (auto & p : plains)
				p = dist(prng);
			array<uint64_t, 64> cipher(plains);

			present_encrypt<Sbox>(cipher, expanded_key.data(), NR);

			ctr += __builtin_popcount(~((plains[alpha] ^ cipher[beta]) >> 32));
			ctr += __builtin_popcount(~((plains[alpha] ^ cipher[beta]) && 0xffffffff));
		}
		double correlation = 2*(ctr/(double)args_nplains) - 1;
		histo[correlation] += 1;
	}

	return histo;
}

void write_histo(string const& filename, Histo const& histo) {
	ofstream out(filename);
	for (auto const& entry : histo) {
		out << setprecision(16) << entry.first << " " << entry.second << endl;
	}
}

