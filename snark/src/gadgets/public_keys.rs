use ark_bn254::Fr;
use poker_core::schnorr::PublicKey;
use zplonk::turboplonk::constraint_system::{ecc::PointVar, turbo::TurboCS};

pub struct PublicKeyOutsource {
    pub public_keys: Vec<PublicKey>,
    pub cs_vars: Vec<PointVar>,
}

impl PublicKeyOutsource {
    pub fn new(cs: &mut TurboCS<Fr>, pks: &[PublicKey]) -> Self {
        let public_keys = pks.to_vec();
        let cs_vars = public_keys
            .iter()
            .map(|x| cs.new_point_variable(x.0.into()))
            .collect();

        Self {
            public_keys,
            cs_vars,
        }
    }

    pub fn prepare_pi_variables(&self, cs: &mut TurboCS<Fr>) {
        for var in self.cs_vars.iter() {
            cs.prepare_pi_point_variable(*var);
        }
    }
}

#[cfg(test)]
mod test {
    use ark_bn254::Fr;
    use poker_core::mock_data::task::mock_task;
    use zplonk::turboplonk::constraint_system::turbo::TurboCS;

    use super::PublicKeyOutsource;

    #[test]
    fn test_reveals_constraint_system() {
        let mut cs = TurboCS::<Fr>::new();
        let task = mock_task();
        let pk_outsource = PublicKeyOutsource::new(&mut cs, &task.players_key);
        pk_outsource.prepare_pi_variables(&mut cs);

        let mut pi = vec![];
        task.players_key.iter().for_each(|x| {
            pi.push(x.0.x);
            pi.push(x.0.y);
        });

        let witness = cs.get_and_clear_witness();
        cs.verify_witness(&witness, &pi).unwrap();
    }
}
