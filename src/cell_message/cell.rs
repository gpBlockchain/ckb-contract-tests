use ckb_testtool::ckb_jsonrpc_types::{Deserialize, Serialize};

pub trait Cell {
    fn get_lock_arg(&self) -> Vec<u8>;

    fn get_type_arg(&self) -> Option<Vec<u8>>;
    fn get_data(&self) -> Vec<u8>;
    fn get_witness(&self) -> Option<Vec<u8>>;

    fn from_arg(lock_arg: Vec<u8>, type_arg: Option<Vec<u8>>, data1: Vec<u8>, witness_args: Option<Vec<u8>>) -> Self
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MoleculeStructFlag {
    pub lock_arg: bool,
    pub type_arg: bool,
    pub data: bool,
    pub witness: bool,
}

impl Default for MoleculeStructFlag {
    fn default() -> Self {
        MoleculeStructFlag {
            lock_arg: true,
            type_arg: true,
            data: true,
            witness: true,
        }
    }
}
//
// impl MoleculeStructFlag {
//     pub(crate) fn default() -> Self {
//         MoleculeStructFlag {
//             lock_arg: true,
//             type_arg: true,
//             data: true,
//             witness: true,
//         }
//     }
// }
