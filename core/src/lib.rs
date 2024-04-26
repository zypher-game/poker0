use std::ops::{Mul, Sub};

use ark_ec::{AffineRepr, CurveGroup};
use ark_ed_on_bn254::EdwardsProjective;
use ark_ff::UniformRand;
use ark_std::rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use zplonk::{
    shuffle::Ciphertext,
    utils::serialization::{ark_deserialize, ark_serialize},
};

pub mod cards;
pub mod combination;
pub mod errors;
pub mod mock_data;
pub mod play;
pub mod schnorr;
pub mod task;

#[macro_use]
extern crate lazy_static;

pub type CiphertextAffineRepr = CiphertextAffine<EdwardsProjective>;

/// An ElGamal ciphertext
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Deserialize, Serialize)]
pub struct CiphertextAffine<C: CurveGroup> {
    /// `e1` = `r * G`
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub e1: C::Affine,
    /// `e2` = `M + r * pk`
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub e2: C::Affine,
}

impl<C: CurveGroup> CiphertextAffine<C> {
    pub fn new(e1: C::Affine, e2: C::Affine) -> Self {
        Self { e1, e2 }
    }

    pub fn rand<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        let m = C::rand(prng);
        let pk = C::rand(prng);
        Self::encrypt(prng, &m, &pk)
    }

    pub fn encrypt<R: CryptoRng + RngCore>(prng: &mut R, m: &C, pk: &C) -> Self {
        let g = C::generator();

        let r = C::ScalarField::rand(prng);
        let e1 = g.mul(&r);
        let e2 = m.add(pk.mul(r));

        Self::new(e1.into_affine(), e2.into_affine())
    }

    pub fn verify(&self, m: &C, sk: &C::ScalarField) -> bool {
        *m == self.e2.sub(self.e1.mul(sk))
    }

    pub fn flatten(&self) -> [C::BaseField; 4] {
        let (x1, y1) = self.e1.xy().unwrap();
        let (x2, y2) = self.e2.xy().unwrap();
        [x2, y2, x1, y1]
    }

    pub fn to_ciphertext(&self) -> Ciphertext<C> {
        Ciphertext {
            e1: self.e1.into(),
            e2: self.e2.into(),
        }
    }
}

impl<C: CurveGroup> From<Ciphertext<C>> for CiphertextAffine<C> {
    fn from(value: Ciphertext<C>) -> Self {
        Self::new(value.e1.into_affine(), value.e2.into_affine())
    }
}
