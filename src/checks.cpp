#include <iostream>
#include <iomanip>
#include <ios>

#include "present_bitslice.h"

using namespace std;

int main(int argc, char **argv) {
	// check SBOX:
	for (size_t i=0; i<16; ++i) {
		uint64_t x0, x1, x2, x3;
		uint64_t y0, y1, y2, y3;

		x0 = -((i >> 0) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1
		x1 = -((i >> 1) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1
		x2 = -((i >> 2) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1
		x3 = -((i >> 3) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1

		Sbox_Present()(y3, y2, y1, y0, x3, x2, x1, x0);

		x0 = x0 > 0 ? 1 : 0;
		x1 = x1 > 0 ? 1 : 0;
		x2 = x2 > 0 ? 1 : 0;
		x3 = x3 > 0 ? 1 : 0;

		y0 = y0 > 0 ? 1 : 0;
		y1 = y1 > 0 ? 1 : 0;
		y2 = y2 > 0 ? 1 : 0;
		y3 = y3 > 0 ? 1 : 0;

		uint64_t output = (y3 << 3) + (y2 << 2) + (y1 << 1) + (y0 << 0);
		cout << "SboxPresent(" << x3 << x2 << x1 << x0 << ") = " << y3 << y2 << y1 << y0;
		cout << " = " << hex << output << dec << " = " << output << endl;
	}
	cout << endl;
	for (size_t i=0; i<16; ++i) {
		uint64_t x0, x1, x2, x3;
		uint64_t y0, y1, y2, y3;

		x0 = -((i >> 0) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1
		x1 = -((i >> 1) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1
		x2 = -((i >> 2) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1
		x3 = -((i >> 3) & 0x1); // convert bit 0 of ctr to 0, if it is 0 or to 0xff...ff if it is 1

		Sbox_R2()(y3, y2, y1, y0, x3, x2, x1, x0);

		x0 = x0 > 0 ? 1 : 0;
		x1 = x1 > 0 ? 1 : 0;
		x2 = x2 > 0 ? 1 : 0;
		x3 = x3 > 0 ? 1 : 0;

		y0 = y0 > 0 ? 1 : 0;
		y1 = y1 > 0 ? 1 : 0;
		y2 = y2 > 0 ? 1 : 0;
		y3 = y3 > 0 ? 1 : 0;

		uint64_t output = (y3 << 3) + (y2 << 2) + (y1 << 1) + (y0 << 0);
		cout << "SboxR2     (" << x3 << x2 << x1 << x0 << ") = " << y3 << y2 << y1 << y0;
		cout << " = " << hex << output << dec << " = " << output << endl;
	}

	return EXIT_SUCCESS;
}

