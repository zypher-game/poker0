use ark_ec::CurveGroup;
use ark_ed_on_bn254::{EdwardsAffine, Fq};
use ark_ff::PrimeField;
use ark_serialize::{Compress, Validate};
use rand_chacha::{
    rand_core::{CryptoRng, RngCore, SeedableRng},
    ChaChaRng,
};
use std::fmt::Display;
use wasm_bindgen::prelude::*;

#[inline(always)]
pub(crate) fn error_to_jsvalue<T: Display>(e: T) -> JsValue {
    JsValue::from_str(&e.to_string())
}

pub fn default_prng() -> impl RngCore + CryptoRng {
    ChaChaRng::from_entropy()
}

pub fn hex_to_scalar<F: PrimeField>(hex: &str) -> Result<F, JsValue> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex).map_err(error_to_jsvalue)?;
    if bytes.len() != 32 {
        return Err(error_to_jsvalue("Bytes length not 32"));
    }
    Ok(F::from_be_bytes_mod_order(&bytes))
}

pub fn hex_to_point<G: CurveGroup>(hex: &str) -> Result<G, JsValue> {
    let hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(hex).map_err(error_to_jsvalue)?;
    G::deserialize_with_mode(bytes.as_slice(), Compress::Yes, Validate::Yes)
        .map_err(error_to_jsvalue)
}

pub fn uncompress_to_point(x_str: &str, y_str: &str) -> Result<EdwardsAffine, JsValue> {
    let x_hex = x_str.trim_start_matches("0x");
    let y_hex = y_str.trim_start_matches("0x");
    let x_bytes = hex::decode(x_hex).map_err(error_to_jsvalue)?;
    let y_bytes = hex::decode(y_hex).map_err(error_to_jsvalue)?;

    let x = Fq::from_be_bytes_mod_order(&x_bytes);
    let y = Fq::from_be_bytes_mod_order(&y_bytes);
    let affine = EdwardsAffine::new(x, y);

    Ok(affine)
}
