use std::fmt;

pub struct SubkeyT {
    high : u64,
    low : u16
}

impl Clone for SubkeyT {
    fn clone(&self) -> Self
    {
        SubkeyT {high: self.high, low: self.low}
    }
}

impl PartialEq for SubkeyT {
    fn eq(&self, other: &SubkeyT) -> bool
    {
        (self.high == other.high) && (self.low == other.low)
    }
}

impl fmt::Display for SubkeyT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Subkey State: 0x{:x}, 0x{:x}", self.high, self.low)
    }
}

impl fmt::Debug for SubkeyT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Subkey State: 0x{:x}, 0x{:x}", self.high, self.low)
    }
}

pub struct KeyT {
    keys: Vec<SubkeyT>
}

/// the PRESENT state is 64 bit long
pub struct StateT {
    state : u64
}

impl fmt::Display for StateT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "State: 0x{:x}", self.state)
    }
}

/// returns s(input), where s is the PRESENT sbox
/// and s is applied to both, only the left (msb) or only the right (lsb) nibble
///
/// # Example
///
/// ```
/// assert_eq!(present::sbox_both(0xf0), 0x2c);
/// assert_eq!(present::sbox_msb(0xf0), 0x20);
/// assert_eq!(present::sbox_lsb(0xf0), 0xfc);
/// ```
pub fn sbox_both(input : u8) -> u8
{
    let s = [0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
             0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2];

    s[(input & 0xf) as usize] | (s[(input >> 4) as usize] << 4)
}
pub fn sbox_msb(input : u8) -> u8
{
    let s = [0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
             0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2];

    (input & 0xf) | (s[(input >> 4) as usize] << 4)
}
pub fn sbox_lsb(input : u8) -> u8
{
    let s = [0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd,
             0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2];

    s[(input & 0xf) as usize] | (input & 0xf0)
}

/// for a given key, run the PRESENT keyschedule and return subkeys for n rounds
pub fn keyschedule(key : &mut KeyT, rounds : usize)
{
    // generate rounds + 1 keys, from key[0]
    let mut high;
    let mut low;
    for i in 1..(rounds + 1)
    {
        // rotate the key register by 61 bit positions to the left
        // or rotate the key register by 19 bit positions to the right
        low = (((*key).keys[i - 1].high >> 3) & 0xffff) as u16;
        high = ((*key).keys[i - 1].high >> 19) |
               (((*key).keys[i - 1].low as u64) << 45) |
               (((*key).keys[i - 1].high & 0b111) << 61);
        // pass the leftmost nibble through the sbox
        high = (high & 0x0fff_ffff_ffff_ffff) | ((sbox_lsb((high >> 60) as u8) as u64) << 60);
        // xor the round_counter to bits 19-15
        low = low ^ (((i & 0b1) << 15) as u16);
        high = high ^ ((i >> 1) as u64);
        (*key).keys.push(SubkeyT {high: high, low: low});
    }
}

#[cfg(test)]
mod test {
    use super::{SubkeyT, KeyT, keyschedule};

    #[test]
    fn test_keyschedule()
    {
        let mut k = KeyT {keys: vec!(SubkeyT {high: 0x0, low: 0x0})};
        keyschedule(&mut k, 31);
        assert_eq!(SubkeyT {high: 0x0000000000000000, low: 0x0000}, k.keys[0]);
        assert_eq!(SubkeyT {high: 0xc000000000000000, low: 0x8000}, k.keys[1]);
        assert_eq!(SubkeyT {high: 0x5000180000000001, low: 0x0000}, k.keys[2]);
        assert_eq!(SubkeyT {high: 0x60000a0003000001, low: 0x8000}, k.keys[3]);
        assert_eq!(SubkeyT {high: 0xb0000c0001400062, low: 0x0000}, k.keys[4]);
        assert_eq!(SubkeyT {high: 0x900016000180002a, low: 0x800c}, k.keys[5]);
        assert_eq!(SubkeyT {high: 0x0001920002c00033, low: 0x0005}, k.keys[6]);
        assert_eq!(SubkeyT {high: 0xa000a0003240005b, low: 0x8006}, k.keys[7]);
        assert_eq!(SubkeyT {high: 0xd000d4001400064c, low: 0x000b}, k.keys[8]);
        assert_eq!(SubkeyT {high: 0x30017a001a800284, low: 0x80c9}, k.keys[9]);
        assert_eq!(SubkeyT {high: 0xe01926002f400355, low: 0x0050}, k.keys[10]);
        assert_eq!(SubkeyT {high: 0xf00a1c0324c005ed, low: 0x806a}, k.keys[11]);
        assert_eq!(SubkeyT {high: 0x800d5e014380649e, low: 0x00bd}, k.keys[12]);
        assert_eq!(SubkeyT {high: 0x4017b001abc02876, low: 0x8c93}, k.keys[13]);
        assert_eq!(SubkeyT {high: 0x71926802f600357f, low: 0x050e}, k.keys[14]);
        assert_eq!(SubkeyT {high: 0x10a1ce324d005ec7, low: 0x86af}, k.keys[15]);
        assert_eq!(SubkeyT {high: 0x20d5e21439c649a8, low: 0x0bd8}, k.keys[16]);
        assert_eq!(SubkeyT {high: 0xc17b041abc428730, low: 0x4935}, k.keys[17]);
        assert_eq!(SubkeyT {high: 0xc926b82f60835781, low: 0x50e6}, k.keys[18]);
        assert_eq!(SubkeyT {high: 0x6a1cd924d705ec19, low: 0xeaf0}, k.keys[19]);
        assert_eq!(SubkeyT {high: 0xbd5e0d439b249aea, low: 0xbd83}, k.keys[20]);
        assert_eq!(SubkeyT {high: 0x07b077abc1a8736e, low: 0x135d}, k.keys[21]);
        assert_eq!(SubkeyT {high: 0x426ba0f60ef5783e, low: 0x0e6d}, k.keys[22]);
        assert_eq!(SubkeyT {high: 0x41cda84d741ec1d5, low: 0x2f07}, k.keys[23]);
        assert_eq!(SubkeyT {high: 0xf5e0e839b509ae8f, low: 0xd83a}, k.keys[24]);
        assert_eq!(SubkeyT {high: 0x2b075ebc1d0736ad, low: 0xb5d1}, k.keys[25]);
        assert_eq!(SubkeyT {high: 0x86ba2560ebd783ad, low: 0xe6d5}, k.keys[26]);
        assert_eq!(SubkeyT {high: 0x8cdab0d744ac1d77, low: 0x7075}, k.keys[27]);
        assert_eq!(SubkeyT {high: 0x1e0eb19b561ae89b, low: 0x83ae}, k.keys[28]);
        assert_eq!(SubkeyT {high: 0xd075c3c1d6336acd, low: 0xdd13}, k.keys[29]);
        assert_eq!(SubkeyT {high: 0x8ba27a0eb8783ac9, low: 0x6d59}, k.keys[30]);
    }
}
