use ckb_std::since::{EpochNumberWithFraction, Since};
use ckb_testtool::{
    ckb_crypto::secp::Generator,
    ckb_hash::blake2b_256,
    ckb_types::{core::TransactionBuilder, prelude::*},
};

use ckb_testtool::ckb_types::prelude::{Builder, Entity};
use sha2::{Digest, Sha256};
use crate::cell_message::cell::MoleculeStructFlag;
use crate::cells::commitment_lock::{CommitmentArgErrCell, CommitmentCellNoHtlcAndPreImage, CommitmentHTCL1Cell, CommitmentHTCL1WithPreimageAndUDTCell, CommitmentHTCL2Cell, CommitmentHTCL2WithPriImageAndUDTCell, CommitmentMaxErrLenWitness, CommitmentMaxWitnessLenErrCell, CommitmentMinErrLenWitness, CommitmentMinWitnessLenErrCell, CommitmentPendinghtlc1WithPreimageWitness, CommitmentPendinghtlc1Witness, CommitmentPendinghtlc2WithPriImageWitness, CommitmentPendinghtlc2Witness, CommitmentWitness, CommitmentWitnessNoHtlcAndPreImage, PendingHtlc};
use crate::{ContractUtil};
const MAX_CYCLES: u64 = 10_000_000;

const EMPTY_WITNESS_ARGS: [u8; 16] = [16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0];
const BYTE_SHANNONS: u64 = 100_000_000;

#[test]
fn test_01() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");

    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();

    let mut commitment_cell = CommitmentCellNoHtlcAndPreImage::default();
    commitment_cell.lock_arg = <[u8; 20]>::try_from(args).unwrap();
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    // sign with revocation key
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let signature = revocation_key
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();

    commitment_cell.witness = Some(CommitmentWitnessNoHtlcAndPreImage {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        unlock_type: 255,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
    });
    let tx = ct.replace_output(tx, ct.alway_contract.clone(), None, &commitment_cell, 500, 0);

    println!("tx: {:?}", tx);

    // run
    let cycles = ct.context
        .verify_tx(&tx, 100000000)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}

#[test]
fn test_multiple_inputs_err() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let mut commitment_cell = CommitmentCellNoHtlcAndPreImage::default();
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            assert!(err.to_string().contains("code 5"))
        }
    };
}

#[test]
fn test_invalid_since_err() {
    // InvalidSince
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");

    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();

    let mut commitment_cell = CommitmentCellNoHtlcAndPreImage::default();
    commitment_cell.lock_arg = <[u8; 20]>::try_from(args).unwrap();
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input_with_since(tx, commitment_contract.clone(), None, &commitment_cell, Since::from_epoch(EpochNumberWithFraction::new(10, 0, 2), false).as_u64(), 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    // sign with revocation key
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let signature = revocation_key
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();

    commitment_cell.witness = Some(CommitmentWitnessNoHtlcAndPreImage {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        unlock_type: 255,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
    });
    let tx = ct.replace_output(tx, ct.alway_contract.clone(), None, &commitment_cell, 500, 0);

    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
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
fn test_invalid_unlock_type_err() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");

    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();

    let mut commitment_cell = CommitmentCellNoHtlcAndPreImage::default();
    commitment_cell.lock_arg = <[u8; 20]>::try_from(args).unwrap();
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    // sign with revocation key
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let signature = revocation_key
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();

    commitment_cell.witness = Some(CommitmentWitnessNoHtlcAndPreImage {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        unlock_type: 254,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
    });
    let tx = ct.replace_output(tx, ct.alway_contract.clone(), None, &commitment_cell, 500, 0);

    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            assert!(err.to_string().contains("code 7"))
        }
    }
}

#[test]
fn test_invalid_htlc_type_err() {
    println!("skip")
}

#[test]
fn test_args_len_error() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let commitment_cell = CommitmentArgErrCell::default();
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 9"))
        }
    };
}

