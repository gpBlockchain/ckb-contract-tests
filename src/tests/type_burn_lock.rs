use ckb_std::ckb_types::prelude::Entity;
use ckb_testtool::ckb_hash::blake2b_256;
use ckb_testtool::ckb_types::core::TransactionBuilder;
use crate::cell_message::cell::MoleculeStructFlag;
use crate::{ContractUtil, impl_cell_methods, impl_cell_methods_without_import};
use crate::prelude::ContextExt;

#[derive(Default, Clone)]
pub struct TBLArgErrCell {
    pub lock_arg: [u8; 31],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}


#[derive(Default)]
pub struct TBLCell {
    pub lock_arg: [u8; 32],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}
impl_cell_methods!(TBLArgErrCell);
impl_cell_methods_without_import!(TBLCell);

/// 1->0
///
///     args 长度小于32
///     结果:Error::Encoding
#[test]
fn test_1_to_0_arg_too_low_err_encoding() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("type-burn-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = TBLArgErrCell::default();
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 4"))
}

///
/// 1->0
///
///     arg>=32
///     input 没有type
///         Error::TypeScriptNotBurnt
#[test]
fn test_1_to_0_arg_no_type() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("type-burn-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = TBLCell::default();
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 6"))
}


///
/// 1->0
///
///     arg>=32
///     input 有type
///     input type 的hash != arg
///         Error::TypeScriptNotBurnt
#[test]
fn test_1_to_0_arg_not_eq_type() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("type-burn-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = TBLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 6"))
}


///
/// 1->0
///
///     arg>=32
///     input 有type
///     input type 的hash == arg
///     output 不存在input type 的hash == arg
///         结果为0
#[test]
fn test_1_to_0_arg_eq_type_and_output_not_eq() {
    let mut ct = ContractUtil::new();
    let type_burn_lock_contract = ct.deploy_contract("type-burn-lock");
    let tx = TransactionBuilder::default().build();
    let mut input_cell = TBLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    input_cell.lock_arg = blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let tx = ct.add_input(tx, type_burn_lock_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);

    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}


///
/// 1->0
///
///     arg>=32
///     input 有type
///     input type 的hash == arg
///     output 存在input type 的hash == arg
///         结果为0
#[test]
fn test_1_to_0_arg_eq_type_and_output_eq() {
    let mut ct = ContractUtil::new();
    let type_burn_lock_contract = ct.deploy_contract("type-burn-lock");
    let tx = TransactionBuilder::default().build();
    let mut input_cell = TBLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    input_cell.lock_arg = blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let tx = ct.add_input(tx, type_burn_lock_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);

    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 6"))
}


///
/// 2->0
///
///     arg>=32
///     input 有type
///     input type 的hash == arg
///     output 不存在input type 的hash == arg
///         结果为0
#[test]
fn test_2_to_0_arg_eq_type_and_output_not_eq() {
    let mut ct = ContractUtil::new();
    let type_burn_lock_contract = ct.deploy_contract("type-burn-lock");
    let tx = TransactionBuilder::default().build();
    let mut input_cell = TBLCell {
        lock_arg: [1; 32],
        type_arg: Some(1),
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    input_cell.lock_arg = blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.type_().to_opt().expect("type script").as_slice());
    let tx = ct.add_input(tx, type_burn_lock_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);
    let tx = ct.add_input(tx, type_burn_lock_contract.clone(), Some(ct.alway_contract.clone()), &input_cell, 1000);

    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}




