use crate::cell_message::cell::MoleculeStructFlag;
use crate::impl_cell_methods;

pub struct Demo {
    pub lock_arg: u8,
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<u8>,
    pub struct_flag: MoleculeStructFlag,
}

impl Demo {

    pub(crate) fn new() -> Self {
        return Demo {
            lock_arg: 0,
            type_arg: None,
            data: 1,
            witness: None,
            struct_flag: MoleculeStructFlag {
                lock_arg: true,
                type_arg: true,
                data: true,
                witness: true,
            },
        };
    }
}




impl_cell_methods!(Demo);