#[test]
fn test_witness_len_err_too_min() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let mut commitment_cell = CommitmentMinWitnessLenErrCell::default();
    commitment_cell.witness = Some(CommitmentMinErrLenWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: 0,
        local_delay_pubkey_hash: [1; 20],
        revocation_pubkey_hash: [1; 20],
        signature: [1; 65],
    });
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 10"))
        }
    };
}

#[test]
fn test_witness_len_err_too_big() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let mut commitment_cell = CommitmentMaxWitnessLenErrCell::default();
    commitment_cell.witness = Some(CommitmentMaxErrLenWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: 0,
        local_delay_pubkey_hash: [1; 20],
        revocation_pubkey_hash: [1; 20],
        unlock_type: 0,
        signature: [1; 65],
        err: [1; 5],
    });
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 10"))
        }
    };
}

#[test]
fn test_empty_witness_args_err() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");

    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();

    let mut commitment_cell = CommitmentCellNoHtlcAndPreImage::default();
    commitment_cell.lock_arg = <[u8; 20]>::try_from(args).unwrap();
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    // sign with revocation key
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let signature = revocation_key
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();

    commitment_cell.witness = Some(CommitmentWitnessNoHtlcAndPreImage {
        empty_witness_args: [1; 16],
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        unlock_type: 255,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
    });
    let tx = ct.replace_output(tx, ct.alway_contract.clone(), None, &commitment_cell, 500, 0);

    println!("tx: {:?}", tx);

    // run
    let cycles = match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            assert!(err.to_string().contains("code 11"))
        }
    };
}

#[test]
fn test_witness_hash_err() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let mut commitment_cell = CommitmentCellNoHtlcAndPreImage::default();
    commitment_cell.witness = Some(CommitmentWitnessNoHtlcAndPreImage {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: 0,
        local_delay_pubkey_hash: [1; 20],
        revocation_pubkey_hash: [1; 20],
        unlock_type: 0,
        signature: [1; 65],
    });
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 12"))
        }
    };
}


#[test]
fn test_output_lock_err() {
    // deploy contract
    let mut ct = ContractUtil::new();
    let commitment_lock_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let udt_contract = ct.deploy_contract("simple_udt");


    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    let payment_amount1 = 1234567890u128;
    let payment_amount2 = 9876543210u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();
    let total_sudt_amount = 424242424242424242u128;

    let mut cc = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: Some([42; 32]),
        data: total_sudt_amount,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc, 1000);


    let new_witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&new_witness_script)[0..20].to_vec();

    let mut cc1 = CommitmentHTCL2WithPriImageAndUDTCell {
        // lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        // output lock err
        lock_arg: [0; 20],

        type_arg: cc.type_arg,
        data: total_sudt_amount - payment_amount1,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with remote_htlc_pubkey
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();
    let signature = remote_htlc_key1
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();


    cc1.witness = Some(CommitmentPendinghtlc2WithPriImageWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0b00000000,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000001,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x0,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
        preimage: preimage1,
    });
    let tx = ct.replace_output(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000, 0);


    println!("tx: {:?}", tx);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("OutputLockError");

    println!("err: {}", err);
    assert!(err.to_string().contains("code 14"))
}


#[test]
fn test_output_type_err() {
    // deploy contract
    let mut ct = ContractUtil::new();
    let commitment_lock_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let udt_contract = ct.deploy_contract("simple_udt");


    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    let payment_amount1 = 1234567890u128;
    let payment_amount2 = 9876543210u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();
    let total_sudt_amount = 424242424242424242u128;

    let mut cc = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: Some([42; 32]),
        data: total_sudt_amount,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc, 1000);


    let new_witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&new_witness_script)[0..20].to_vec();

    let mut cc1 = CommitmentHTCL2WithPriImageAndUDTCell {
        // lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        // output lock err
        lock_arg: [0; 20],
        // output type err
        // type_arg:cc.type_arg,
        type_arg: Some([40; 32]),
        data: total_sudt_amount - payment_amount1,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with remote_htlc_pubkey
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();
    let signature = remote_htlc_key1
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();


    cc1.witness = Some(CommitmentPendinghtlc2WithPriImageWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0b00000000,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000001,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x0,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
        preimage: preimage1,
    });
    let tx = ct.replace_output(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000, 0);


    println!("tx: {:?}", tx);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("OutputLockError");

    println!("err: {}", err);
    assert!(err.to_string().contains("code 14"))
}

