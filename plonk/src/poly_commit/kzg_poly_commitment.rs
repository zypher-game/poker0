use crate::{
    poly_commit::{
        errors::PolyComSchemeError,
        field_polynomial::FpPolynomial,
        pcs::{HomomorphicPolyComElem, PolyComScheme, ToBytes},
    },
    utils::prelude::*,
};

use ark_bn254::{Bn254, Fq12Config, Fr, G1Projective};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup, PrimeGroup, VariableBaseMSM};
use ark_ff::{AdditiveGroup, Fp12, One, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::{UniformRand, Zero};

/// KZG commitment scheme over the `Group`.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct KZGCommitment<G: CanonicalSerialize + CanonicalDeserialize>(
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")] pub G,
);

impl<G> ToBytes for KZGCommitment<G>
where
    G: CurveGroup,
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.0.serialize_with_mode(&mut buf, Compress::Yes).unwrap();
        buf
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        let aff_repr = self.0.into_affine();
        let x: G::BaseField = aff_repr.x().unwrap_or(G::BaseField::ZERO);
        let y: G::BaseField = aff_repr.y().unwrap_or(G::BaseField::ZERO);

        let mut buf_x = vec![];
        x.serialize_with_mode(&mut buf_x, Compress::Yes).unwrap();
        buf_x.reverse();

        let mut buf_y = vec![];
        y.serialize_with_mode(&mut buf_y, Compress::Yes).unwrap();
        buf_y.reverse();

        buf_x.extend_from_slice(&buf_y);

        buf_x
    }
}

impl HomomorphicPolyComElem for KZGCommitment<G1Projective> {
    type Scalar = Fr;
    fn get_base() -> Self {
        KZGCommitment(G1Projective::generator())
    }

    fn get_identity() -> Self {
        KZGCommitment(G1Projective::zero())
    }

    fn add(&self, other: &Self) -> Self {
        KZGCommitment(self.0.add(&other.0))
    }

    fn add_assign(&mut self, other: &Self) {
        self.0.add_assign(&other.0)
    }

    fn sub(&self, other: &Self) -> Self {
        KZGCommitment(self.0.sub(&other.0))
    }

    fn sub_assign(&mut self, other: &Self) {
        self.0.sub_assign(&other.0)
    }

    fn mul(&self, exp: &Fr) -> Self {
        KZGCommitment(self.0.mul(exp))
    }

    fn mul_assign(&mut self, exp: &Fr) {
        self.0.mul_assign(exp)
    }
}

impl<F: PrimeField> ToBytes for FpPolynomial<F> {
    fn to_transcript_bytes(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn to_bytes(&self) -> Vec<u8> {
        unimplemented!()
    }
}

impl<F: PrimeField> HomomorphicPolyComElem for FpPolynomial<F> {
    type Scalar = F;

    fn get_base() -> Self {
        unimplemented!()
    }

    fn get_identity() -> Self {
        unimplemented!()
    }

    fn add(&self, other: &Self) -> Self {
        self.add(other)
    }

    fn add_assign(&mut self, other: &Self) {
        self.add_assign(other)
    }

    fn sub(&self, other: &Self) -> Self {
        self.sub(other)
    }

    fn sub_assign(&mut self, other: &Self) {
        self.sub_assign(other)
    }

    fn mul(&self, exp: &F) -> Self {
        self.mul_scalar(exp)
    }

    fn mul_assign(&mut self, exp: &F) {
        self.mul_scalar_assign(exp)
    }
}

/// KZG opening proof.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct KZGOpenProof<G1: CanonicalSerialize + CanonicalDeserialize>(
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")] pub G1,
);

impl<G: PrimeGroup> ToBytes for KZGOpenProof<G> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.0.serialize_with_mode(&mut buf, Compress::Yes).unwrap();
        buf
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        unimplemented!()
    }
}

/// KZG commitment scheme about `PairingEngine`.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct KZGCommitmentScheme<P: Pairing> {
    /// public parameter about G1.
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub public_parameter_group_1: Vec<P::G1>,
    /// public parameter about G1.
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub public_parameter_group_2: Vec<P::G2>,
}

