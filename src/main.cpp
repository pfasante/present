#include <cstdint>

#include <iostream>
#include <iomanip>

extern "C" {
#include "present_bitslice.h"
}

using namespace std;

int main(int arc, char **argv) {
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
	return 0;
}
