use ckb_testtool::ckb_hash::blake2b_256;
use ckb_testtool::ckb_types::core::TransactionBuilder;
use ckb_testtool::ckb_types::prelude::Entity;
use crate::cell_message::cell::MoleculeStructFlag;
use crate::{ContractUtil, impl_cell_methods, impl_cell_methods_without_import};
use crate::prelude::ContextExt;
use crate::tests::input_type_proxy_lock::ITPLArgErrCell;

#[derive(Default, Clone)]
pub struct OTPLCell {
    pub lock_arg: [u8; 32],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}

#[derive(Default)]
pub struct OTPLArgErrCell {
    pub lock_arg: [u8; 31],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}
impl_cell_methods!(OTPLCell);
impl_cell_methods_without_import!(OTPLArgErrCell);

/// 1->0
///
///     args 长度小于32
///     结果:Error::Encoding
#[test]
fn test_1_to_0_arg_too_low_err_encoding() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("output-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = OTPLArgErrCell::default();
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 4"))
}


/// 1->0
///
///     args >= 32
///     output 的type 为空
///         InvalidUnlock
#[test]
fn test_1_to_0_output_type_is_empty() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("output-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = OTPLCell::default();
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 6"))
}


/// 1->0
///
///     args >= 32
///     有多个outputs
///
///     部分output为空，部分output有值
///     第1个output的load_cell_type_hash == arg[0..32]
///     结果0
#[test]
fn test_1_to_0_output_type_is_eq() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("output-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let output_cell = OTPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &output_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &output_cell, 1000);
    let mut input_cell = OTPLCell::default();
    input_cell.lock_arg = blake2b_256(tx.output(0).expect("").type_().to_opt().expect("type exit").as_slice());
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}

/// 1->0
///
///     args >= 32
///     有多个outputs
///
///     部分output为空，部分output有值
///     第1个不匹配，第二个为空，第3个output的load_cell_type_hash == arg[0..32]
///     结果0
#[test]
fn test_1_to_0_output_type_is_eq_at_index_3() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("output-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let mut output_cell = OTPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &output_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &output_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &output_cell, 1000);
    output_cell.type_arg = Some(2);
    let mut input_cell = OTPLCell::default();
    input_cell.lock_arg = blake2b_256(tx.output(2).expect("").type_().to_opt().expect("type exit").as_slice());
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 3000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}


/// 1->0
///
///     args >= 32
///     有多个outputs
///
///     部分output为空，部分output有值
///     所有的outputs的load_cell_type_hash != arg[0..32]
///         结果InvalidUnlock
#[test]
fn test_1_to_0_output_type_not_eq() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("output-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let mut output_cell = OTPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &output_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &output_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &output_cell, 1000);
    output_cell.type_arg = Some(2);
    let input_cell = OTPLCell::default();
    // input_cell.lock_arg = blake2b_256(tx.output(2).expect("").type_().to_opt().expect("type exit").as_slice());
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 3000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("pass");
    assert!(ret.to_string().contains("code 6"))
}


/// 2->0
///
///     args >= 32
///     有多个outputs
///
///     部分output为空，部分output有值
///     第1个不匹配，第二个为空，第3个output的load_cell_type_hash == arg[0..32]
///         结果0
#[test]
fn test_2_to_0_output_type_eq() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("output-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let mut output_cell = OTPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &output_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &output_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &output_cell, 1000);
    output_cell.type_arg = Some(2);
    let mut input_cell = OTPLCell::default();
    input_cell.lock_arg = blake2b_256(tx.output(2).expect("").type_().to_opt().expect("type exit").as_slice());
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 3000);
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 3000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}

