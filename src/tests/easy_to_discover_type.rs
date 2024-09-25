use ckb_std::ckb_types::packed::CellOutput;
use ckb_testtool::ckb_hash::blake2b_256;
use ckb_testtool::ckb_jsonrpc_types::{Deserialize, Serialize};
use ckb_testtool::ckb_traits::CellDataProvider;
use ckb_testtool::ckb_types::core::TransactionBuilder;
use ckb_testtool::ckb_types::prelude::Entity;
use crate::cell_message::cell::MoleculeStructFlag;
use crate::{ContractUtil, impl_cell_methods, impl_cell_methods_without_import};
use serde_molecule::big_array_serde;
use crate::prelude::ContextExt;

#[derive(Default, Clone)]
pub struct EasyToDiscoverTypeCell {
    pub lock_arg: u8,
    pub type_arg: Option<[u8; 32]>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}

#[derive(Default)]
pub struct EasyToDiscoverTypeArgErr1Cell {
    pub lock_arg: u8,
    pub type_arg: Option<[u8; 31]>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LongArg {
    #[serde(with = "big_array_serde")]
    pub arg: [u8; 33],
}
impl Default for LongArg {
    fn default() -> Self {
        LongArg {
            arg: [0; 33],
        }
    }
}


#[derive(Default)]
pub struct EasyToDiscoverTypeArgErr2Cell {
    pub lock_arg: u8,
    pub type_arg: Option<LongArg>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}


impl_cell_methods!(EasyToDiscoverTypeCell);
impl_cell_methods_without_import!(EasyToDiscoverTypeArgErr1Cell);
impl_cell_methods_without_import!(EasyToDiscoverTypeArgErr2Cell);


/// 0->1
///
///     args 的长度小于32
///         结果:InsufficientArgsLength
#[test]
fn arg_length_too_low() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");
    let tx = TransactionBuilder::default().build();
    let code_hash = ct.context.get_cell_data_hash(&ct.alway_contract);
    println!("alway_contract:{:?}", code_hash);
    let code_hash1 = ct.context.get_cell_data_hash(&easy_to_discover_type_contract);
    println!("easy_to_discover_type_contract:{:?}", code_hash1);

    let mut easy_cell = EasyToDiscoverTypeArgErr1Cell {
        lock_arg: 2,
        type_arg: Some([1; 31]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &easy_cell, 1000);
    easy_cell.type_arg = Some(blake2b_256(&easy_cell.get_data())[0..31].try_into().unwrap());
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret1 = ct.context.should_be_failed(&tx, 1000000).expect_err("InsufficientArgsLength");
    assert!(ret1.to_string().contains("code 5"));
}

///
/// 0->1
///
///     - args 的长度等于32
///     - data的hash和args不匹配
///         结果:DataHashNotMatch
#[test]
fn test_0_to_1_data_hash_not_match() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");
    let tx = TransactionBuilder::default().build();
    let easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &easy_cell, 1000);
    // easy_cell.type_arg = Some(blake2b_256(&easy_cell.get_data()));
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret1 = ct.context.should_be_failed(&tx, 1000000).expect_err("DataHashNotMatch");
    assert!(ret1.to_string().contains("code 6"));
}

/// 0->1
///
///     args 的长度等于32
///     data的hash和args匹配
///     结果:0
#[test]
fn test_0_to_1_hash_args_matched() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");
    let tx = TransactionBuilder::default().build();
    let mut easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &easy_cell, 1000);
    easy_cell.type_arg = Some(blake2b_256(&easy_cell.get_data()));
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("matched ");
}


///
/// 0->1
///
///     args 的长度超过32
///     data的hash和args[0..32]不匹配
///     结果:DataHashNotMatch
#[test]
fn test_0_to_1_data_too_long_and_hash_not_match_err() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");
    let tx = TransactionBuilder::default().build();
    let easy_cell = EasyToDiscoverTypeArgErr2Cell {
        lock_arg: 2,
        type_arg: Some(LongArg {
            arg: [1u8; 33]
        }),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &easy_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret1 = ct.context.should_be_failed(&tx, 1000000).expect_err("DataHashNotMatch");
    assert!(ret1.to_string().contains("code 6"));
}


