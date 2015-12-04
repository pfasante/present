#ifndef __present_bitslice_h__
#define __present_bitslice_h__

#ifdef __cplusplus
extern "C" {
#endif  // __cplusplus

void present_encrypt(uint64_t *X, const uint64_t *subkeys, const size_t nr);
void present_keyschedule(uint64_t *subkeys, uint64_t *key, const size_t nr);

void transpose(uint64_t *out, uint64_t *inp, const size_t out_size, const size_t inp_size);
uint64_t mirror64(uint64_t ins);

#ifdef __cplusplus
}
#endif  // __cplusplus
#endif  // __present_bitslice_h__

