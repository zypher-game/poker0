use poker_core::{mock_data::task::mock_task, task::Task0};
use poker_methods::{POKER_METHOD_ELF, POKER_METHOD_ID};
use risc0_zkvm::{default_executor, default_prover, ExecutorEnv};

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

pub fn execute_local(task: &Task0) {
    let env = ExecutorEnv::builder()
        .write(&task)
        .unwrap()
        .build()
        .unwrap();

    let exec = default_executor();

    let _session_info = exec.execute(env, POKER_METHOD_ELF).unwrap();

    // Executing the following code requires the deserialize0 feature, but such mack tack will fail.
    //
    // let journal : TaskCommit = session_info.journal.decode().unwrap();
    // assert_eq!(journal.room_id, task.room_id);
    // assert_eq!(journal.players_hand, task.players_hand);
    // assert_eq!(journal.winner, 2);
}

// cargo run --package poker-host --bin poker-host
// RISC0_PPROF_OUT=./profile.pb cargo run --package poker-host --bin poker-host
fn main() {
    let task = mock_task();
    let task0 = task.convert0();
    execute_local(&task0);
}
