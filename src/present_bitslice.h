#ifndef __present_bitslice_h__
#define __present_bitslice_h__

void encrypt(uint64_t *X, const uint64_t *subkeys, const size_t nr);
void key_schedule(uint64_t *subkeys, uint64_t *key, const size_t nr);

void transpose(uint64_t *out, uint64_t *inp, const size_t out_size, const size_t inp_size);
uint64_t Mirror64(uint64_t ins);

#endif  // __present_bitslice_h__

