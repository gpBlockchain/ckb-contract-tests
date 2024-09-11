use ckb_testtool::ckb_hash::blake2b_256;

use ckb_testtool::ckb_types::core::TransactionBuilder;
use ckb_testtool::ckb_types::prelude::Entity;
use crate::cell_message::cell::MoleculeStructFlag;
use crate::{ContractUtil, impl_cell_methods};
use crate::cells::demo::Demo;
use crate::prelude::ContextExt;

#[derive(Default)]
pub struct LockProxyLockCell {
    pub lock_arg: [u8; 32],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}
impl_cell_methods!(LockProxyLockCell);


/// 1->0
///
///     没有另外lock
///     结果:InvalidUnlock
#[test]
fn test_1_to_0_not_eq_args() {
    let mut ct = ContractUtil::new();
    let lock_proxy_lock_contract = ct.deploy_contract("lock-proxy-lock");
    let tx = TransactionBuilder::default().build();

    let lock_proxy_cell = LockProxyLockCell {
        lock_arg: [1; 32],
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);
    let tx = ct.add_outpoint(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);

    let tx = ct.context.complete_tx(tx);
    let ret1 = ct.context.should_be_failed(&tx, 1000000).expect_err("InvalidUnlock");
    println!("ret:{:?}", ret1);
    assert!(ret1.to_string().contains("code 6"))
}


/// 1->0
///
///     有另外lock
///     arg = 另外合约的load_cell_lock_hash
///     结果:0
#[test]
fn test_1_to_0_other_lock_eq_args() {
    let mut ct = ContractUtil::new();
    let lock_proxy_lock_contract = ct.deploy_contract("lock-proxy-lock");
    let tx = TransactionBuilder::default().build();

    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 100);
    let lock_proxy_cell = LockProxyLockCell {
        lock_arg: blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.lock().as_slice()),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);
    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);

    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}

/// 1->0
///
///     有另外lock
///     只有一个
///     arg != 另外合约的load_cell_lock_hash
///     结果:0
#[test]
fn test_1_to_0_other_lock_not_eq_args() {
    let mut ct = ContractUtil::new();
    let lock_proxy_lock_contract = ct.deploy_contract("lock-proxy-lock");
    let tx = TransactionBuilder::default().build();

    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 100);
    let lock_proxy_cell = LockProxyLockCell {
        lock_arg: [1; 32],
        // lock_arg: blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.lock().as_slice()),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &lock_proxy_cell, 100);

    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("InvalidUnlock");
    assert!(ret.to_string().contains("code 6"))
}

/// 1->0
///
///     有另外lock
///     有多个
///     存在一个arg =另外合约的load_cell_lock_hash
///     结果:0
#[test]
fn test_1_to_0_other_lock_exist_1_lock_hash_eq_args() {
    let mut ct = ContractUtil::new();
    let lock_proxy_lock_contract = ct.deploy_contract("lock-proxy-lock");
    let tx = TransactionBuilder::default().build();


    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 100);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo {
        lock_arg: 2,
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    }, 100);

    let lock_proxy_cell = LockProxyLockCell {
        lock_arg: blake2b_256(ct.context.get_cell(&tx.inputs().get(1).unwrap().previous_output()).unwrap().0.lock().as_slice()),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &lock_proxy_cell, 100);

    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}


/// 1->0
///
///     有另外lock
///     有多个
///     arg !=另外合约的load_cell_lock_hash
///     结果:0
#[test]
fn test_1_to_0_other_lock_lock_hash_eq_args() {
    let mut ct = ContractUtil::new();
    let lock_proxy_lock_contract = ct.deploy_contract("lock-proxy-lock");
    let tx = TransactionBuilder::default().build();


    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 100);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo {
        lock_arg: 2,
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    }, 100);

    let lock_proxy_cell = LockProxyLockCell {
        lock_arg: [1; 32],
        // lock_arg: blake2b_256(ct.context.get_cell(&tx.inputs().get(0).unwrap().previous_output()).unwrap().0.lock().as_slice()),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &lock_proxy_cell, 100);

    let tx = ct.context.complete_tx(tx);
    let ret = ct.context.should_be_failed(&tx, 1000000).expect_err("InvalidUnlock");
    assert!(ret.to_string().contains("code 6"))
}


/// 2->0
///
///     有另外lock
///     有多个
///     存在一个arg =另外合约的load_cell_lock_hash
///     结果:0
#[test]
fn test_2_to_0_other_lock_exist_1_lock_hash_eq_args() {
    let mut ct = ContractUtil::new();
    let lock_proxy_lock_contract = ct.deploy_contract("lock-proxy-lock");
    let tx = TransactionBuilder::default().build();


    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo::default(), 100);
    let tx = ct.add_input(tx, ct.alway_contract.clone(), None, &Demo {
        lock_arg: 2,
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    }, 100);

    let lock_proxy_cell = LockProxyLockCell {
        lock_arg: blake2b_256(ct.context.get_cell(&tx.inputs().get(1).unwrap().previous_output()).unwrap().0.lock().as_slice()),
        type_arg: None,
        data: 0,
        witness: None,
        struct_flag: Default::default(),
    };

    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);
    let tx = ct.add_input(tx, lock_proxy_lock_contract.clone(), None, &lock_proxy_cell, 100);
    let tx = ct.add_outpoint(tx, ct.alway_contract.clone(), None, &lock_proxy_cell, 100);

    let tx = ct.context.complete_tx(tx);
    ct.context.should_be_passed(&tx, 1000000).expect("pass");
}
