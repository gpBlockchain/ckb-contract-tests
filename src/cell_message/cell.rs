pub trait Cell {
    fn get_lock_arg(&self) -> Vec<u8>;

    fn get_type_arg(&self) -> Option<Vec<u8>>;
    fn get_data(&self) -> Vec<u8>;
    fn get_witness(&self) -> Option<Vec<u8>>;
}


pub struct MoleculeStructFlag {
    pub lock_arg: bool,
    pub type_arg: bool,
    pub data: bool,
    pub witness: bool,
}
