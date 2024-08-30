use ckb_testtool::ckb_types::core::TransactionBuilder;
use ckb_testtool::ckb_types::prelude::{Builder, Entity};

use crate::cells::xudt_data::{XUDTData, XUDTDataCell};
use crate::{ContractUtil};
use crate::prelude::ContextExt;


#[test]
fn test_transfer_successful() {
    let input_token_cell = XUDTDataCell::new([1; 32], XUDTData { amount: 2000 });
    let input_token2_cell = XUDTDataCell::new([1; 32], XUDTData { amount: 2000 });

    let output_token1_cell = XUDTDataCell::new([1; 32], XUDTData { amount: 2000 });
    let output_token2_cell = XUDTDataCell::new([1; 32], XUDTData { amount: 2000 });

    let mut ct = ContractUtil::new();
    let type_contract = ct.deploy_contract("XUDT");
    let mut tx = TransactionBuilder::default().build();
    tx = ct.add_input(tx, ct.alway_contract.clone(), Some(type_contract.clone()), &input_token_cell, 100);
    tx = ct.add_input(tx, ct.alway_contract.clone(), Some(type_contract.clone()), &input_token2_cell, 100);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(type_contract.clone()), &output_token1_cell, 100);
    tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(type_contract.clone()), &output_token2_cell, 100);
    tx = ct.context.complete_tx(tx);
    let ret1 = ct.context.should_be_passed(&tx, 1000000);
    println!("ret:{:?}", ret1)
}