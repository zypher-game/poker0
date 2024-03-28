use std::ops::{Add, Mul, Sub};

use crate::errors::{PokerError, Result};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ed_on_bn254::EdwardsAffine;
use ark_ff::{BigInteger, PrimeField};
use ark_std::UniformRand;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use zplonk::{
    anemoi::{AnemoiJive, AnemoiJive254},
    utils::serialization::{ark_deserialize, ark_serialize},
};

/// The public key.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct PrivateKey(
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub  ark_ed_on_bn254::Fr,
);

/// The private key.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Hash, Default)]
pub struct PublicKey(
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub  ark_ed_on_bn254::EdwardsAffine,
);

/// The signature.
#[derive(Clone, Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
pub struct Signature {
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub s: ark_ed_on_bn254::Fr,
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub e: ark_bn254::Fr,
}

/// The keypair for schnorr signature.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct KeyPair {
    pub(crate) private_key: PrivateKey,
    pub(crate) public_key: PublicKey,
}

impl Into<zshuffle::keygen::Keypair> for KeyPair {
    fn into(self) -> zshuffle::keygen::Keypair {
        zshuffle::keygen::Keypair {
            secret: self.get_private_key().0,
            public: self.get_public_key().0.into(),
        }
    }
}

impl KeyPair {
    pub fn sample<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        let sk = PrivateKey::random(prng);
        Self::from_private_key(sk)
    }

    pub fn from_private_key(sk: PrivateKey) -> Self {
        let vk = ark_ed_on_bn254::EdwardsAffine::generator().mul(&sk.0);
        Self {
            private_key: sk,
            public_key: PublicKey(vk.into_affine()),
        }
    }

    /// Get the private key.
    pub fn get_private_key(&self) -> PrivateKey {
        self.private_key.clone()
    }

    /// Get the public key.
    pub fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn sign<R: CryptoRng + RngCore>(&self, msg: &[u8], prng: &mut R) -> Result<Signature> {
        let r = ark_ed_on_bn254::Fr::rand(prng);
        let big_r = EdwardsAffine::generator().mul(&r).into_affine();

        let mut input = vec![ark_bn254::Fr::from_be_bytes_mod_order(msg)];
        input.extend_from_slice(&[
            self.get_public_key().0.x,
            self.get_public_key().0.y,
            big_r.x,
            big_r.y,
        ]);

        let e = AnemoiJive254::eval_variable_length_hash(&input);

        let e_reduction =
            ark_ed_on_bn254::Fr::from_be_bytes_mod_order(&e.into_bigint().to_bytes_be());

        let s = r.sub(&self.get_private_key().0.mul(e_reduction));

        Ok(Signature { s, e })
    }

    // pub fn sign<R: CryptoRng + RngCore>(&self, msg: &[u8], prng: &mut R) -> Result<Signature> {
    //     let r = ark_ed_on_bn254::Fr::rand(prng);
    //     let big_r = EdwardsAffine::generator().mul(&r).into_affine();

    //     let mut pk_bytes = vec![];
    //     self.get_public_key()
    //         .0
    //         .serialize_uncompressed(&mut pk_bytes)
    //         .unwrap();
    //     let mut r_bytes = vec![];
    //     big_r.serialize_uncompressed(&mut r_bytes).unwrap();

    //     let mut msg = msg.to_vec();
    //     msg.extend(&pk_bytes);
    //     msg.extend(&r_bytes);

    //     let digest = Impl::hash_bytes(&msg);

    //     let e = ark_ed_on_bn254::Fr::from_be_bytes_mod_order(digest.as_bytes());

    //     let s = r.sub(&self.get_private_key().0.mul(e));

    //     Ok(Signature { s, e })
    // }
}

impl PrivateKey {
    pub fn random<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        let sk = ark_ed_on_bn254::Fr::rand(prng);
        Self(sk)
    }
}

impl PublicKey {
    pub fn get_raw(&self) -> ark_ed_on_bn254::EdwardsProjective {
        self.0.into()
    }

    pub fn rand<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        Self(ark_ed_on_bn254::EdwardsAffine::rand(prng))
    }

    pub fn verify(&self, s: &Signature, msg: &[u8]) -> Result<()> {
        let e_reduction =
            ark_ed_on_bn254::Fr::from_be_bytes_mod_order(&s.e.into_bigint().to_bytes_be());
        let big_r = EdwardsAffine::generator()
            .mul(&s.s)
            .add(self.0.mul(&e_reduction))
            .into_affine();

        let mut input = vec![ark_bn254::Fr::from_be_bytes_mod_order(msg)];
        input.extend_from_slice(&[self.0.x, self.0.y, big_r.x, big_r.y]);

        let e = AnemoiJive254::eval_variable_length_hash(&input);

        if e != s.e {
            Err(PokerError::VerifySignatureError)
        } else {
            Ok(())
        }
    }

    // pub fn verify(&self, s: &Signature, msg: &[u8]) -> Result<()> {
    //     let big_r = EdwardsAffine::generator()
    //         .mul(&s.s)
    //         .add(self.0.mul(&s.e))
    //         .into_affine();

    //     let mut pk_bytes = vec![];
    //     self.0.serialize_uncompressed(&mut pk_bytes).unwrap();
    //     let mut r_bytes = vec![];
    //     big_r.serialize_uncompressed(&mut r_bytes).unwrap();

    //     let mut msg = msg.to_vec();
    //     msg.extend(&pk_bytes);
    //     msg.extend(&r_bytes);

    //     let digest = Impl::hash_bytes(&msg);

    //     let e = ark_ed_on_bn254::Fr::from_be_bytes_mod_order(digest.as_bytes());

    //     if e != s.e {
    //         Err(PokerError::VerifySignatureError)
    //     } else {
    //         Ok(())
    //     }
    // }
}

#[cfg(test)]
mod test {
    use crate::schnorr::KeyPair;
    use rand_chacha::{rand_core::SeedableRng, ChaChaRng};

    #[test]
    fn test_schnorr() {
        let mut prng = ChaChaRng::from_seed([0u8; 32]);
        let key_pair = KeyPair::sample(&mut prng);
        let msg = "If I play a bomb card, how would you respond?";
        let s = key_pair.sign(msg.as_bytes(), &mut prng).unwrap();
        assert!(key_pair.get_public_key().verify(&s, msg.as_bytes()).is_ok());
    }
}
