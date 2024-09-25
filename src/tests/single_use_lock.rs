use std::default::Default;
use ckb_testtool::ckb_jsonrpc_types::{Deserialize, Serialize};
use ckb_testtool::ckb_types::core::TransactionBuilder;
use ckb_testtool::ckb_types::prelude::{Entity, Unpack};
use crate::cell_message::cell::MoleculeStructFlag;
use crate::{ContractUtil, impl_cell_methods, impl_cell_methods_without_import};
use crate::cells::demo::Demo;
use crate::prelude::ContextExt;

#[derive(Default, Clone)]
pub struct SULArgErrCell {
    pub lock_arg: [u8; 32],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Default, Debug)]
pub struct OutputJson {
    pub tx_hash: [u8; 32],
    pub index: u32,
}
#[derive(Default)]
pub struct SULCell {
    pub lock_arg: OutputJson,
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}
impl_cell_methods!(SULArgErrCell);
impl_cell_methods_without_import!(SULCell);

/// 1->0
///
///     args 长度小于32
///     结果:Error::Encoding
#[test]
fn test_1_to_0_arg_too_low_err_encoding() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("single-use-lock");
    let tx = TransactionBuilder::default().build();
    let input_cell = SULArgErrCell::default();
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("Encoding");
    assert!(ret.to_string().contains("code 4"))
}


/// 1->0
///
///     arg >= 36
///     arg 为OutPoint
///     其他cell的outpoint == arg
///         结果:0
#[test]
fn test_1_to_0_exist_arg_eq_output() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("single-use-lock");
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 1000);
    let output = tx.inputs().get(0).expect("input exist ").previous_output();

    let input_cell = SULCell {
        lock_arg: OutputJson {
            tx_hash: output.tx_hash().as_slice().try_into().unwrap(),
            index: output.index().unpack(),
        },
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}

/// 1->0
///
///     arg >= 36
///     arg 为OutPoint
///     cell的outpoint != arg
///         结果: OutpointNotFound
#[test]
fn test_1_to_0_arg_not_eq_output() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("single-use-lock");
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 1000);
    // let output = tx.inputs().get(0).expect("input exist ").previous_output();

    let input_cell = SULCell {
        // lock_arg: OutputJson {
        //     tx_hash: output.tx_hash().as_slice().try_into().unwrap(),
        //     index: output.index().unpack(),
        // },
        lock_arg: Default::default(),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("OutpointNotFound");
    assert!(ret.to_string().contains("code 7"))
}


/// 2->0
///
///     arg >= 36
///     arg 为OutPoint
///     其他cell的outpoint == arg
///         结果:0
#[test]
fn test_2_to_0_exist_arg_eq_output() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("single-use-lock");
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 1000);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 1000);
    let output = tx.inputs().get(0).expect("input exist ").previous_output();

    let input_cell = SULCell {
        lock_arg: OutputJson {
            tx_hash: output.tx_hash().as_slice().try_into().unwrap(),
            index: output.index().unpack(),
        },
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}


/// 2->0
///
///     arg >= 36
///     arg 为OutPoint
///     其他cell outpoint 都!= arg
///         结果: OutpointNotFound
#[test]
fn test_2_to_0_arg_not_eq_output() {
    let mut ct = ContractUtil::new();
    let input_type_proxy_contract = ct.deploy_contract("single-use-lock");
    let tx = TransactionBuilder::default().build();
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 1000);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 1000);
    let output = tx.inputs().get(0).expect("input exist ").previous_output();

    let input_cell = SULCell {
        // lock_arg: OutputJson {
        //     tx_hash: output.tx_hash().as_slice().try_into().unwrap(),
        //     index: output.index().unpack(),
        // },
        lock_arg:Default::default(),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_input(tx, input_type_proxy_contract.clone(), None, &input_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("OutpointNotFound");
    assert!(ret.to_string().contains("code 7"))
}
