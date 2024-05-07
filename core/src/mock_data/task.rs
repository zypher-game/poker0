use ark_ed_on_bn254::EdwardsProjective;
use ark_std::rand::SeedableRng;
use hashbrown::HashMap;
use rand_chacha::ChaChaRng;
use zplonk::shuffle::Ciphertext;

use crate::{
    cards::{reveal0, unmask, verify_reveal0, CryptoCard, ENCODING_CARDS_MAPPING},
    combination::CryptoCardCombination,
    left_rotate,
    play::{PlayAction, PlayerEnvBuilder},
    schnorr::KeyPair,
    task::Task,
    CiphertextAffine, CiphertextAffineRepr,
};

pub fn mock_task() -> Task {
    let mut rng = ChaChaRng::from_seed([0u8; 32]);
    let a = r##"
    {"private_key":[199,118,160,104,194,165,207,217,144,164,212,71,181,14,72,177,120,188,233,202,221,172,188,248,216,225,221,164,66,122,141,0],"public_key":[198,139,168,239,228,144,7,82,187,206,88,35,111,79,134,63,62,56,231,98,241,53,113,86,165,66,74,181,254,47,228,162]}
     "##;
    let b = r##"
    {"private_key":[159,254,8,205,249,43,125,141,199,65,215,232,162,202,51,1,249,136,244,62,94,79,52,54,199,126,33,11,174,2,3,5],"public_key":[94,53,185,211,206,160,229,182,98,91,232,221,106,21,242,162,153,250,55,49,138,11,184,221,250,42,225,150,125,187,176,46]}
      "##;
    let c = r##"
    {"private_key":[210,122,124,112,222,6,173,188,209,175,27,133,180,55,104,153,48,93,57,111,150,225,178,193,249,87,191,109,19,153,228,5],"public_key":[59,196,247,52,176,21,214,60,244,109,153,119,203,235,57,221,223,53,180,218,93,220,193,243,153,117,147,135,103,58,27,175]}
       "##;

    let alice: KeyPair = serde_json::from_str(&a).unwrap();
    let bob: KeyPair = serde_json::from_str(&b).unwrap();
    let charlie: KeyPair = serde_json::from_str(&c).unwrap();

    let card_serialized = r##"
  [{"e1":[25,132,147,10,175,21,184,81,174,58,7,190,11,204,23,233,91,45,151,4,197,230,60,36,92,220,155,172,169,111,138,174],"e2":[51,146,64,212,178,79,63,62,235,32,76,164,169,63,147,13,63,159,34,11,56,110,58,30,217,222,230,81,199,254,64,18]},{"e1":[14,239,133,109,234,141,107,206,194,186,26,207,33,73,112,46,85,186,149,71,229,85,243,175,0,85,16,200,245,104,230,22],"e2":[23,197,253,141,129,42,224,76,219,34,27,115,67,87,243,23,210,34,167,19,1,63,176,199,182,176,141,141,199,54,173,175]},{"e1":[64,23,232,121,241,31,160,120,217,100,157,126,175,8,8,3,217,53,60,80,77,251,151,5,250,83,132,255,236,241,34,45],"e2":[69,15,94,55,202,245,177,246,127,180,16,39,106,85,152,229,2,5,177,240,4,180,247,165,245,60,148,51,251,151,177,29]},{"e1":[16,59,121,240,77,107,176,102,246,69,191,159,232,171,151,114,38,170,147,241,148,252,122,172,124,0,180,244,175,205,32,13],"e2":[214,167,223,163,133,224,49,191,252,179,25,159,114,239,88,154,82,161,162,162,39,205,68,11,160,131,129,233,207,199,126,25]},{"e1":[253,47,181,194,69,155,110,186,180,148,197,245,188,14,42,245,120,251,229,48,211,243,98,217,226,19,105,188,254,76,125,166],"e2":[178,246,72,87,172,216,111,31,9,1,120,47,200,24,3,144,72,250,160,29,117,80,4,107,209,182,17,189,136,49,53,167]},{"e1":[128,148,18,101,146,55,175,95,177,105,63,175,130,50,91,81,127,116,35,112,5,5,145,89,67,63,139,155,154,60,254,160],"e2":[43,213,30,93,182,66,53,96,219,93,207,144,242,90,146,226,198,156,212,199,79,198,176,133,78,94,243,71,170,217,27,158]},{"e1":[107,8,199,131,216,244,65,160,69,7,168,13,102,30,17,88,2,179,24,145,149,31,159,70,89,29,139,46,34,231,140,1],"e2":[212,121,106,163,6,112,95,98,149,195,90,134,105,19,187,160,12,2,128,90,180,183,242,60,7,69,251,158,41,243,218,129]},{"e1":[171,240,28,112,129,137,218,62,154,116,0,10,7,174,89,73,102,11,199,250,112,171,118,197,47,93,203,46,191,100,206,25],"e2":[138,240,237,96,67,2,251,88,141,230,86,126,113,198,123,47,165,66,70,43,133,16,61,102,79,61,142,156,82,176,53,167]},{"e1":[252,0,232,77,244,21,168,183,17,175,213,195,13,70,191,72,221,232,191,93,29,147,247,221,219,45,23,22,236,220,228,166],"e2":[215,147,125,145,249,73,51,193,19,176,145,176,45,217,164,178,183,192,234,169,141,143,124,125,199,32,223,4,22,18,185,130]},{"e1":[237,11,64,26,39,154,196,174,41,212,137,162,250,11,238,188,85,27,202,177,106,159,204,20,189,146,150,218,195,75,106,156],"e2":[12,42,10,160,3,167,218,232,78,164,133,223,220,218,105,171,21,164,223,149,125,113,119,117,200,113,202,255,120,231,212,146]},{"e1":[195,13,133,29,37,233,77,170,16,133,147,21,161,42,173,15,155,200,226,79,56,76,145,185,238,192,132,172,39,27,61,42],"e2":[147,216,229,81,27,163,166,39,230,20,113,93,57,50,57,42,252,103,206,254,45,189,36,227,27,118,151,82,218,34,170,155]},{"e1":[236,230,231,177,145,115,76,155,114,154,230,192,57,178,4,254,170,44,194,172,118,238,147,11,149,87,4,37,197,190,242,165],"e2":[52,153,185,222,27,59,56,165,2,107,11,107,124,209,55,3,186,243,141,145,99,188,209,65,12,205,9,74,248,225,43,10]},{"e1":[71,124,71,207,93,24,205,249,119,169,184,201,174,12,97,23,16,136,179,130,174,71,142,29,83,37,164,184,103,178,89,45],"e2":[80,119,200,237,237,60,187,179,71,76,112,208,25,145,95,114,207,191,7,209,7,47,178,195,232,12,62,136,176,198,108,131]},{"e1":[182,162,31,113,0,136,239,200,64,112,53,84,135,146,242,175,182,109,163,251,46,226,176,172,56,182,197,67,67,90,149,5],"e2":[111,175,195,7,62,36,120,23,60,131,66,117,164,229,1,45,231,145,191,173,20,254,165,45,53,99,1,108,205,66,245,35]},{"e1":[176,30,239,181,72,154,55,215,225,228,106,47,61,83,2,126,221,1,2,90,152,216,164,99,118,73,89,64,117,180,0,7],"e2":[162,126,221,41,168,107,85,27,162,235,72,221,142,61,196,237,6,200,57,6,21,114,252,72,80,83,136,148,234,38,165,7]},{"e1":[167,62,30,220,220,64,148,243,75,103,25,221,9,125,102,148,58,138,131,38,88,207,153,211,5,29,152,113,145,212,32,153],"e2":[113,157,111,121,178,107,182,170,242,208,238,255,196,159,248,227,73,89,49,182,254,52,183,58,144,179,57,161,4,86,145,159]},{"e1":[216,74,103,249,45,226,103,33,210,210,242,86,109,53,6,72,109,50,72,108,104,52,11,80,192,119,81,140,103,33,167,26],"e2":[189,221,26,11,99,62,234,120,81,111,213,27,53,28,245,122,85,93,193,40,59,214,9,133,220,82,165,228,224,22,77,167]},{"e1":[222,148,245,243,57,255,144,96,198,15,140,72,43,169,180,151,215,152,3,166,241,90,117,192,149,125,37,239,75,166,16,37],"e2":[108,175,143,190,23,48,72,125,22,71,129,98,123,211,220,49,41,164,18,213,117,170,149,247,135,113,18,241,32,149,215,134]},{"e1":[195,41,115,209,123,26,29,99,66,137,7,204,118,22,127,16,75,79,247,229,108,227,248,74,132,46,0,199,41,159,214,47],"e2":[236,159,200,164,203,121,9,131,46,236,124,29,53,241,89,12,149,15,255,82,54,253,134,121,132,165,139,215,149,137,235,158]},{"e1":[237,200,205,34,157,67,249,41,135,43,229,71,148,143,234,42,158,7,56,232,187,81,193,143,55,87,140,48,18,11,10,148],"e2":[3,61,164,139,244,17,47,67,121,18,92,216,99,253,102,132,51,92,133,92,157,233,208,9,136,107,58,42,202,131,211,33]},{"e1":[49,208,70,246,145,135,17,106,217,6,15,57,148,197,220,123,242,243,86,170,87,103,38,156,77,76,100,26,100,91,53,170],"e2":[198,85,195,245,149,97,251,67,132,91,93,55,64,189,116,26,239,239,156,246,210,145,129,159,193,185,239,173,68,191,240,168]},{"e1":[126,68,53,60,26,5,11,147,210,182,17,192,158,168,37,186,63,217,193,85,27,145,204,182,249,254,238,166,9,172,94,138],"e2":[60,220,184,59,32,168,41,148,69,129,156,142,213,252,212,94,116,82,121,84,199,158,179,156,85,67,53,204,3,134,190,12]},{"e1":[229,138,183,193,78,16,10,148,239,196,115,88,84,201,176,251,88,156,183,203,192,189,222,206,12,170,255,191,111,112,63,35],"e2":[200,125,195,49,169,190,246,161,107,238,198,196,171,3,211,12,189,232,109,54,245,125,139,16,16,32,33,178,119,108,223,21]},{"e1":[152,167,158,16,56,169,117,35,185,67,204,248,148,197,141,142,114,225,5,133,217,1,28,163,235,8,91,88,158,200,55,4],"e2":[73,85,65,152,173,206,212,53,75,138,7,52,56,217,181,48,112,177,51,114,196,30,93,128,101,65,82,121,103,51,67,39]},{"e1":[42,80,157,43,3,153,127,215,211,165,201,44,115,31,166,82,74,4,224,190,238,227,81,184,96,234,7,68,201,126,22,132],"e2":[146,179,173,207,221,252,188,226,6,252,110,174,62,90,91,171,186,67,53,4,134,152,214,37,237,78,245,41,73,179,62,172]},{"e1":[147,65,83,69,52,32,29,123,59,54,85,16,204,80,167,13,80,70,169,152,180,131,88,56,144,236,250,216,71,61,112,38],"e2":[210,115,171,47,251,210,181,52,84,242,169,71,209,244,37,179,164,175,151,208,221,251,211,33,176,23,155,15,77,240,40,146]},{"e1":[32,196,23,100,134,11,46,142,252,101,12,164,140,140,155,63,224,97,109,227,99,27,242,12,197,104,109,109,174,201,152,46],"e2":[17,108,85,132,116,65,131,228,7,20,22,171,188,25,45,219,166,138,87,126,154,50,178,248,34,206,28,219,5,67,215,2]},{"e1":[91,223,13,93,79,143,156,104,107,26,59,31,34,173,70,13,142,115,153,219,100,46,196,211,242,45,15,162,154,225,66,45],"e2":[83,67,127,227,29,208,98,173,254,77,157,204,147,148,222,174,175,181,41,203,228,2,205,227,132,224,170,255,43,169,192,32]},{"e1":[235,53,123,156,72,1,187,48,144,72,154,216,102,251,210,175,2,78,70,179,115,17,66,102,24,214,23,193,186,205,201,2],"e2":[200,29,54,16,102,177,17,230,224,170,101,223,7,1,228,183,61,140,106,193,207,131,255,107,53,20,255,156,252,76,99,174]},{"e1":[29,136,148,20,18,75,161,90,59,35,111,49,169,131,190,2,105,60,171,173,245,222,57,61,171,134,67,230,15,170,219,169],"e2":[216,182,117,226,160,80,151,229,39,57,96,182,227,233,173,213,52,48,76,244,217,55,75,201,229,91,119,79,172,115,213,3]},{"e1":[14,46,63,233,87,216,9,252,145,165,58,205,170,251,248,207,106,86,195,128,219,133,18,30,139,188,247,247,248,47,233,158],"e2":[153,47,150,99,38,122,42,41,120,186,158,199,215,164,203,163,255,215,153,223,142,233,190,82,243,251,249,85,179,142,48,166]},{"e1":[73,254,118,115,24,176,125,247,22,12,202,102,118,226,140,222,220,172,65,110,239,224,202,35,62,75,201,115,93,2,152,0],"e2":[219,244,22,118,167,4,144,63,30,23,237,50,139,58,111,28,81,240,65,8,105,4,213,151,56,216,3,229,231,14,222,131]},{"e1":[52,39,231,124,35,141,66,249,116,105,206,130,54,8,88,165,124,122,64,84,140,117,232,150,169,121,74,191,218,2,162,154],"e2":[215,234,19,175,55,244,102,187,21,162,226,224,250,209,132,165,63,226,13,76,121,150,75,224,37,114,94,112,191,249,234,22]},{"e1":[36,135,29,5,216,233,234,102,138,124,67,74,152,217,120,11,62,188,51,134,10,127,30,248,120,170,249,166,22,156,244,169],"e2":[138,155,181,55,88,243,23,225,134,105,182,179,188,99,233,244,138,34,53,34,247,205,48,152,195,207,251,212,118,22,196,137]},{"e1":[153,3,120,122,250,151,39,76,215,54,184,201,78,207,40,54,25,236,81,112,141,196,213,166,10,163,128,188,81,138,255,5],"e2":[177,136,165,146,1,250,54,150,142,221,32,178,248,177,84,193,29,172,165,189,74,212,126,8,138,235,121,229,133,213,64,11]},{"e1":[56,72,98,131,236,150,246,18,120,255,174,183,15,24,117,73,52,95,129,79,186,187,12,41,115,110,74,21,57,79,2,12],"e2":[239,209,166,31,175,134,171,29,91,208,58,92,115,169,28,224,246,202,246,240,63,244,22,56,161,212,67,188,25,76,30,133]},{"e1":[97,212,175,225,65,51,93,64,110,189,64,92,196,187,230,44,211,114,157,228,188,166,171,115,212,173,246,182,142,56,205,171],"e2":[177,130,112,167,44,203,34,34,140,180,142,129,243,111,81,103,155,87,95,90,117,9,252,241,46,247,75,188,2,60,72,21]},{"e1":[177,33,78,209,206,32,110,223,15,7,73,82,87,95,23,43,204,56,109,244,34,11,179,131,50,59,127,75,28,163,19,156],"e2":[71,101,164,11,117,250,36,92,199,65,113,238,192,7,245,185,174,155,7,76,237,21,190,69,254,138,188,78,13,47,216,157]},{"e1":[45,44,159,93,184,178,96,226,58,206,206,22,143,255,165,93,20,80,136,204,222,232,32,57,171,122,234,202,61,227,32,12],"e2":[35,227,195,180,77,236,66,0,211,204,38,165,155,174,41,103,25,47,62,208,205,52,105,210,242,224,216,235,147,172,145,168]},{"e1":[62,219,163,186,143,213,19,98,136,11,198,54,216,198,65,108,63,101,29,165,54,142,8,0,124,193,51,58,63,124,39,39],"e2":[81,109,189,203,57,105,228,193,66,31,182,34,101,176,133,166,60,136,64,105,79,227,12,164,218,190,207,200,110,52,41,137]},{"e1":[148,172,193,125,80,45,232,142,191,123,204,86,161,80,190,240,209,204,119,239,228,205,91,5,106,44,44,232,59,193,162,7],"e2":[161,187,116,201,70,120,168,18,141,34,2,38,168,193,122,48,119,124,136,134,234,13,140,80,192,4,175,242,223,118,35,137]},{"e1":[255,23,84,234,64,212,40,47,53,192,26,164,48,188,86,64,171,89,83,98,53,81,45,173,240,116,233,166,167,63,238,5],"e2":[148,90,238,205,62,11,160,138,171,133,70,170,101,206,111,204,148,24,85,199,98,14,253,63,220,251,19,210,245,33,37,20]},{"e1":[225,230,172,1,251,78,64,95,23,6,211,157,101,37,63,183,204,185,254,196,54,227,93,138,64,141,2,73,67,121,92,145],"e2":[55,146,98,149,207,150,128,200,147,31,207,85,135,117,121,79,55,49,136,3,17,24,214,151,135,192,209,142,247,14,53,149]},{"e1":[246,43,236,96,69,68,199,184,204,63,194,18,82,27,242,97,141,8,57,76,181,85,60,48,221,113,24,62,81,121,230,141],"e2":[177,40,18,2,153,173,177,185,114,235,226,57,103,253,90,213,40,219,92,224,184,224,7,113,91,15,167,113,132,30,115,9]},{"e1":[148,237,176,172,20,1,127,80,71,3,33,85,136,195,40,11,43,39,52,239,43,57,161,65,104,18,125,165,8,26,250,1],"e2":[7,236,55,32,140,181,60,233,221,22,62,201,202,7,252,49,45,74,194,25,237,98,9,253,217,217,29,133,190,20,105,6]},{"e1":[202,14,147,160,76,246,112,242,191,137,220,79,108,41,134,233,83,245,210,247,201,206,87,212,192,81,68,145,75,252,194,26],"e2":[212,113,198,143,35,112,158,10,177,129,168,0,24,17,69,139,222,225,110,157,214,92,66,90,138,195,214,198,52,77,59,14]},{"e1":[181,109,65,255,72,34,138,248,253,40,147,43,113,138,107,227,233,1,132,84,15,104,25,215,53,226,91,10,242,218,229,142],"e2":[174,198,227,28,147,8,55,129,203,40,58,96,239,36,192,209,229,139,142,81,80,170,83,48,85,71,87,36,128,108,46,2]},{"e1":[108,93,244,90,34,217,140,253,99,43,202,34,132,172,139,115,67,63,215,235,3,161,98,65,137,28,55,48,95,86,178,156],"e2":[171,67,124,252,41,175,6,236,251,235,129,18,217,29,230,7,97,24,161,187,49,81,141,232,250,118,251,111,12,148,183,155]}]
     "##;

    let shuffle_cards: Vec<Ciphertext<EdwardsProjective>> =
        serde_json::from_str(&card_serialized).unwrap();

    let shuffle_cards = shuffle_cards
        .iter()
        .map(|x| (*x).into())
        .collect::<Vec<CiphertextAffineRepr>>();

    let alice_deck = &shuffle_cards[..16];
    let bob_deck = &shuffle_cards[16..32];
    let charlie_deck = &shuffle_cards[32..];

    let mut a_card = vec![];
    let mut b_card = vec![];
    let mut c_card = vec![];

    let mut reveal_proofs = HashMap::new();

    for card in alice_deck.iter() {
        let (reveal_card_b, reveal_proof_b) = reveal0(&mut rng, &bob, card).unwrap();
        let (reveal_card_c, reveal_proof_c) = reveal0(&mut rng, &charlie, card).unwrap();
        let (reveal_card_a, reveal_proof_a) = reveal0(&mut rng, &alice, card).unwrap();
        verify_reveal0(&bob.public_key, card, &reveal_card_b, &reveal_proof_b).unwrap();
        verify_reveal0(&charlie.public_key, card, &reveal_card_c, &reveal_proof_c).unwrap();
        verify_reveal0(&alice.public_key, card, &reveal_card_a, &reveal_proof_a).unwrap();

        let reveals = vec![reveal_card_b, reveal_card_a, reveal_card_c];
        let unmasked_card = unmask(card, &reveals);

        let opened_card = ENCODING_CARDS_MAPPING.get(&unmasked_card.0).unwrap();
        a_card.push(opened_card);

        reveal_proofs.insert(
            card,
            vec![
                (reveal_card_b, reveal_proof_b, bob.get_public_key()),
                (reveal_card_c, reveal_proof_c, charlie.get_public_key()),
                (reveal_card_a, reveal_proof_a, alice.get_public_key()),
            ],
        );
    }

    for card in bob_deck.iter() {
        let (reveal_card_c, reveal_proof_c) = reveal0(&mut rng, &charlie, card).unwrap();
        let (reveal_card_a, reveal_proof_a) = reveal0(&mut rng, &alice, card).unwrap();
        let (reveal_card_b, reveal_proof_b) = reveal0(&mut rng, &bob, card).unwrap();
        verify_reveal0(&bob.public_key, card, &reveal_card_b, &reveal_proof_b).unwrap();
        verify_reveal0(&charlie.public_key, card, &reveal_card_c, &reveal_proof_c).unwrap();
        verify_reveal0(&alice.public_key, card, &reveal_card_a, &reveal_proof_a).unwrap();

        let reveals = vec![reveal_card_b, reveal_card_a, reveal_card_c];
        let unmasked_card = unmask(card, &reveals);

        let opened_card = ENCODING_CARDS_MAPPING.get(&unmasked_card.0).unwrap();
        b_card.push(opened_card);

        reveal_proofs.insert(
            card,
            vec![
                (reveal_card_a, reveal_proof_a, alice.get_public_key()),
                (reveal_card_b, reveal_proof_b, bob.get_public_key()),
                (reveal_card_c, reveal_proof_c, charlie.get_public_key()),
            ],
        );
    }

    for card in charlie_deck.iter() {
        let (reveal_card_c, reveal_proof_c) = reveal0(&mut rng, &charlie, card).unwrap();
        let (reveal_card_a, reveal_proof_a) = reveal0(&mut rng, &alice, card).unwrap();
        let (reveal_card_b, reveal_proof_b) = reveal0(&mut rng, &bob, card).unwrap();
        verify_reveal0(&bob.public_key, card, &reveal_card_b, &reveal_proof_b).unwrap();
        verify_reveal0(&charlie.public_key, card, &reveal_card_c, &reveal_proof_c).unwrap();
        verify_reveal0(&alice.public_key, card, &reveal_card_a, &reveal_proof_a).unwrap();

        let reveals = vec![reveal_card_b, reveal_card_a, reveal_card_c];
        let unmasked_card = unmask(card, &reveals);

        let opened_card = ENCODING_CARDS_MAPPING.get(&unmasked_card.0).unwrap();
        c_card.push(opened_card);

        reveal_proofs.insert(
            card,
            vec![
                (reveal_card_c, reveal_proof_c, charlie.get_public_key()),
                (reveal_card_a, reveal_proof_a, alice.get_public_key()),
                (reveal_card_b, reveal_proof_b, bob.get_public_key()),
            ],
        );
    }

    let players_key = vec![
        alice.get_public_key(),
        bob.get_public_key(),
        charlie.get_public_key(),
    ];

    let players_hand = vec![
        alice_deck
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect(),
        bob_deck
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect(),
        charlie_deck
            .iter()
            .map(|x| CryptoCard(CiphertextAffine::from(*x)))
            .collect(),
    ];

    //  ---------------round 0--------------------
    let bob_play_0_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(bob_deck[0].into()),
            CryptoCard(bob_deck[6].into()),
        )))
        .reveals(&[
            reveal_proofs.get(&bob_deck[0]).unwrap().clone(),
            reveal_proofs.get(&bob_deck[6]).unwrap().clone(),
        ])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_0_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(1)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(charlie_deck[7].into()),
            CryptoCard(charlie_deck[8].into()),
        )))
        .reveals(&[
            reveal_proofs.get(&charlie_deck[7]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[8]).unwrap().clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_0_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(2)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(alice_deck[11].into()),
            CryptoCard(alice_deck[14].into()),
        )))
        .reveals(&[
            reveal_proofs.get(&alice_deck[11]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[14]).unwrap().clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_0_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(3)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_0_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(0)
        .turn_id(4)
        .action(PlayAction::PAAS)
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let round_0 = vec![
        bob_play_0_0,
        charlie_play_0_0,
        alice_play_0_0,
        bob_play_0_1,
        charlie_play_0_1,
    ];

    //  ---------------round 1--------------------
    let alice_play_1_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Straight(vec![
            CryptoCard(alice_deck[4].into()),
            CryptoCard(alice_deck[5].into()),
            CryptoCard(alice_deck[1].into()),
            CryptoCard(alice_deck[13].into()),
            CryptoCard(alice_deck[10].into()),
            CryptoCard(alice_deck[8].into()),
        ])))
        .reveals(&[
            reveal_proofs.get(&alice_deck[4]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[5]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[1]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[13]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[10]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[8]).unwrap().clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_1_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_1_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(1)
        .turn_id(2)
        .action(PlayAction::PAAS)
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let round_1 = vec![alice_play_1_0, bob_play_1_0, charlie_play_1_0];

    //  ---------------round 2--------------------
    let alice_play_2_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::ThreeWithOne(
            CryptoCard(alice_deck[3].into()),
            CryptoCard(alice_deck[7].into()),
            CryptoCard(alice_deck[15].into()),
            CryptoCard(alice_deck[9].into()),
        )))
        .reveals(&[
            reveal_proofs.get(&alice_deck[3]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[7]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[15]).unwrap().clone(),
            reveal_proofs.get(&alice_deck[9]).unwrap().clone(),
        ])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_2_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_2_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(2)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::ThreeWithOne(
            CryptoCard(charlie_deck[0].into()),
            CryptoCard(charlie_deck[5].into()),
            CryptoCard(charlie_deck[14].into()),
            CryptoCard(charlie_deck[9].into()),
        )))
        .reveals(&[
            reveal_proofs.get(&charlie_deck[0]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[5]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[14]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[9]).unwrap().clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_2_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(3)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_2_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(2)
        .turn_id(4)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let round_2 = vec![
        alice_play_2_0,
        bob_play_2_0,
        charlie_play_2_0,
        alice_play_2_1,
        bob_play_2_1,
    ];

    //  ---------------round 3--------------------
    let charlie_play_3_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::ThreeWithOne(
            CryptoCard(charlie_deck[12].into()),
            CryptoCard(charlie_deck[13].into()),
            CryptoCard(charlie_deck[15].into()),
            CryptoCard(charlie_deck[2].into()),
        )))
        .reveals(&[
            reveal_proofs.get(&charlie_deck[12]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[13]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[15]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[2]).unwrap().clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_3_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(1)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_3_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(3)
        .turn_id(2)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let round_3 = vec![charlie_play_3_0, alice_play_3_0, bob_play_3_0];

    //  ---------------round 4--------------------
    let charlie_play_4_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            charlie_deck[6].into(),
        ))))
        .reveals(&[reveal_proofs.get(&charlie_deck[6]).unwrap().clone()])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_4_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(1)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            alice_deck[0].into(),
        ))))
        .reveals(&[reveal_proofs.get(&alice_deck[0]).unwrap().clone()])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_4_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(2)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            bob_deck[7].into(),
        ))))
        .reveals(&[reveal_proofs.get(&bob_deck[7]).unwrap().clone()])
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_4_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(3)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            charlie_deck[3].into(),
        ))))
        .reveals(&[reveal_proofs.get(&charlie_deck[3]).unwrap().clone()])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_4_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(4)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_4_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(4)
        .turn_id(5)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let round_4 = vec![
        charlie_play_4_0,
        alice_play_4_0,
        bob_play_4_0,
        charlie_play_4_1,
        alice_play_4_1,
        bob_play_4_1,
    ];

    //  ---------------round 5--------------------
    let charlie_play_5_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            charlie_deck[4].into(),
        ))))
        .reveals(&[reveal_proofs.get(&charlie_deck[4]).unwrap().clone()])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_5_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(1)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            alice_deck[2].into(),
        ))))
        .reveals(&[reveal_proofs.get(&alice_deck[2]).unwrap().clone()])
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_5_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(2)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let charlie_play_5_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(3)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Single(CryptoCard(
            charlie_deck[10].into(),
        ))))
        .reveals(&[reveal_proofs.get(&charlie_deck[10]).unwrap().clone()])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let alice_play_5_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(4)
        .action(PlayAction::PAAS)
        .build_and_sign(&alice, &mut rng)
        .unwrap();

    let bob_play_5_1 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(5)
        .turn_id(5)
        .action(PlayAction::PAAS)
        .build_and_sign(&bob, &mut rng)
        .unwrap();

    let round_5 = vec![
        charlie_play_5_0,
        alice_play_5_0,
        bob_play_5_0,
        charlie_play_5_1,
        alice_play_5_1,
        bob_play_5_1,
    ];

    //  ---------------round 6--------------------
    let charlie_play_6_0 = PlayerEnvBuilder::new()
        .room_id(1)
        .round_id(6)
        .turn_id(0)
        .action(PlayAction::PLAY)
        .play_cards(Some(CryptoCardCombination::Pair(
            CryptoCard(charlie_deck[1].into()),
            CryptoCard(charlie_deck[11].into()),
        )))
        .reveals(&[
            reveal_proofs.get(&charlie_deck[1]).unwrap().clone(),
            reveal_proofs.get(&charlie_deck[11]).unwrap().clone(),
        ])
        .build_and_sign(&charlie, &mut rng)
        .unwrap();

    let round_6 = vec![charlie_play_6_0];

    let first_player = 1;

    let mut players_env = vec![
        round_0, round_1, round_2, round_3, round_4, round_5, round_6,
    ];
    players_env.iter_mut().for_each(|x| {
        x.iter_mut()
            .for_each(|y| y.sync_reveal_order(&left_rotate(&players_key, first_player)))
    });

    Task {
        room_id: 1,
        first_player,
        players_key,
        players_env,
        players_hand,
    }
}
