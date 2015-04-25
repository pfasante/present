extern crate present;
use present::*;

#[test]
fn test_keyschedule()
{
    let mut k = KeyT::new(0x0, 0x0);
    k.keyschedule(31);
    assert_eq!(SubkeyT::new(0x0000000000000000, 0x0000), k[0]);
    assert_eq!(SubkeyT::new(0xc000000000000000, 0x8000), k[1]);
    assert_eq!(SubkeyT::new(0x5000180000000001, 0x0000), k[2]);
    assert_eq!(SubkeyT::new(0x60000a0003000001, 0x8000), k[3]);
    assert_eq!(SubkeyT::new(0xb0000c0001400062, 0x0000), k[4]);
    assert_eq!(SubkeyT::new(0x900016000180002a, 0x800c), k[5]);
    assert_eq!(SubkeyT::new(0x0001920002c00033, 0x0005), k[6]);
    assert_eq!(SubkeyT::new(0xa000a0003240005b, 0x8006), k[7]);
    assert_eq!(SubkeyT::new(0xd000d4001400064c, 0x000b), k[8]);
    assert_eq!(SubkeyT::new(0x30017a001a800284, 0x80c9), k[9]);
    assert_eq!(SubkeyT::new(0xe01926002f400355, 0x0050), k[10]);
    assert_eq!(SubkeyT::new(0xf00a1c0324c005ed, 0x806a), k[11]);
    assert_eq!(SubkeyT::new(0x800d5e014380649e, 0x00bd), k[12]);
    assert_eq!(SubkeyT::new(0x4017b001abc02876, 0x8c93), k[13]);
    assert_eq!(SubkeyT::new(0x71926802f600357f, 0x050e), k[14]);
    assert_eq!(SubkeyT::new(0x10a1ce324d005ec7, 0x86af), k[15]);
    assert_eq!(SubkeyT::new(0x20d5e21439c649a8, 0x0bd8), k[16]);
    assert_eq!(SubkeyT::new(0xc17b041abc428730, 0x4935), k[17]);
    assert_eq!(SubkeyT::new(0xc926b82f60835781, 0x50e6), k[18]);
    assert_eq!(SubkeyT::new(0x6a1cd924d705ec19, 0xeaf0), k[19]);
    assert_eq!(SubkeyT::new(0xbd5e0d439b249aea, 0xbd83), k[20]);
    assert_eq!(SubkeyT::new(0x07b077abc1a8736e, 0x135d), k[21]);
    assert_eq!(SubkeyT::new(0x426ba0f60ef5783e, 0x0e6d), k[22]);
    assert_eq!(SubkeyT::new(0x41cda84d741ec1d5, 0x2f07), k[23]);
    assert_eq!(SubkeyT::new(0xf5e0e839b509ae8f, 0xd83a), k[24]);
    assert_eq!(SubkeyT::new(0x2b075ebc1d0736ad, 0xb5d1), k[25]);
    assert_eq!(SubkeyT::new(0x86ba2560ebd783ad, 0xe6d5), k[26]);
    assert_eq!(SubkeyT::new(0x8cdab0d744ac1d77, 0x7075), k[27]);
    assert_eq!(SubkeyT::new(0x1e0eb19b561ae89b, 0x83ae), k[28]);
    assert_eq!(SubkeyT::new(0xd075c3c1d6336acd, 0xdd13), k[29]);
    assert_eq!(SubkeyT::new(0x8ba27a0eb8783ac9, 0x6d59), k[30]);
}