impl<P: Pairing> KZGCommitmentScheme<P> {
    /// Create a new instance of a KZG polynomial commitment scheme.
    /// `max_degree` - max degree of the polynomial,
    /// `prng` - pseudo-random generator.
    pub fn new<R: CryptoRng + RngCore>(max_degree: usize, prng: &mut R) -> KZGCommitmentScheme<P> {
        let s = P::ScalarField::rand(prng);

        let mut public_parameter_group_1: Vec<P::G1> = Vec::new();

        let mut elem_g1 = P::G1::generator();

        for _ in 0..=max_degree {
            public_parameter_group_1.push(elem_g1.clone());
            elem_g1 = elem_g1.mul(&s);
        }

        let mut public_parameter_group_2: Vec<P::G2> = Vec::new();
        let elem_g2 = P::G2::generator();
        public_parameter_group_2.push(elem_g2.clone());
        public_parameter_group_2.push(elem_g2.mul(&s));

        KZGCommitmentScheme {
            public_parameter_group_1,
            public_parameter_group_2,
        }
    }

    /// Serialize the parameters to unchecked bytes.
    pub fn to_unchecked_bytes(&self) -> Result<Vec<u8>, PolyComSchemeError> {
        let mut bytes = vec![];
        let len_1 = self.public_parameter_group_1.len() as u32;
        let len_2 = self.public_parameter_group_2.len() as u32;
        bytes.extend(len_1.to_le_bytes());
        bytes.extend(len_2.to_le_bytes());

        for i in &self.public_parameter_group_1 {
            let mut buf = Vec::new();
            i.serialize_with_mode(&mut buf, Compress::No).unwrap();
            bytes.extend(buf);
        }
        for i in &self.public_parameter_group_2 {
            let mut buf = Vec::new();
            i.serialize_with_mode(&mut buf, Compress::No).unwrap();
            bytes.extend(buf);
        }
        Ok(bytes)
    }

    /// Deserialize the parameters from unchecked bytes.
    pub fn from_unchecked_bytes(bytes: &[u8]) -> Result<Self, PolyComSchemeError> {
        if bytes.len() < 8 {
            return Err(PolyComSchemeError::DeserializationError);
        }
        let mut len_1_bytes = [0u8; 4];
        let mut len_2_bytes = [0u8; 4];
        len_1_bytes.copy_from_slice(&bytes[0..4]);
        len_2_bytes.copy_from_slice(&bytes[4..8]);
        let len_1 = u32::from_le_bytes(len_1_bytes) as usize;
        let len_2 = u32::from_le_bytes(len_2_bytes) as usize;
        let n_1 = P::G1::default().serialized_size(Compress::No);
        let n_2 = P::G2::default().serialized_size(Compress::No);

        let bytes_1 = &bytes[8..];
        let bytes_2 = &bytes[8 + (n_1 * len_1)..];
        let mut p1 = vec![];
        let mut p2 = vec![];

        for i in 0..len_1 {
            let reader = &bytes_1[n_1 * i..n_1 * (i + 1)];
            let g1 = P::G1::deserialize_with_mode(reader, Compress::No, Validate::No)
                .map_err(|_| PolyComSchemeError::DeserializationError)?;
            p1.push(g1);
        }

        for i in 0..len_2 {
            let reader = &bytes_2[n_2 * i..n_2 * (i + 1)];
            let g2 = P::G2::deserialize_with_mode(reader, Compress::No, Validate::No)
                .map_err(|_| PolyComSchemeError::DeserializationError)?;
            p2.push(g2);
        }

        Ok(Self {
            public_parameter_group_1: p1,
            public_parameter_group_2: p2,
        })
    }
}

/// KZG commitment scheme over the BN254 curve
pub type KZGCommitmentSchemeBN254 = KZGCommitmentScheme<Bn254>;

