use serde::{Deserialize, Serialize};
use crate::cell_message::cell::MoleculeStructFlag;
use serde_molecule::{dynvec_serde, big_array_serde, struct_serde};
use crate::{impl_cell_methods, impl_cell_methods_without_import};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PendingHtlc {
    pub htlc_type: u8,
    pub payment_amount: u128, // u128 in little endian
    pub payment_hash: [u8; 20],
    pub remote_htlc_pubkey_hash: [u8; 20],
    pub local_htlc_pubkey_hash: [u8; 20],
    pub htlc_expiry: u64, // 8 bytes, u64 in little endian, must be an absolute timestamp
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentWitness {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    #[serde(with = "dynvec_serde")]
    pub pending_htlc: Vec<PendingHtlc>,
    pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
    #[serde(with = "struct_serde")]
    pub preimage: Option<[u8; 32]>,
}


#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentWitnessNoHtlcAndPreImage {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    // #[serde(with = "dynvec_serde")]
    // pub pending_htlc: Vec<PendingHtlc>,
    pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
    // #[serde(with = "struct_serde")]
    // pub preimage:Option<[u8;32]>
}


pub struct CommitmentCellNoHtlcAndPreImage {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<CommitmentWitnessNoHtlcAndPreImage>,
    pub struct_flag: MoleculeStructFlag,
}

impl CommitmentCellNoHtlcAndPreImage {
    pub(crate) fn default() -> Self {
        CommitmentCellNoHtlcAndPreImage {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}


pub struct CommitmentCell {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<CommitmentWitness>,
    pub struct_flag: MoleculeStructFlag,
}

impl CommitmentCell {
    pub(crate) fn default() -> Self {
        CommitmentCell {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentPendinghtlc1Witness {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    #[serde(with = "struct_serde")]
    pub pending_htlc1: PendingHtlc,
    pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
    // #[serde(with = "struct_serde")]
    // pub preimage: Option<[u8; 32]>,
}

pub struct CommitmentHTCL1Cell {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<[u8; 32]>,
    pub data: u128,
    pub witness: Option<CommitmentPendinghtlc1Witness>,
    pub struct_flag: MoleculeStructFlag,
}

impl CommitmentHTCL1Cell {
    pub(crate) fn default() -> Self {
        CommitmentHTCL1Cell {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentPendinghtlc2Witness {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    #[serde(with = "struct_serde")]
    pub pending_htlc1: PendingHtlc,

    #[serde(with = "struct_serde")]
    pub pending_htlc2: PendingHtlc,

    pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
    // #[serde(with = "struct_serde")]
    // pub preimage: Option<[u8; 32]>,
}

pub struct CommitmentHTCL2Cell {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<CommitmentPendinghtlc2Witness>,
    pub struct_flag: MoleculeStructFlag,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentPendinghtlc2WithPriImageWitness {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    #[serde(with = "struct_serde")]
    pub pending_htlc1: PendingHtlc,

    #[serde(with = "struct_serde")]
    pub pending_htlc2: PendingHtlc,

    pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
    // #[serde(with = "struct_serde")]
    pub preimage: [u8; 32],
}


pub struct CommitmentHTCL2WithPriImageAndUDTCell {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<[u8; 32]>,
    pub data: u128,
    pub witness: Option<CommitmentPendinghtlc2WithPriImageWitness>,
    pub struct_flag: MoleculeStructFlag,
}


impl CommitmentHTCL2WithPriImageAndUDTCell {
    pub(crate) fn default() -> Self {
        CommitmentHTCL2WithPriImageAndUDTCell {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}


impl CommitmentHTCL2Cell {
    pub(crate) fn default() -> Self {
        CommitmentHTCL2Cell {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}


pub struct CommitmentArgErrCell {
    pub lock_arg: [u8; 22],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<CommitmentWitness>,
    pub struct_flag: MoleculeStructFlag,
}

impl CommitmentArgErrCell {
    pub(crate) fn default() -> Self {
        CommitmentArgErrCell {
            lock_arg: [0; 22],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentMinErrLenWitness {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    // pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
}

pub struct CommitmentMinWitnessLenErrCell {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<CommitmentMinErrLenWitness>,
    pub struct_flag: MoleculeStructFlag,
}

impl CommitmentMinWitnessLenErrCell {
    pub(crate) fn default() -> Self {
        CommitmentMinWitnessLenErrCell {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}


#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentMaxErrLenWitness {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
    pub err: [u8; 5],
}

pub struct CommitmentMaxWitnessLenErrCell {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<u8>,
    pub data: u8,
    pub witness: Option<CommitmentMaxErrLenWitness>,
    pub struct_flag: MoleculeStructFlag,
}

impl CommitmentMaxWitnessLenErrCell {
    pub(crate) fn default() -> Self {
        CommitmentMaxWitnessLenErrCell {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct CommitmentPendinghtlc1WithPreimageWitness {
    pub empty_witness_args: [u8; 16],
    pub local_delay_epoch: u64,
    pub local_delay_pubkey_hash: [u8; 20],
    pub revocation_pubkey_hash: [u8; 20],

    #[serde(with = "struct_serde")]
    pub pending_htlc1: PendingHtlc,
    pub unlock_type: u8,
    #[serde(with = "big_array_serde")]
    pub signature: [u8; 65],
    // #[serde(with = "struct_serde")]
    pub preimage: [u8; 32],
}

pub struct CommitmentHTCL1WithPreimageAndUDTCell {
    pub lock_arg: [u8; 20],
    pub type_arg: Option<[u8; 32]>,
    pub data: u128,
    pub witness: Option<CommitmentPendinghtlc1WithPreimageWitness>,
    pub struct_flag: MoleculeStructFlag,
}

impl CommitmentHTCL1WithPreimageAndUDTCell {
    pub(crate) fn default() -> Self {
        CommitmentHTCL1WithPreimageAndUDTCell {
            lock_arg: [0; 20],
            type_arg: None,
            data: 0,
            witness: None,
            struct_flag: MoleculeStructFlag::default(),
        }
    }
}


impl_cell_methods!(CommitmentCellNoHtlcAndPreImage);
impl_cell_methods_without_import!(CommitmentCell);
impl_cell_methods_without_import!(CommitmentArgErrCell);
impl_cell_methods_without_import!(CommitmentMinWitnessLenErrCell);
impl_cell_methods_without_import!(CommitmentMaxWitnessLenErrCell);
impl_cell_methods_without_import!(CommitmentHTCL1Cell);
impl_cell_methods_without_import!(CommitmentHTCL2Cell);
impl_cell_methods_without_import!(CommitmentHTCL2WithPriImageAndUDTCell);
impl_cell_methods_without_import!(CommitmentHTCL1WithPreimageAndUDTCell);