#[test]
fn test_output_udt_amount_err() {
    let mut ct = ContractUtil::new();
    let commitment_lock_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let udt_contract = ct.deploy_contract("simple_udt");


    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    let payment_amount1 = 1234567890u128;
    let payment_amount2 = 9876543210u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();
    let total_sudt_amount = 424242424242424242u128;

    let mut cc = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: Some([42; 32]),
        data: total_sudt_amount,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc, 1000);


    let new_witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&new_witness_script)[0..20].to_vec();

    let mut cc1 = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),

        type_arg: cc.type_arg,
        // udtAmountErr
        // data: total_sudt_amount - payment_amount1,
        data: total_sudt_amount - payment_amount1 + 1,

        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with remote_htlc_pubkey
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();
    let signature = remote_htlc_key1
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();


    cc1.witness = Some(CommitmentPendinghtlc2WithPriImageWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0b00000000,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000001,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x0,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
        preimage: preimage1,
    });
    let tx = ct.replace_output(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000, 0);


    println!("tx: {:?}", tx);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("outputUdtAmountErr");

    println!("err: {}", err);
    assert!(err.to_string().contains("code 16"))
}

#[test]
fn test_preimage_received_err() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");

    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    let payment_amount1 = 5 * BYTE_SHANNONS as u128;
    let payment_amount2 = 8 * BYTE_SHANNONS as u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();


    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000011].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        Sha256::digest(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&witness_script)[0..20].to_vec();

    let mut cell = CommitmentHTCL2Cell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_contract.clone(), None, &cell, 1000);

    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &cell, 500);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &cell, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with revocation key
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let signature = revocation_key
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();

    cell.witness = Some(CommitmentPendinghtlc2Witness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000011,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(Sha256::digest(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x1,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
    });
    let tx = ct.replace_output(tx, ct.alway_contract.clone(), None, &cell, 500, 0);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("Error::PreimageError");
    assert!(err.to_string().contains("code 17"))
}

#[test]
fn test_preimage_offer_err() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");

    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    let payment_amount1 = 5 * BYTE_SHANNONS as u128;
    let payment_amount2 = 8 * BYTE_SHANNONS as u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();


    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000011].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        Sha256::digest(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&witness_script)[0..20].to_vec();

    let mut cell = CommitmentHTCL2Cell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_contract.clone(), None, &cell, 1000);

    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &cell, 500);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &cell, 500);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with revocation key
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let signature = revocation_key
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();

    cell.witness = Some(CommitmentPendinghtlc2Witness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000011,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(Sha256::digest(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x0,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
    });
    let tx = ct.replace_output(tx, ct.alway_contract.clone(), None, &cell, 500, 0);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("Error::PreimageError");
    println!("err:{:?}", err);
    assert!(err.to_string().contains("code 17"))
}

#[test]
fn test_udt_output_capacity_err() {
    let mut ct = ContractUtil::new();
    let commitment_lock_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let udt_contract = ct.deploy_contract("simple_udt");


    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    let payment_amount1 = 1234567890u128;
    let payment_amount2 = 9876543210u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();
    let total_sudt_amount = 424242424242424242u128;

    let mut cc = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: Some([42; 32]),
        data: total_sudt_amount,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc, 1000);


    let new_witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&new_witness_script)[0..20].to_vec();

    let mut cc1 = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),

        type_arg: cc.type_arg,
        data: total_sudt_amount - payment_amount1,

        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    // err: OutputCapacityError
    // let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 999);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with remote_htlc_pubkey
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();
    let signature = remote_htlc_key1
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();


    cc1.witness = Some(CommitmentPendinghtlc2WithPriImageWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0b00000000,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000001,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x0,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
        preimage: preimage1,
    });
    // err: OutputCapacityError
    // let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.replace_output(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 999, 0);


    println!("tx: {:?}", tx);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("OutputCapacityError");

    println!("err: {}", err);
    assert!(err.to_string().contains("code 13"))
}


