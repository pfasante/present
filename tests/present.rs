extern crate linc;
extern crate rand;
use linc::*;
use linc::present::*;
use rand::Rand;

#[test]
fn test_keyschedule() {
    let k = PresentKeySchedule::new(PresentCipherKey::new(0x0, 0x0), 31);
    assert_eq!(PresentRoundKey::new(0x0000_0000_0000_0000), k[0]);
    assert_eq!(PresentRoundKey::new(0xc000_0000_0000_0000), k[1]);
    assert_eq!(PresentRoundKey::new(0x5000_1800_0000_0001), k[2]);
    assert_eq!(PresentRoundKey::new(0x6000_0a00_0300_0001), k[3]);
    assert_eq!(PresentRoundKey::new(0xb000_0c00_0140_0062), k[4]);
    assert_eq!(PresentRoundKey::new(0x9000_1600_0180_002a), k[5]);
    assert_eq!(PresentRoundKey::new(0x0001_9200_02c0_0033), k[6]);
    assert_eq!(PresentRoundKey::new(0xa000_a000_3240_005b), k[7]);
    assert_eq!(PresentRoundKey::new(0xd000_d400_1400_064c), k[8]);
    assert_eq!(PresentRoundKey::new(0x3001_7a00_1a80_0284), k[9]);
    assert_eq!(PresentRoundKey::new(0xe019_2600_2f40_0355), k[10]);
    assert_eq!(PresentRoundKey::new(0xf00a_1c03_24c0_05ed), k[11]);
    assert_eq!(PresentRoundKey::new(0x800d_5e01_4380_649e), k[12]);
    assert_eq!(PresentRoundKey::new(0x4017_b001_abc0_2876), k[13]);
    assert_eq!(PresentRoundKey::new(0x7192_6802_f600_357f), k[14]);
    assert_eq!(PresentRoundKey::new(0x10a1_ce32_4d00_5ec7), k[15]);
    assert_eq!(PresentRoundKey::new(0x20d5_e214_39c6_49a8), k[16]);
    assert_eq!(PresentRoundKey::new(0xc17b_041a_bc42_8730), k[17]);
    assert_eq!(PresentRoundKey::new(0xc926_b82f_6083_5781), k[18]);
    assert_eq!(PresentRoundKey::new(0x6a1c_d924_d705_ec19), k[19]);
    assert_eq!(PresentRoundKey::new(0xbd5e_0d43_9b24_9aea), k[20]);
    assert_eq!(PresentRoundKey::new(0x07b0_77ab_c1a8_736e), k[21]);
    assert_eq!(PresentRoundKey::new(0x426b_a0f6_0ef5_783e), k[22]);
    assert_eq!(PresentRoundKey::new(0x41cd_a84d_741e_c1d5), k[23]);
    assert_eq!(PresentRoundKey::new(0xf5e0_e839_b509_ae8f), k[24]);
    assert_eq!(PresentRoundKey::new(0x2b07_5ebc_1d07_36ad), k[25]);
    assert_eq!(PresentRoundKey::new(0x86ba_2560_ebd7_83ad), k[26]);
    assert_eq!(PresentRoundKey::new(0x8cda_b0d7_44ac_1d77), k[27]);
    assert_eq!(PresentRoundKey::new(0x1e0e_b19b_561a_e89b), k[28]);
    assert_eq!(PresentRoundKey::new(0xd075_c3c1_d633_6acd), k[29]);
    assert_eq!(PresentRoundKey::new(0x8ba2_7a0e_b878_3ac9), k[30]);
}

#[test]
/// Testvectors from the PRESENT paper
fn test_encryption() {
    println!("Checking testvectors from PRESENT paper");

    let c = Present::new(PresentCipherKey::new(0x0, 0x0),
                         31)
                    .enc(0x0, 31);
    assert_eq!(0x5579c138_7b228445, c);
    println!("TV1 correct");
    let c = Present::new(PresentCipherKey::new(0xffffffff_ffffffff, 0xffff),
                         31)
                    .enc(0x0, 31);
    assert_eq!(0xe72c46c0_f5945049, c);
    println!("TV2 correct");

    let c = Present::new(PresentCipherKey::new(0x0, 0x0),
                         31)
                    .enc(0xffffffff_ffffffff, 31);
    assert_eq!(0xa112ffc7_2f68417b, c);
    println!("TV3 correct");

    let c = Present::new(PresentCipherKey::new(0xffffffff_ffffffff, 0xffff),
                         31)
                    .enc(0xffffffff_ffffffff, 31);
    assert_eq!(0x3333dcd3_213210d2, c);
    println!("TV4 correct");
}

#[test]
fn test_enc_dec_is_id() {
    println!("Check if decryption(encryption(m)) == m");
    let mut rng = rand::thread_rng();
    let cipher = Present::new(PresentCipherKey::rand(&mut rng), 31);
    for _ in 0..1000 {
        let m = u64::rand(&mut rng);
        let c = cipher.enc(m, 31);
        assert_eq!(m, cipher.dec(c, 31));
    }
}

