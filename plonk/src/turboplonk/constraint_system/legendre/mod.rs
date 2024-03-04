use crate::turboplonk::constraint_system::{TurboCS, VarIndex};
use ark_ff::PrimeField;

impl<F: PrimeField> TurboCS<F> {
    pub fn check_legendre_correctness(
        &mut self,
        bit: bool,
        qnr: F,
        idx: F,
        key_var: VarIndex,
        nonce_var: VarIndex,
        bit_var: VarIndex,
        trace_var: VarIndex,
    ) {
        let zero = F::ZERO;
        let one = F::ONE;
        let minus_one = one.neg();
        let minus_idx = idx.neg();

        let zero_var = self.zero_var();

        let modifier = if bit { F::ONE } else { qnr };

        let modifier_var = self.new_variable(modifier);

        self.push_add_selectors(zero, qnr, zero, zero);
        self.push_mul_selectors(F::ONE - &qnr, minus_one);
        self.push_constant_selector(zero);
        self.push_ecc_selector(zero);
        self.push_out_selector(zero);

        self.wiring[0].push(bit_var);
        self.wiring[1].push(trace_var);
        self.wiring[2].push(modifier_var);
        self.wiring[3].push(trace_var);
        self.wiring[4].push(zero_var);
        self.finish_new_gate();

        let tmp_var = self.add(key_var, nonce_var);

        self.push_add_selectors(zero, zero, zero, zero);
        self.push_mul_selectors(one, minus_one);
        self.push_constant_selector(zero);
        self.push_ecc_selector(zero);
        self.push_out_selector(minus_idx);

        self.wiring[0].push(tmp_var);
        self.wiring[1].push(modifier_var);
        self.wiring[2].push(trace_var);
        self.wiring[3].push(trace_var);
        self.wiring[4].push(modifier_var);
        self.finish_new_gate();
    }
}
