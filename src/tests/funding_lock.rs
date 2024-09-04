use crate::ContractUtil;
use ckb_testtool::{
    ckb_hash::blake2b_256,
    ckb_types::{core::TransactionBuilder, prelude::*},
};
use musig2::{
    CompactSignature, FirstRound, KeyAggContext, PartialSignature, SecNonceSpices,
};
use secp256k1::{
    rand::{self, RngCore},
    PublicKey, Secp256k1, SecretKey,
};
use sha2::{Digest};
use crate::cell_message::cell::MoleculeStructFlag;
use crate::cells::funding_lock::{FundingCell, FundingWitness};
use crate::cells::funding_lock_err::{FundingErrCell, FundingErrWitness};
use crate::prelude::ContextExt;

const EMPTY_WITNESS_ARGS: [u8; 16] = [16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0];

#[test]
fn test_funding_lock() {
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");


    // generate two random secret keys
    let sec_key_1 = SecretKey::new(&mut rand::thread_rng());
    let sec_key_2 = SecretKey::new(&mut rand::thread_rng());

    // public key aggregation
    let secp256k1 = Secp256k1::new();
    let pub_key_1 = sec_key_1.public_key(&secp256k1);
    let pub_key_2 = sec_key_2.public_key(&secp256k1);
    let key_agg_ctx = KeyAggContext::new(vec![pub_key_1, pub_key_2]).unwrap();
    let aggregated_pub_key: PublicKey = key_agg_ctx.aggregated_pubkey();
    let x_only_pub_key = aggregated_pub_key.x_only_public_key().0.serialize();

    // prepare scripts
    let pub_key_hash = blake2b_256(x_only_pub_key);


    let mut fc = FundingCell {
        lock_arg: <[u8; 20]>::try_from(pub_key_hash[0..20].to_vec().as_slice()).unwrap(),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_outpoint(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    let tx = ct.context.complete_tx(tx);

    // sign and add witness
    let tx_hash: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let version = 0u64.to_le_bytes();
    let binding = tx.clone().inputs().get(0).unwrap().previous_output();
    let funding_out_point = binding.as_slice();

    let message = blake2b_256(
        [
            version.to_vec(),
            funding_out_point.to_vec(),
            tx_hash.to_vec(),
        ]
            .concat(),
    );

    let mut first_round_1 = {
        let mut nonce_seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut nonce_seed);

        FirstRound::new(
            key_agg_ctx.clone(),
            nonce_seed,
            0,
            SecNonceSpices::new()
                .with_seckey(sec_key_1)
                .with_message(&message),
        )
            .unwrap()
    };

    let mut first_round_2 = {
        let mut nonce_seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut nonce_seed);

        FirstRound::new(
            key_agg_ctx,
            nonce_seed,
            1,
            SecNonceSpices::new()
                .with_seckey(sec_key_2)
                .with_message(&message),
        )
            .unwrap()
    };

    first_round_1
        .receive_nonce(1, first_round_2.our_public_nonce())
        .unwrap();
    first_round_2
        .receive_nonce(0, first_round_1.our_public_nonce())
        .unwrap();

    let mut second_round_1 = first_round_1.finalize(sec_key_1, &message).unwrap();
    let mut second_round_2 = first_round_2.finalize(sec_key_2, &message).unwrap();
    let signature_1: PartialSignature = second_round_1.our_signature();
    let signature_2: PartialSignature = second_round_2.our_signature();

    second_round_1.receive_signature(1, signature_2).unwrap();
    let aggregated_signature_1: CompactSignature = second_round_1.finalize().unwrap();
    second_round_2.receive_signature(0, signature_1).unwrap();
    let aggregated_signature_2: CompactSignature = second_round_2.finalize().unwrap();

    assert_eq!(aggregated_signature_1, aggregated_signature_2);
    // println!("signature: {:?}", aggregated_signature_1.to_bytes());


    fc.witness = Some(FundingWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        version: 0u64,
        funding_out_point: <[u8; 36]>::try_from(&funding_out_point[0..36]).unwrap(),
        pubkey: x_only_pub_key,
        signature: aggregated_signature_1.into(),
    });

    let tx = ct.replace_output(tx, funding_lock_contract, None, &fc, 500, 0);


    // run
    let cycles = ct.context
        .should_be_passed(&tx, 100000000).unwrap();
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_multiple_inputs_err() {
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");


    let fc = FundingCell {
        lock_arg: [0; 20],
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_outpoint(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    let tx = ct.context.complete_tx(tx);
    match ct.context
        .should_be_failed(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            assert!(err.to_string().contains("code 5"))
        }
    };
}

#[test]
fn test_empty_witness_args_error() {
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");


    let fc = FundingCell {
        lock_arg: [0; 20],
        type_arg: None,
        data: 0,
        witness: Some(FundingWitness {
            empty_witness_args: [1; 16],
            version: 0u64,
            funding_out_point: [1; 36],
            pubkey: [1; 32],
            signature: [1; 64],
        }),
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_outpoint(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    let tx = ct.context.complete_tx(tx);
    match ct.context
        .should_be_failed(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 7"))
        }
    };
}

#[test]
fn test_witness_len_zero_error() {
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");


    let fc = FundingCell {
        lock_arg: [0; 20],
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_outpoint(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    let tx = ct.context.complete_tx(tx);
    match ct.context
        .should_be_failed(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code -1"))
        }
    };
}

#[test]
fn test_witness_len_error() {
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");


    let fc = FundingErrCell {
        lock_arg: [0; 20],
        type_arg: None,
        data: 0,
        witness: Some(FundingErrWitness {
            empty_witness_args: EMPTY_WITNESS_ARGS,
            version: 0u64,
            funding_out_point: [1; 36],
            pubkey: [1; 32],
            signature: [1; 64],
            err:[1;32]
        }),
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_outpoint(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    let tx = ct.context.complete_tx(tx);
    match ct.context
        .should_be_failed(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 6"))
        }
    };
}

#[test]
fn test_funding_out_point_error() {
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");


    let fc = FundingCell {
        lock_arg: [0; 20],
        type_arg: None,
        data: 0,
        witness: Some(FundingWitness {
            empty_witness_args: EMPTY_WITNESS_ARGS,
            version: 0u64,
            funding_out_point: [1; 36],
            pubkey: [1; 32],
            signature: [1; 64],
        }),
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_outpoint(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    let tx = ct.context.complete_tx(tx);
    match ct.context
        .should_be_failed(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 8"))
        }
    };
}

#[test]
fn test_exec_cell_error() {
    let mut ct = ContractUtil::new();
    let funding_lock_contract = ct.deploy_contract("funding-lock");
    let auth_contract = ct.deploy_contract("auth");

    let fc = FundingCell {
        lock_arg: [0; 20],
        type_arg: None,
        data: 0,
        witness: Some(FundingWitness {
            empty_witness_args: EMPTY_WITNESS_ARGS,
            version: 0u64,
            funding_out_point: [1; 36],
            pubkey: [1; 32],
            signature: [1; 64],
        }),
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, funding_lock_contract.clone(), None, &fc, 500);

    let binding = tx.clone().inputs().get(0).unwrap().previous_output();
    let funding_out_point = binding.as_slice();

    let fc = FundingCell {
        lock_arg: [0; 20],
        type_arg: None,
        data: 0,
        witness: Some(FundingWitness {
            empty_witness_args: EMPTY_WITNESS_ARGS,
            version: 0u64,
            funding_out_point: <[u8; 36]>::try_from(&funding_out_point[0..36]).unwrap(),
            pubkey: [1; 32],
            signature: [1; 64],
        }),
        struct_flag: MoleculeStructFlag::default(),
    };


    let tx = ct.add_outpoint(tx, funding_lock_contract.clone(), None, &fc, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    let tx = ct.context.complete_tx(tx);
    match ct.context
        .should_be_failed(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 110"))
        }
    };
}

#[test]
fn change_tx_message() {}