/// 0->1
///
///     args 的长度超过32
///     data的hash和args[0..32]匹配
///         结果:0
#[test]
fn test_0_t0_1_arg_too_long_but_matched() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");
    let tx = TransactionBuilder::default().build();
    let mut easy_cell = EasyToDiscoverTypeArgErr2Cell {
        lock_arg: 2,
        type_arg: None,
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &easy_cell, 1000);
    let mut arg = blake2b_256(&easy_cell.get_data());
    let mut arg = arg[0..32].to_vec();
    arg.push(1);
    easy_cell.type_arg = Some(LongArg {
        arg: arg.as_slice().try_into().unwrap(),
    });
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("matched ");
}

/// 1->1
///
///     input的args和output args 不一致
///     output args 和 output.data的hash 一致
///         结果: return 0
#[test]
fn test_1_to_1_output_arg_matched_data_hash() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");

    let tx = TransactionBuilder::default().build();
    let mut input_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };
    input_easy_cell.type_arg = Some(blake2b_256(input_easy_cell.get_data()));

    let mut out_put_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 2,
        witness: None,
        struct_flag: Default::default(),
    };
    out_put_easy_cell.type_arg = Some(blake2b_256(out_put_easy_cell.get_data()));


    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &input_easy_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &out_put_easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("matched ");
}

/// 1->1
///
///     input的args和output args 不一致
///     output args 和 data的hash 不一致
///     结果:DataHashNotMatch
#[test]
fn test_1_to_1_output_arg_not_matched_data_hash() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");

    let tx = TransactionBuilder::default().build();
    let mut input_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };
    input_easy_cell.type_arg = Some(blake2b_256(input_easy_cell.get_data()));

    let out_put_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 2,
        witness: None,
        struct_flag: Default::default(),
    };


    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &input_easy_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &out_put_easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("DataHashNotMatch");
    assert!(ret.to_string().contains("code 6"))
}

/// 1->1
///
///     input的args和output args 一致
///     data 的hash和之前的inputargs 一致
///         结果: return 0
#[test]
fn test_1_to_1_args_eq_and_data_hash_match_args() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");

    let tx = TransactionBuilder::default().build();
    let mut input_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };
    input_easy_cell.type_arg = Some(blake2b_256(input_easy_cell.get_data()));

    let out_put_easy_cell = input_easy_cell.clone();

    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &input_easy_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &out_put_easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("eq");
}

///
/// 1->2
///
///     2个output 的args 一致
///     data 的codeHash和arg 匹配
///         return 0
#[test]
fn test_1_to_2_datas_hash_matched_args() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");

    let tx = TransactionBuilder::default().build();
    let mut input_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };
    input_easy_cell.type_arg = Some(blake2b_256(input_easy_cell.get_data()));

    let out_put_easy_cell = input_easy_cell.clone();

    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &input_easy_cell, 2000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &out_put_easy_cell, 1000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &out_put_easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("eq");
}


/// 1->2
///
///     2个output 的args 一致
///     第二个data 和arg不匹配
///     结果:DataHashNotMatch
#[test]
fn test_1_to_2_datas_hash_not_matched_args() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");

    let tx = TransactionBuilder::default().build();
    let mut input_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };
    input_easy_cell.type_arg = Some(blake2b_256(input_easy_cell.get_data()));

    let mut out_put_easy_cell = input_easy_cell.clone();

    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &input_easy_cell, 2000);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &out_put_easy_cell, 1000);
    out_put_easy_cell.data = 3;
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &out_put_easy_cell, 1000);
    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("DataHashNotMatch");
    assert!(ret.to_string().contains("code 6"));
}

/// 2->0
///
///     output 可以为空
///     结果: 0
#[test]
fn test_2_to_0() {
    let mut ct = ContractUtil::new();
    let easy_to_discover_type_contract = ct.deploy_contract("easy-to-discover-type");

    let tx = TransactionBuilder::default().build();
    let mut input_easy_cell = EasyToDiscoverTypeCell {
        lock_arg: 2,
        type_arg: Some([1; 32]),
        data: 1,
        witness: None,
        struct_flag: Default::default(),
    };
    input_easy_cell.type_arg = Some(blake2b_256(input_easy_cell.get_data()));
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &input_easy_cell, 2000);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), Some(easy_to_discover_type_contract.clone()), &input_easy_cell, 2000);
    input_easy_cell.data = 2;
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &input_easy_cell, 4000);
    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("eq");
}