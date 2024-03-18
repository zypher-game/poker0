use ::bonsai_sdk::alpha::{responses::SnarkReceipt, SessionId};
use bonsai_sdk::alpha as bonsai_sdk;
use poker_core::errors::{PokerError, Result};
use std::time::Duration;

pub fn stark_to_snark(session_id: SessionId) -> Result<SnarkReceipt> {
    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)
        .map_err(|x| PokerError::BonsaiSdkError(x.to_string()))?;

    let snark_session = client
        .create_snark(session_id.uuid)
        .map_err(|x| PokerError::BonsaiSdkError(x.to_string()))?;

    loop {
        let res = snark_session
            .status(&client)
            .map_err(|x| PokerError::BonsaiSdkError(x.to_string()))?;

        match res.status.as_str() {
            "RUNNING" => {
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            "SUCCEEDED" => {
                return Ok(res.output.unwrap());
            }
            _ => {
                return Err(PokerError::BonsaiSdkError(format!(
                    "Workflow exited: {} - | err: {}",
                    res.status,
                    res.error_msg.unwrap_or_default()
                )));
            }
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod test {
    use crate::{snark::stark_to_snark, stark::prove_bonsai};
    use ark_bn254::Fr;
    use ark_ff::{BigInteger, PrimeField};
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use hex::FromHex;
    use num_bigint::BigInt;
    use poker_core::{
        mock_data::{journal::mock_journal, task0::mock_task0},
        task::{Task0, TaskCommit},
    };
    use poker_methods::POKER_METHOD_ID;
    use risc0_zkvm::{
        serde::{from_slice, to_vec},
        sha::{Digest, Digestible},
        Groth16Proof, Groth16Receipt, Groth16Seal, InnerReceipt, Journal, Receipt,
        ALLOWED_IDS_ROOT,
    };
    use std::{str::FromStr, time::Instant};

    fn fr_from_bytes(scalar: &[u8]) -> Fr {
        let scalar: Vec<u8> = scalar.iter().rev().cloned().collect();
        Fr::deserialize_uncompressed(&*scalar).unwrap()
    }

    fn to_fixed_array(input: Vec<u8>) -> [u8; 32] {
        let mut fixed_array = [0u8; 32];
        let start = core::cmp::max(32, input.len()) - core::cmp::min(32, input.len());
        fixed_array[start..].copy_from_slice(&input[input.len().saturating_sub(32)..]);
        fixed_array
    }

    fn from_u256(value: &str) -> Vec<u8> {
        let value = if let Some(stripped) = value.strip_prefix("0x") {
            to_fixed_array(hex::decode(stripped).unwrap()).to_vec()
        } else {
            to_fixed_array(BigInt::from_str(value).unwrap().to_bytes_be().1).to_vec()
        };

        value
    }

    fn split_digest(d: Digest) -> (Fr, Fr) {
        let big_endian: Vec<u8> = d.as_bytes().to_vec().iter().rev().cloned().collect();
        let middle = big_endian.len() / 2;
        let (a, b) = big_endian.split_at(middle);
        (
            fr_from_bytes(&from_u256(&format!("0x{}", hex::encode(a)))),
            fr_from_bytes(&from_u256(&format!("0x{}", hex::encode(b)))),
        )
    }

    #[test]
    fn stark_to_snark_test() {
        dotenv::dotenv().ok();
        let task_bytes = mock_task0();
        let task: Task0 = from_slice(&task_bytes).unwrap();

        let start = Instant::now();
        let (receipt, session_id) = prove_bonsai(&task).unwrap();
        println!("Prover time: {:.2?}", start.elapsed());

        assert!(receipt.verify(POKER_METHOD_ID).is_ok());

        let commit: TaskCommit = receipt.journal.decode().unwrap();
        assert_eq!(commit.room_id, task.room_id);
        assert_eq!(commit.players_hand, task.players_hand);
        assert_eq!(commit.winner, 2);

        let start = Instant::now();
        let snark_proof = stark_to_snark(session_id).unwrap();
        println!("Stark2Snark time: {:.2?}", start.elapsed());

        let receipt_claim = receipt.get_claim().unwrap();

        {
            let (c1, c2) = split_digest(Digest::from_hex(ALLOWED_IDS_ROOT).unwrap());
            let (m1, m2) = split_digest(receipt_claim.digest());

            assert_eq!(c2.into_bigint().to_bytes_be(), snark_proof.snark.public[0]);
            assert_eq!(c1.into_bigint().to_bytes_be(), snark_proof.snark.public[1]);
            assert_eq!(m2.into_bigint().to_bytes_be(), snark_proof.snark.public[2]);
            assert_eq!(m1.into_bigint().to_bytes_be(), snark_proof.snark.public[3]);
        }

        // {
        //     let groth16_seal = Groth16Seal {
        //         a: snark_proof.snark.a.clone(),
        //         b: snark_proof.snark.b.clone(),
        //         c: snark_proof.snark.c.clone(),
        //     };

        //     let receipt = Receipt::new(
        //         InnerReceipt::Groth16(Groth16Receipt {
        //             seal: groth16_seal.to_vec(),
        //             claim: receipt_claim.clone(),
        //         }),
        //         receipt.journal.bytes,
        //     );

        //     assert!(receipt.verify(POKER_METHOD_ID).is_ok());
        // }

        // {
        //     let groth16_seal = Groth16Seal {
        //         a: snark_proof.snark.a,
        //         b: snark_proof.snark.b,
        //         c: snark_proof.snark.c,
        //     };

        //     let groth16_proof =
        //         Groth16Proof::from_seal(&groth16_seal, receipt_claim.digest()).unwrap();

        //     assert!(groth16_proof.verify().is_ok())
        // }
    }

    #[test]
    fn onchain_verify_test() {
        dotenv::dotenv().ok();
        let task_bytes = mock_task0();
        let task: Task0 = from_slice(&task_bytes).unwrap();

        let start = Instant::now();
        let (receipt, session_id) = prove_bonsai(&task).unwrap();
        println!("Prover time: {:.2?}", start.elapsed());

        assert!(receipt.verify(POKER_METHOD_ID).is_ok());

        let journal: Vec<u8> = bytemuck::cast_slice(&mock_journal()).to_vec();
        assert_eq!(journal, receipt.journal.bytes);

        let commit: TaskCommit = receipt.journal.decode().unwrap();
        assert_eq!(commit.room_id, task.room_id);
        assert_eq!(commit.players_hand, task.players_hand);
        assert_eq!(commit.winner, 2);

        let start = Instant::now();
        let snark_proof = stark_to_snark(session_id).unwrap();
        println!("Stark2Snark time: {:.2?}", start.elapsed());

        let receipt_claim = receipt.get_claim().unwrap();

        let groth16_seal = Groth16Seal {
            a: snark_proof.snark.a,
            b: snark_proof.snark.b,
            c: snark_proof.snark.c,
        };
        let image_id: Digest = POKER_METHOD_ID.into();

        println!("---------on-chain verification data---------");
        println!("seal:0x{}", hex::encode(groth16_seal.to_vec()));
        println!("image_id:0x{}", image_id);
        println!("post_digest:0x{}", receipt_claim.post.digest());
        println!("jounral:0x{}", receipt.journal.digest());
    }

    #[test]
    fn journal_test() {
        let journal_byte32 = mock_journal();
        // println!("{:?}",journal_byte32);
        let journal_bytes: Vec<u8> = bytemuck::cast_slice(&journal_byte32).to_vec();
        println!("{:?}", journal_bytes);
        let commit: TaskCommit = from_slice(&journal_byte32).unwrap();

        let vec = to_vec(&commit).unwrap();

        let x0 = commit.players_hand[0][0].0.e1;
        let mut y0 = vec![];
        x0.serialize_uncompressed(&mut y0).unwrap();

        let x1 = commit.players_hand[0][0].0.e2;
        let mut y1 = vec![];
        x1.serialize_uncompressed(&mut y1).unwrap();

        println!("y0:{:?}", y0);
        println!("y1:{:?}", y1);

        //  println!("{:?}",commit);
    }
}
