use ark_bn254::Fr;
use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective};
use zplonk::turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS};

use poker_core::schnorr::PublicKey;

pub struct PublicKeyOutsource {
    pub public_keys: Vec<EdwardsProjective>,
    pub cs_vars: Vec<PointVar>,
    pub affine_reps: Vec<EdwardsAffine>,
}

impl PublicKeyOutsource {
    pub fn new(cs: &mut TurboCS<Fr>, pks: &[PublicKey]) -> Self {
        let affine_reps: Vec<EdwardsAffine> = pks.iter().map(|x| x.0).collect();
        let public_keys: Vec<EdwardsProjective> = affine_reps.iter().map(|x| (*x).into()).collect();
        let cs_vars = public_keys
            .iter()
            .map(|x| cs.new_point_variable(*x))
            .collect();

        Self {
            public_keys,
            cs_vars,
            affine_reps,
        }
    }
}
