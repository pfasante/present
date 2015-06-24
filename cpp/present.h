#ifndef __PRESENT_H__
#define __PRESENT_H__

static const uint64_t NUM_THREADS = 32;
static const uint64_t NUM_TRAILS[] = {
    1, 1, 1, 3, 9, 27, 72, 192, 512, 1344, 3528, 9261, 24255, 63525, 166375, 435600,
    1140480, 2985984, 7817472, 20466576, 53582633, 140281323, 367261713, 961504803,
    2517252696, 6590254272, 17253512704, 45170283840, 118257341400, 309601747125,
    810547899975};
static const uint64_t input_indices[] = {21};
static const uint64_t output_indices[] = {21};

static const uint64_t NUM_MASKS  = sizeof(input_indices) / sizeof(uint64_t);
static const uint64_t NUM_ROUNDS = 5;
static const uint64_t NUM_KEYS   = 20000;
static const size_t NUM_KEYS_PER_THREAD = NUM_KEYS / NUM_THREADS;
static const uint64_t PRECISION  = 5;

// COMPLEXITY = ((1/c^2) / N_T)
static const uint64_t COMPLEXITY  = ((1 << (4*NUM_ROUNDS)) / NUM_TRAILS[NUM_ROUNDS-1]);
static const uint64_t NUM_PLAIN = COMPLEXITY * 64;

// we need 64 round keys at the same time,
// because we use a bitsliced present implementation
typedef std::array<uint64_t, 64*(NUM_ROUNDS+1)> round_keys_t;
typedef std::array<round_keys_t, NUM_KEYS_PER_THREAD> keys_t;
typedef std::array<std::map<double, int>, NUM_MASKS> corr_histo_t;

typedef std::shared_ptr<round_keys_t> p_round_keys_t;
typedef std::shared_ptr<keys_t> p_keys_t;
typedef std::shared_ptr<corr_histo_t> p_corr_histo_t;

#endif  // __PRESENT_H__

