
#[macro_export]
macro_rules! impl_cell_methods {

    ($struct_name:ident) => {
        use crate::cell_message::cell::Cell;
        use serde_molecule::to_vec;
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
        }
    };
}




