
#[macro_export]
macro_rules! impl_cell_methods {

    ($struct_name:ident) => {
        use crate::cell_message::cell::Cell;
        use serde_molecule::to_vec;
        use serde_molecule::from_slice;
        impl Cell for $struct_name {
            fn get_lock_arg(&self) -> Vec<u8> {
                to_vec(&self.lock_arg, self.struct_flag.lock_arg).unwrap()
            }

            fn get_type_arg(&self) -> Option<Vec<u8>> {
                match &self.type_arg {
                    None => None,
                    Some(arg) => Some(to_vec(arg, self.struct_flag.type_arg).unwrap()),
                }
            }

            fn get_data(&self) -> Vec<u8> {
                to_vec(&self.data, self.struct_flag.data).unwrap()
            }

            fn get_witness(&self) -> Option<Vec<u8>> {
                match &self.witness {
                    None => None,
                    Some(arg) => Some(to_vec(arg, self.struct_flag.witness).unwrap()),
                }
            }

            fn from_arg(lock_arg: Vec<u8>, type_arg: Option<Vec<u8>>, data1: Vec<u8>, witness_args: Option<Vec<u8>>) -> Self {
        let mut data = Self::default();
        data.lock_arg = from_slice(&lock_arg, data.struct_flag.lock_arg).unwrap();
        data.type_arg = match type_arg {
            None => { None }
            Some(arg) => {
                Some(from_slice(&arg, data.struct_flag.type_arg).unwrap())
            }
        };
        data.data = from_slice(&data1, data.struct_flag.data).unwrap();
        data.witness = match witness_args {
            None => { None }
            Some(witness) => {
                Some(from_slice(&witness, data.struct_flag.witness).unwrap())
            }
        };
        data
    }

        }
    };
}

#[macro_export]
macro_rules! impl_cell_methods_without_import {

    ($struct_name:ident) => {
        impl Cell for $struct_name {
            fn get_lock_arg(&self) -> Vec<u8> {
                to_vec(&self.lock_arg, self.struct_flag.lock_arg).unwrap()
            }

            fn get_type_arg(&self) -> Option<Vec<u8>> {
                match &self.type_arg {
                    None => None,
                    Some(arg) => Some(to_vec(arg, self.struct_flag.type_arg).unwrap()),
                }
            }

            fn get_data(&self) -> Vec<u8> {
                to_vec(&self.data, self.struct_flag.data).unwrap()
            }

            fn get_witness(&self) -> Option<Vec<u8>> {
                match &self.witness {
                    None => None,
                    Some(arg) => Some(to_vec(arg, self.struct_flag.witness).unwrap()),
                }
            }

            fn from_arg(lock_arg: Vec<u8>, type_arg: Option<Vec<u8>>, data1: Vec<u8>, witness_args: Option<Vec<u8>>) -> Self {
        let mut data = Self::default();
        data.lock_arg = from_slice(&lock_arg, data.struct_flag.lock_arg).unwrap();
        data.type_arg = match type_arg {
            None => { None }
            Some(arg) => {
                Some(from_slice(&arg, data.struct_flag.type_arg).unwrap())
            }
        };
        data.data = from_slice(&data1, data.struct_flag.data).unwrap();
        data.witness = match witness_args {
            None => { None }
            Some(witness) => {
                Some(from_slice(&witness, data.struct_flag.witness).unwrap())
            }
        };
        data
    }

        }
    };
}



