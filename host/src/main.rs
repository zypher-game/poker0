use poker_core::{mock_data::task::mock_task, task::Task0};
use poker_methods::{POKER_METHOD_ELF, POKER_METHOD_ID};
use risc0_zkvm::{default_prover, ExecutorEnv};

pub fn prove_task(task: &Task0) {
    let env = ExecutorEnv::builder()
        .write(&task)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    let start = std::time::Instant::now();
    let receipt = prover.prove(env, POKER_METHOD_ELF).unwrap();
    println!("prover time: {:.2?}", start.elapsed());

    assert!(receipt.verify(POKER_METHOD_ID).is_ok());
}
/*
#export BONSAI_API_URL="https://api.bonsai.xyz/"
#export BONSAI_API_KEY="L2r4ndmLV92w8SEFl11tlabBXEEk6NIs97RsMZCC"

read cycle:1582938
serialize_uncompressed cycle:13716
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:4964
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:4964
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:4964
serialize_uncompressed cycle:4964
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
serialize_uncompressed cycle:3870
total cycle:2995539
*/

// cargo run --package poker-host --bin poker-host
// RISC0_PPROF_OUT=./profile.pb cargo run --package poker-host --bin poker-host
fn main() {
    let task = mock_task();
    let task0 = task.convert0();
    prove_task(&task0);
}
