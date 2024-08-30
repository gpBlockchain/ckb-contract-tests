### CKB Contract Test Framework

#### Preparation

1. Place the contract in the `build/release` directory.

#### Deploying a Contract
```rust
use crate::{ContractUtil};

let mut ct = ContractUtil::new();
let xudt_contract = ct.deploy_contract("XUDT");
```

#### Invoking a Contract

To invoke a contract, you need to construct cell parameters.

A cell structure must include the following fields:
- `lock_arg`: Must be a serializable type.
- `type_arg`: Must be a serializable type. Set to `None` if you do not want to invoke a type contract.
- `data`: Must be a serializable type.
- `witness`: Must be a serializable type; can be `None`.
- `struct_flag`: Defines the encoding type for the 4 fields. Currently, only `molecule table` and `molecule struct` are supported.

```rust
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct XUDTData {
    pub amount: u128,
}

const EMPTY_WITNESS_ARGS: [u8; 16] = [16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0];

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct XUDTWitness {
    pub empty_witness_args: [u8; 16],
}

pub struct XUDTDataCell {
    pub lock_arg: u8,
    pub type_arg: Option<[u8; 32]>,
    pub data: XUDTData,
    pub witness: Option<XUDTWitness>,
    pub struct_flag: MoleculeStructFlag,
}

impl_cell_methods!(XUDTDataCell);
```

#### Invoking a Contract Example

A simple XUDT transfer: A(token:2000) -> B(token:2000)

```rust
let input_token_cell = XUDTDataCell::new([1; 32], XUDTData { amount: 2000 });
let output_token1_cell = XUDTDataCell::new([1; 32], XUDTData { amount: 2000 });
let mut ct = ContractUtil::new();
let type_contract = ct.deploy_contract("XUDT");
let mut tx = TransactionBuilder::default().build();
tx = ct.add_input(tx, ct.alway_contract.clone(), Some(type_contract.clone()), &input_token_cell, 100);
tx = ct.add_outpoint(tx, ct.alway_contract.clone(), Some(type_contract.clone()), &output_token1_cell, 100);
tx = ct.context.complete_tx(tx);
let ret1 = ct.context.should_be_passed(&tx, 1000000);
println!("ret:{:?}", ret1)
```

refer: src/tests/xudt.rs