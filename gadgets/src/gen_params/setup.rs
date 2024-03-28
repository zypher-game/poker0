#![allow(clippy::upper_case_acronyms)]
#![allow(non_camel_case_types)]
#![cfg_attr(any(feature = "no_srs", feature = "no_vk"), allow(unused))]

use ark_bn254::G1Projective;
use poker_core::mock_data::task::mock_task;
use poker_gadgets::{
    build_cs::{build_cs, N_CARDS, N_PLAYS},
    create_and_rescale_outsource,
    gen_params::{params::VerifierParams, SRS},
};
use std::path::PathBuf;
use structopt::StructOpt;
use zplonk::{
    poly_commit::kzg_poly_commitment::KZGCommitmentSchemeBN254,
    turboplonk::constraint_system::ConstraintSystem,
};

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Tool to generate necessary zero-knowledge proof parameters.",
    rename_all = "kebab-case"
)]
enum Actions {
    /// Generates the verifying key
    VK { directory: PathBuf },

    /// Cut the SRS, adapt to Lagrange
    CUT_SRS { directory: PathBuf },

    /// Generates the permutation
    PERMUTATION { directory: PathBuf },

    /// Generates all necessary parameters
    ALL { directory: PathBuf },
}

fn main() {
    use Actions::*;
    let action = Actions::from_args();
    match action {
        VK { directory } => gen_vk(directory),

        CUT_SRS { directory } => cut_srs(directory),

        PERMUTATION { directory } => gen_premutation(directory),

        ALL { directory } => gen_all(directory),
    };
}

// cargo run --release --features="gen no_vk" --bin gen-params vk "./parameters"
fn gen_vk(directory: PathBuf) {
    let params = VerifierParams::get().unwrap();
    println!(
        "the size of the constraint system for {} step of shifts: {}",
        1, params.shrunk_cs.size
    );

    let (common, special) = params.split().unwrap();
    let common_ser = bincode::serialize(&common).unwrap();

    let mut common_path = directory.clone();
    common_path.push("vk-common.bin");
    save_to_file(&common_ser, common_path);

    let specials_ser = bincode::serialize(&special).unwrap();
    let mut specials_path = directory.clone();
    specials_path.push("vk-specific.bin");
    save_to_file(&specials_ser, specials_path);
}

// cargo run --release --features="gen no_vk" --bin gen-params permutation "./parameters"
fn gen_premutation(directory: PathBuf) {
    let (players_key, reveal_outsources, unmask_outsources, signature_outsources) =
        create_and_rescale_outsource(&mock_task(), N_PLAYS, N_CARDS);

    let cs = build_cs(
        &players_key,
        &reveal_outsources,
        &unmask_outsources,
        &signature_outsources,
    );

    let special = cs.compute_permutation();

    let specials_ser = bincode::serialize(&special).unwrap();
    let mut specials_path = directory.clone();
    specials_path.push("permutation.bin");
    save_to_file(&specials_ser, specials_path);
}

// cargo run --release --features="gen no_vk" --bin gen-params cut-srs "./parameters"
fn cut_srs(mut path: PathBuf) {
    let srs = SRS.unwrap();
    let KZGCommitmentSchemeBN254 {
        public_parameter_group_1,
        public_parameter_group_2,
    } = KZGCommitmentSchemeBN254::from_unchecked_bytes(&srs).unwrap();

    if public_parameter_group_1.len() == 2078 {
        println!("Already complete");
        return;
    }

    let mut new_group_1 = vec![G1Projective::default(); 2078];
    new_group_1[0..2051].copy_from_slice(&public_parameter_group_1[0..2051]);
    new_group_1[2051..2054].copy_from_slice(&public_parameter_group_1[4096..4099]);
    new_group_1[2054..2057].copy_from_slice(&public_parameter_group_1[8192..8195]);
    new_group_1[2057..2060].copy_from_slice(&public_parameter_group_1[16384..16387]);
    new_group_1[2060..2063].copy_from_slice(&public_parameter_group_1[32768..32771]);
    new_group_1[2063..2066].copy_from_slice(&public_parameter_group_1[65536..65539]);
    new_group_1[2066..2069].copy_from_slice(&public_parameter_group_1[131072..131075]);
    new_group_1[2069..2072].copy_from_slice(&public_parameter_group_1[262144..262147]);
    new_group_1[2072..2075].copy_from_slice(&public_parameter_group_1[524288..524291]);
    new_group_1[2075..2078].copy_from_slice(&public_parameter_group_1[1048576..1048579]);

    let new_srs = KZGCommitmentSchemeBN254 {
        public_parameter_group_2,
        public_parameter_group_1: new_group_1,
    };

    let bytes = new_srs.to_unchecked_bytes().unwrap();
    path.push("srs-padding.bin");
    save_to_file(&bytes, path);
}

// cargo run --release --features="gen no_vk" --bin gen-params all "./parameters"
fn gen_all(directory: PathBuf) {
    gen_premutation(directory.clone());
    gen_vk(directory.clone());
    cut_srs(directory)
}

fn save_to_file(params_ser: &[u8], out_filename: ark_std::path::PathBuf) {
    use ark_std::io::Write;
    let filename = out_filename.to_str().unwrap();
    let mut f = ark_std::fs::File::create(&filename).expect("Unable to create file");
    f.write_all(params_ser).expect("Unable to write data");
}