#[test]
fn test_output_capacity_err() {
    let mut ct = ContractUtil::new();
    let commitment_lock_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let udt_contract = ct.deploy_contract("simple_udt");


    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    let payment_amount1 = 2u128;
    let payment_amount2 = 2u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();
    let total_sudt_amount = 424242424242424242u128;

    let mut cc = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: Some([42; 32]),
        data: total_sudt_amount,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_lock_contract.clone(), None, &cc, 1000);


    let new_witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&new_witness_script)[0..20].to_vec();

    let mut cc1 = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),

        type_arg: cc.type_arg,
        data: total_sudt_amount - payment_amount1,

        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    // err: OutputCapacityError
    // let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), None, &cc1, 999);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with remote_htlc_pubkey
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();
    let signature = remote_htlc_key1
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();


    cc1.witness = Some(CommitmentPendinghtlc2WithPriImageWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0b00000000,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000001,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x0,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
        preimage: preimage1,
    });
    // err: OutputCapacityError
    // let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.replace_output(tx, commitment_lock_contract.clone(), None, &cc1, 999, 0);


    println!("tx: {:?}", tx);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("OutputCapacityError");

    println!("err: {}", err);
    assert!(err.to_string().contains("code 13"))
}

#[test]
fn test_output_capacity_overflow() {
    let mut ct = ContractUtil::new();
    let commitment_lock_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");
    let udt_contract = ct.deploy_contract("simple_udt");


    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();
    let remote_htlc_key1 = generator.gen_keypair();
    let remote_htlc_key2 = generator.gen_keypair();
    let local_htlc_key1 = generator.gen_keypair();
    let local_htlc_key2 = generator.gen_keypair();
    let preimage1 = [42u8; 32];
    let preimage2 = [24u8; 32];
    // 1000 - 20000 -> subtract with overflow
    let payment_amount1 = 20000u128;
    let payment_amount2 = 2u128;
    // timeout after 2024-04-01 01:00:00
    let expiry1 = Since::from_timestamp(1711976400, true).unwrap();
    // timeout after 2024-04-02 01:00:00
    let expiry2 = Since::from_timestamp(1712062800, true).unwrap();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000000].to_vec(),
        payment_amount1.to_le_bytes().to_vec(),
        blake2b_256(preimage1)[0..20].to_vec(),
        blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec(),
        expiry1.as_u64().to_le_bytes().to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();
    let total_sudt_amount = 424242424242424242u128;

    let mut cc = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),
        type_arg: Some([42; 32]),
        data: total_sudt_amount,
        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, commitment_lock_contract.clone(), None, &cc, 1000);


    let new_witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
        [0b00000001].to_vec(),
        payment_amount2.to_le_bytes().to_vec(),
        blake2b_256(preimage2)[0..20].to_vec(),
        blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec(),
        blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec(),
        expiry2.as_u64().to_le_bytes().to_vec(),
    ].concat();
    let args = blake2b_256(&new_witness_script)[0..20].to_vec();

    let mut cc1 = CommitmentHTCL2WithPriImageAndUDTCell {
        lock_arg: <[u8; 20]>::try_from(args).unwrap(),

        type_arg: cc.type_arg,
        data: total_sudt_amount - payment_amount1,

        witness: None,
        struct_flag: MoleculeStructFlag::default(),
    };
    // err: OutputCapacityError
    // let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), None, &cc1, 999);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);
    let tx = ct.context.complete_tx(tx);

    // sign with remote_htlc_pubkey
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();
    let signature = remote_htlc_key1
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();


    cc1.witness = Some(CommitmentPendinghtlc2WithPriImageWitness {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        pending_htlc1: PendingHtlc {
            htlc_type: 0b00000000,
            payment_amount: payment_amount1,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage1)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key1.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry1.as_u64(),
        },
        pending_htlc2: PendingHtlc {
            htlc_type: 0b00000001,
            payment_amount: payment_amount2,
            payment_hash: <[u8; 20]>::try_from(blake2b_256(preimage2)[0..20].to_vec()).unwrap(),
            remote_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(remote_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            local_htlc_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_htlc_key2.1.serialize())[0..20].to_vec()).unwrap(),
            htlc_expiry: expiry2.as_u64(),
        },
        unlock_type: 0x0,
        signature: <[u8; 65]>::try_from(signature).unwrap(),
        preimage: preimage1,
    });
    // err: OutputCapacityError
    // let tx = ct.add_outpoint(tx, commitment_lock_contract.clone(), Some(udt_contract.clone()), &cc1, 1000);
    let tx = ct.replace_output(tx, commitment_lock_contract.clone(), None, &cc1, 999, 0);


    println!("tx: {:?}", tx);

    // run
    let err = ct.context
        .verify_tx(&tx, 100000000)
        .expect_err("subtract with overflow");

    println!("err: {}", err);
    assert!(err.to_string().contains("code -1"))
}

