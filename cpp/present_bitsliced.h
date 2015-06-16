#ifndef __PRESENT_BITSLICED_H__
#define __PRESENT_BITSLICED_H__

#ifdef _cplusplus
extern "C" {
#endif

void transpose(uint64_t *out, uint64_t const* inp,
        size_t const out_size, size_t const inp_size);
void key_schedule(uint64_t *subkeys, uint64_t *key, size_t const nr);
void encrypt(uint64_t *X, uint64_t const* subkeys, size_t const nr);

#ifdef _cplusplus
}
#endif

#endif
