use ark_bn254::Fr;
use ark_ff::PrimeField;
use num_bigint::BigUint;

pub mod public_keys;
pub mod reveals;
pub mod signatures;
pub mod unmask;

pub fn get_divisor() -> (Fr, BigUint) {
    let m_bytes = [
        6, 12, 137, 206, 92, 38, 52, 5, 55, 10, 8, 182, 208, 48, 43, 11, 171, 62, 237, 184, 57, 32,
        238, 10, 103, 114, 151, 220, 57, 33, 38, 241,
    ];
    let m_field = Fr::from_be_bytes_mod_order(&m_bytes);
    let m = BigUint::from_bytes_be(&m_bytes);

    (m_field, m)
}
