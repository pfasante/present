#ifndef __keyschedule_h__
#define __keyschedule_h__

#include <array>
#include <chrono>
#include <cstdlib>
#include <random>
#include <thread>

class Expanded_Key
{
public:
	virtual std::uint64_t operator[](std::size_t idx) const = 0;
	virtual std::uint64_t const operator[](std::size_t idx) = 0;
};

template <std::size_t NR>
class Independent_Key : Expanded_Key
{
public:
	Independent_Key();
	~Independent_Key() {};

	std::uint64_t operator[](std::size_t idx) const { return expanded_keys[idx]; };
	std::uint64_t const operator[](std::size_t idx) { return expanded_keys[idx]; };

private:
	std::array<std::uint64_t, NR+1> expanded_keys;
};

template <std::size_t NR>
Independent_Key<NR>::Independent_Key()
	: expanded_keys()
{
	std::random_device rd;

	static thread_local std::mt19937 prng(rd());
	std::uniform_int_distribution<uint64_t> dist;
	for (auto & key : expanded_keys)
		key = dist(prng);
}

template <std::size_t NR>
class Constant_Key : Expanded_Key
{
public:
	Constant_Key();
	~Constant_Key() {};

	std::uint64_t operator[](std::size_t idx) const { return key; }
	std::uint64_t const operator[](std::size_t idx) { return key; }

private:
	std::uint64_t key;
};

template <std::size_t NR>
Constant_Key<NR>::Constant_Key()
{
	std::random_device rd;

	static thread_local std::mt19937 prng(rd());
	std::uniform_int_distribution<uint64_t> dist;
	key = dist(prng);
}

#endif  // __keyschedule_h__

