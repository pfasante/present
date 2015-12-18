#ifndef __keyschedule_h__
#define __keyschedule_h__

#include <array>
#include <chrono>
#include <cstdlib>
#include <random>
#include <thread>
#include <ios>
#include <iomanip>

/** Expanded_Key
 * \brief abstract class to describe an interface for an expanded key
 *        stored in bitslice representation.
 *        Thus we need 64 uint64_t's per round (one round key is one
 *        uint64_t, and this implementation assumes a 64 bit state).
 */
class Expanded_Key
{
public:
	//virtual std::array<std::uint64_t, 64>& operator[](std::size_t idx) = 0;
	//virtual std::array<std::uint64_t, 64> const& operator[](std::size_t idx) const = 0;
	virtual std::uint64_t const* data() = 0;
};


/** Independent_Key
 *
 */
template<std::size_t NR>
class Independent_Key : Expanded_Key
{
public:
	Independent_Key();
	~Independent_Key() {};

	//std::array<std::uint64_t, 64>& operator[](std::size_t idx) { return expanded_keys[idx]; };
	//std::array<std::uint64_t, 64> const& operator[](std::size_t idx) const { return expanded_keys[idx]; };
	std::uint64_t const* data() { return expanded_keys.data(); };

	template<std::size_t NR_>
	friend std::ostream& operator<<(std::ostream& ostr, Independent_Key<NR_> const& key);

private:
	std::array<std::uint64_t, 64 * (NR+1)> expanded_keys;
};

template<std::size_t NR>
Independent_Key<NR>::Independent_Key()
	: expanded_keys()
{
	std::random_device rd;
	static thread_local std::mt19937 prng(rd());
	std::uniform_int_distribution<uint64_t> dist;

	for (size_t k=0; k<64*(NR+1); k+=64)
	{
		uint64_t round_key = dist(prng);
		for (size_t i=0; i<64; ++i)
			// if the key bit is one, set bitsliced_key to 0xff...ff
			expanded_keys[k+i] = -((round_key >> i) & 0x1);
	}
}

template<std::size_t NR>
std::ostream& operator<<(std::ostream& ostr, Independent_Key<NR> const& key)
{
	for (size_t k=0; k<64*(NR+1); k+=64)
	{
		uint64_t round_key = 0;
		for (size_t i=0; i<64; ++i)
			round_key |= ((key.expanded_keys[k+i]) & 0x1) << i;
		ostr << std::setw(16) << std::setfill('0') << std::hex;
		ostr << round_key << " ";
		ostr << std::dec;
	}
	ostr << std::endl;
	return ostr;
}


/** Constant_Key
 *
 */
template<std::size_t NR>
class Constant_Key : Expanded_Key
{
public:
	Constant_Key();
	~Constant_Key() {};

	//std::array<std::uint64_t, 64>& operator[](std::size_t idx) { return expanded_keys[idx]; };
	//std::array<std::uint64_t, 64> const& operator[](std::size_t idx) const { return expanded_keys[idx]; };
	std::uint64_t const* data() { return expanded_keys.data(); };

	template<std::size_t NR_>
	friend std::ostream& operator<<(std::ostream& ostr, Constant_Key<NR_> const& key);

private:
	std::array<std::uint64_t, 64 * (NR+1)> expanded_keys;
};

template<std::size_t NR>
Constant_Key<NR>::Constant_Key()
	: expanded_keys()
{
	std::random_device rd;
	static thread_local std::mt19937 prng(rd());
	std::uniform_int_distribution<uint64_t> dist;

	uint64_t round_key = dist(prng);
	for (size_t k=0; k<64*(NR+1); k+=64)
		for (size_t i=0; i<64; ++i)
			// if the key bit is one, set bitsliced_key to 0xff...ff
			expanded_keys[k+i] = -((round_key >> i) & 0x1);
}

template<std::size_t NR>
std::ostream& operator<<(std::ostream& ostr, Constant_Key<NR> const& key)
{
	uint64_t round_key = 0;
	for (size_t i=0; i<64; ++i)
		// if the key bit is one, set bitsliced_key to 0xff...ff
		round_key |= ((key.expanded_keys[i]) & 0x1) << i;
	ostr << std::setw(16) << std::setfill('0') << std::hex;
	ostr << round_key << " ";
	ostr << std::dec << std::endl;
	return ostr;
}

#endif  // __keyschedule_h__

