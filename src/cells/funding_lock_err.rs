use serde::{Deserialize, Serialize};
use crate::cell_message::cell::MoleculeStructFlag;
use serde_molecule::big_array_serde;
use crate::impl_cell_methods;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct FundingErrWitness {
    pub empty_witness_args: [u8; 16],
    pub version: u64,
    #[serde(with = "big_array_serde")]
    pub funding_out_point: [u8; 36],
    pub pubkey: [u8; 32],
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 64],

    pub err: [u8; 32],

}
pub struct FundingErrCell {
    pub lock_arg: [u8;20],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<FundingErrWitness>,
    pub struct_flag: MoleculeStructFlag,
}


impl FundingErrCell {
    pub(crate) fn default() -> Self {
        return FundingErrCell {
            lock_arg: [0;20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        };
    }
}

impl_cell_methods!(FundingErrCell);