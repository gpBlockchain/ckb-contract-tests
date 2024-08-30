use crate::cell_message::cell::MoleculeStructFlag;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::impl_cell_methods;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct XUDTDataCell {
    pub lock_arg: u8,
    pub type_arg: Option<[u8; 32]>,
    pub data: XUDTData,
    pub witness: Option<XUDTWitness>,
    pub struct_flag: MoleculeStructFlag,
}


//
// pub struct CellStruct<A, B, C, D>
// where
//     A: Serialize + Deserialize<'static>,
//     B: Serialize + Deserialize<'static>,
//     C: Serialize + Deserialize<'static>,
//     D: Serialize + Deserialize<'static>,
// {
//     pub lock_arg: A,
//     pub type_arg: Option<B>,
//     pub data: C,
//     pub witness: Option<D>,
//     pub struct_flag: MoleculeStructFlag,
// }

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct XUDTData {
    pub amount: u128,
}
const EMPTY_WITNESS_ARGS: [u8; 16] = [16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0];

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct XUDTWitness {
    pub empty_witness_args: [u8; 16],
}

impl XUDTDataCell {
    pub(crate) fn default() -> Self {
        return XUDTDataCell {
            lock_arg: 0,
            type_arg: None,
            data: XUDTData { amount: 0 },
            witness: None,
            struct_flag: MoleculeStructFlag {
                lock_arg: true,
                type_arg: true,
                data: true,
                witness: true,
            },
        };
    }

    pub(crate) fn new(type_arg: [u8; 32], data: XUDTData) -> Self {
        return XUDTDataCell {
            lock_arg: 0,
            type_arg: Some(type_arg),
            data: data,
            witness: Some(XUDTWitness {
                empty_witness_args: EMPTY_WITNESS_ARGS
            }),
            struct_flag: MoleculeStructFlag {
                lock_arg: true,
                type_arg: true,
                data: true,
                witness: true,
            },
        };
    }
}


impl_cell_methods!(XUDTDataCell);
