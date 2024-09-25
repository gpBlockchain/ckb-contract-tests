use ckb_testtool::ckb_hash::blake2b_256;
use ckb_testtool::ckb_types::core::TransactionBuilder;
use ckb_testtool::ckb_types::prelude::Entity;
use crate::cell_message::cell::MoleculeStructFlag;
use crate::{ContractUtil, impl_cell_methods, impl_cell_methods_without_import};
use crate::prelude::ContextExt;

#[derive(Default)]
pub struct ITPLArgErrCell {
    pub lock_arg: [u8; 31],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}

#[derive(Default)]
pub struct ITPLCell {
    pub lock_arg: [u8; 32],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}
impl_cell_methods!(ITPLArgErrCell);
impl_cell_methods_without_import!(ITPLCell);

/// 1->0
///
///     args 长度小于32
///     结果:Error::Encoding
#[test]
fn test_1_to_0_arg_too_low_err_encoding() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = ITPLArgErrCell::default();
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 4"))
}


///
/// 1->0
///
///     type 为空
///     结果:InvalidUnlock
#[test]
fn test_1_to_0_type_empty_err() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = ITPLCell::default();
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 6"))
    // blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.lock().as_slice()),
}


/// 1->0
///
///     type 存在
///     args 和type_hash 一致
///     结果：0
#[test]
fn test_1_to_0_type_hash_eq_args() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let input_with_type_cell = ITPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);
    let hash = blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let input_cell = ITPLCell {
        lock_arg: hash,
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("Encoding");
}

/// 1->0
///
///     input里有type 存在
///     args 和type_hash 不一致
///     结果：InvalidUnlock
#[test]
fn test_1_to_0_type_hash_not_eq_args() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let input_with_type_cell = ITPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);
    // let hash = blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let input_cell = ITPLCell {
        lock_arg: [1; 32],
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("InvalidUnlock");
    assert!(ret.to_string().contains("code 6"))
}

/// 2->0
///
///     args 长度大于等于32
///     2个args 一样
///     type 存在
///     一个type
///     args 和type hash 匹配
///         retrun 0
#[test]
fn test_2_to_0_type_hash_eq_args() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let input_with_type_cell = ITPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);
    let hash = blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let input_cell = ITPLCell {
        lock_arg: hash,
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 3000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("Encoding");
}

/// 2->0
///
///     args 长度大于等于32
///     2个args 一样
///     type 存在
///     多个type
///     匹配的type不在同一个cell
///         return 0
#[test]
fn test_2_to_0_type_hash_eq_args_in_diff_cell(){
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let mut input_with_type_cell = ITPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);
    input_with_type_cell.type_arg = Some(2);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);

    let hash = blake2b_256(ct.context.get_cell(&tx.inputs().get(1).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let input_cell = ITPLCell {
        lock_arg: hash,
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 3000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("Encoding");

}


/// 2->0
///
///     args 长度大于等于32
///     2个args 一样
///     type 存在
///     多个type
///     匹配的type在同一个cell
///         return 0
#[test]
fn test_2_to_0_type_hash_eq_args_in_same_cell(){
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let mut input_with_type_cell = ITPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);
    input_with_type_cell.type_arg = Some(2);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);

    let hash = blake2b_256(ct.context.get_cell(&tx.inputs().get(1).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let input_cell = ITPLCell {
        lock_arg: hash,
        type_arg: Some(2),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 3000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("Encoding");
}


/// 2->0
///
///     args 长度大于等于32
///     2个args 一样
///     type 存在
///     多个type
///     都不匹配
///         return 0
#[test]
fn test_2_to_0_type_hash_not_eq_args_in_same_cell(){
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("input-type-proxy-lock");
    let tx = TransactionBuilder::default().build();
    let mut input_with_type_cell = ITPLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);
    input_with_type_cell.type_arg = Some(2);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_with_type_cell, 1000);

    // let hash = blake2b_256(ct.context.get_cell(&tx.inputs().get(1).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let input_cell = ITPLCell {
        lock_arg: [1;32],
        type_arg: Some(2),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 3000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("InvalidUnlock");
    assert!(ret.to_string().contains("code 6"))
}