#[test]
fn test_auth_err() {
    let mut ct = ContractUtil::new();
    let commitment_contract = ct.deploy_contract("commitment-lock");
    let auth_contract = ct.deploy_contract("auth");

    // prepare script
    let mut generator = Generator::new();
    // 42 hours = 4.5 epochs
    let local_delay_epoch = Since::from_epoch(EpochNumberWithFraction::new(10, 1, 2), false);
    let local_delay_epoch_key = generator.gen_keypair();
    let revocation_key = generator.gen_keypair();

    let witness_script = [
        local_delay_epoch.as_u64().to_le_bytes().to_vec(),
        blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec(),
        blake2b_256(revocation_key.1.serialize())[0..20].to_vec(),
    ].concat();

    let args = blake2b_256(&witness_script)[0..20].to_vec();

    let mut commitment_cell = CommitmentCellNoHtlcAndPreImage::default();
    commitment_cell.lock_arg = <[u8; 20]>::try_from(args).unwrap();
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, commitment_contract.clone(), None, &commitment_cell, 1000);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &commitment_cell, 500);
    let tx = ct.context.complete_tx(tx);
    let tx = ct.add_contract_cell_dep(tx, &auth_contract);

    // sign with revocation key
    let message: [u8; 32] = tx.hash().as_slice().try_into().unwrap();

    let signature = revocation_key
        .0
        .sign_recoverable(&message.into())
        .unwrap()
        .serialize();

    commitment_cell.witness = Some(CommitmentWitnessNoHtlcAndPreImage {
        empty_witness_args: EMPTY_WITNESS_ARGS,
        local_delay_epoch: local_delay_epoch.as_u64(),
        local_delay_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(local_delay_epoch_key.1.serialize())[0..20].to_vec()).unwrap(),
        revocation_pubkey_hash: <[u8; 20]>::try_from(blake2b_256(revocation_key.1.serialize())[0..20].to_vec()).unwrap(),
        unlock_type: 255,
        signature: [1; 65],
    });
    let tx = ct.replace_output(tx, ct.alway_contract.clone(), None, &commitment_cell, 500, 0);

    println!("tx: {:?}", tx);

    // run
    match ct.context
        .verify_tx(&tx, 100000000) {
        Ok(_) => {
            assert!(false)
        }
        Err(err) => {
            println!("err:{:?}", err);
            assert!(err.to_string().contains("code 101"))
        }
    };
}