impl<'b> PolyComScheme for KZGCommitmentSchemeBN254 {
    type Field = Fr;
    type Commitment = KZGCommitment<G1Projective>;

    fn max_degree(&self) -> usize {
        self.public_parameter_group_1.len() - 1
    }

    fn commit(
        &self,
        polynomial: &FpPolynomial<Fr>,
    ) -> Result<Self::Commitment, PolyComSchemeError> {
        let coefs = polynomial.get_coefs_ref();

        let degree = polynomial.degree();

        if degree + 1 > self.public_parameter_group_1.len() {
            return Err(PolyComSchemeError::DegreeError);
        }

        let points_raw =
            G1Projective::normalize_batch(&self.public_parameter_group_1[0..degree + 1]);

        let commitment_value = G1Projective::msm(&points_raw, &coefs).unwrap();

        Ok(KZGCommitment(commitment_value))
    }

    fn eval(&self, poly: &FpPolynomial<Self::Field>, point: &Self::Field) -> Self::Field {
        poly.eval(point)
    }

    fn apply_blind_factors(
        &self,
        commitment: &Self::Commitment,
        blinds: &[Self::Field],
        zeroing_degree: usize,
    ) -> Self::Commitment {
        let mut commitment = commitment.0.clone();
        for (i, blind) in blinds.iter().enumerate() {
            let mut blind = blind.clone();
            commitment = commitment + &(self.public_parameter_group_1[i] * &blind);
            blind = blind.neg();
            commitment = commitment + &(self.public_parameter_group_1[zeroing_degree + i] * &blind);
        }
        KZGCommitment(commitment)
    }

    fn prove(
        &self,
        poly: &FpPolynomial<Self::Field>,
        x: &Self::Field,
        max_degree: usize,
    ) -> Result<Self::Commitment, PolyComSchemeError> {
        let eval = poly.eval(x);

        if poly.degree() > max_degree {
            return Err(PolyComSchemeError::DegreeError);
        }

        let nominator = poly.sub(&FpPolynomial::from_coefs(vec![eval]));

        // Negation must happen in Fq
        let point_neg = x.neg();

        // X - x
        let vanishing_poly = FpPolynomial::from_coefs(vec![point_neg, Self::Field::one()]);
        let (q_poly, r_poly) = nominator.div_rem(&vanishing_poly); // P(X)-P(x) / (X-x)

        if !r_poly.is_zero() {
            return Err(PolyComSchemeError::PCSProveEvalError);
        }

        let proof = self.commit(&q_poly).unwrap();
        Ok(proof)
    }

    fn verify(
        &self,
        cm: &Self::Commitment,
        _degree: usize,
        point: &Self::Field,
        eval: &Self::Field,
        proof: &Self::Commitment,
    ) -> Result<(), PolyComSchemeError> {
        let g1_0 = self.public_parameter_group_1[0].clone();
        let g2_0 = self.public_parameter_group_2[0].clone();
        let g2_1 = self.public_parameter_group_2[1].clone();

        let x_minus_point_group_element_group_2 = &g2_1.sub(&g2_0.mul(point));

        let left_pairing_eval = if eval.is_zero() {
            Bn254::pairing(&cm.0, &g2_0)
        } else {
            Bn254::pairing(&cm.0.sub(&g1_0.mul(eval)), &g2_0)
        };

        let right_pairing_eval = Bn254::pairing(&proof.0, x_minus_point_group_element_group_2);

        if left_pairing_eval == right_pairing_eval {
            Ok(())
        } else {
            Err(PolyComSchemeError::PCSProveEvalError)
        }
    }

    fn batch_verify_diff_points(
        &self,
        cm_vec: &[Self::Commitment],
        point_vec: &[Self::Field],
        eval_vec: &[Self::Field],
        proofs: &[Self::Commitment],
        challenge: &Self::Field,
    ) -> Result<(), PolyComSchemeError> {
        assert!(proofs.len() > 0);
        assert_eq!(proofs.len(), point_vec.len());
        assert_eq!(proofs.len(), eval_vec.len());
        assert_eq!(proofs.len(), cm_vec.len());

        let g1_0 = self.public_parameter_group_1[0].clone();
        let g2_0 = self.public_parameter_group_2[0].clone();
        let g2_1 = self.public_parameter_group_2[1].clone();

        let left_second = g2_1;
        let right_second = g2_0;

        let mut left_first = proofs[0].0.clone();
        let mut right_first = proofs[0].0.mul(&point_vec[0]);
        let mut right_first_val = eval_vec[0].clone();
        let mut right_first_comm = cm_vec[0].0.clone();

        let mut cur_challenge = challenge.clone();
        for i in 1..proofs.len() {
            let new_comm = proofs[i].0.mul(&cur_challenge);

            left_first.add_assign(&new_comm);
            right_first.add_assign(&new_comm.mul(&point_vec[i]));
            right_first_val.add_assign(&eval_vec[i].mul(&cur_challenge));
            right_first_comm.add_assign(&cm_vec[i].0.mul(&cur_challenge));

            cur_challenge.mul_assign(challenge);
        }
        right_first.sub_assign(&g1_0.mul(&right_first_val));
        right_first.add_assign(&right_first_comm);

        let pairing_eval = Bn254::multi_pairing(
            &[left_first, right_first.neg()],
            &[left_second, right_second],
        )
        .0;

        if pairing_eval == Fp12::<Fq12Config>::one() {
            Ok(())
        } else {
            Err(PolyComSchemeError::PCSProveEvalError)
        }
    }

    fn shrink_to_verifier_only(&self) -> Result<Self, PolyComSchemeError> {
        Ok(Self {
            public_parameter_group_1: vec![self.public_parameter_group_1[0].clone()],
            public_parameter_group_2: vec![
                self.public_parameter_group_2[0].clone(),
                self.public_parameter_group_2[1].clone(),
            ],
        })
    }
}
