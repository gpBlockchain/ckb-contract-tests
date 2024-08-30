use ckb_testtool::{
    ckb_error::Error,
    ckb_types::{
        bytes::Bytes,
        core::{Cycle, TransactionView},
    },
    context::Context,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use ckb_testtool::ckb_types::packed::{CellInput, CellOutputBuilder, OutPoint, ScriptOptBuilder};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use crate::cell_message::cell::Cell;
use crate::prelude::ContextExt;

#[cfg(test)]
pub(crate) mod utilities;
mod tests;
mod cells;
mod cell_message;

pub mod prelude {
    use ckb_testtool::{
        ckb_error::Error,
        ckb_types::core::{Cycle, TransactionView},
    };

    pub const MAX_CYCLES: u64 = 10_000_000;
    pub const SPV_CELL_CAP: u64 = 500;
    pub const SPV_HEADERS_GROUP_SIZE: usize = 20; // Speed up to save time.

    // This helper method runs Context::verify_tx, but in case error happens,
    // it also dumps current transaction to failed_txs folder.
    pub trait ContextExt {
        fn should_be_passed(&self, tx: &TransactionView, max_cycles: u64) -> Result<Cycle, Error>;
        fn should_be_failed(&self, tx: &TransactionView, max_cycles: u64) -> Result<Cycle, Error>;
    }
}

// The exact same Loader code from capsule's template, except that
// now we use MODE as the environment variable
const TEST_ENV_VAR: &str = "MODE";

pub enum TestEnv {
    Debug,
    Release,
}

impl FromStr for TestEnv {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(TestEnv::Debug),
            "release" => Ok(TestEnv::Release),
            _ => Err("no match"),
        }
    }
}

pub struct Loader(PathBuf);

impl Default for Loader {
    fn default() -> Self {
        let test_env = match env::var(TEST_ENV_VAR) {
            Ok(val) => val.parse().expect("test env"),
            Err(_) => TestEnv::Release,
        };
        Self::with_test_env(test_env)
    }
}

impl Loader {
    fn with_test_env(env: TestEnv) -> Self {
        let load_prefix = match env {
            TestEnv::Debug => "debug",
            TestEnv::Release => "release",
        };
        let mut base_path = match env::var("TOP") {
            Ok(val) => {
                let mut base_path: PathBuf = val.into();
                base_path.push("build");
                base_path
            }
            Err(_) => {
                let mut base_path = PathBuf::new();
                // cargo may use a different cwd when running tests, for example:
                // when running debug in vscode, it will use workspace root as cwd by default,
                // when running test by `cargo test`, it will use tests directory as cwd,
                // so we need a fallback path
                base_path.push("build");
                if !base_path.exists() {
                    base_path.pop();
                    base_path.push("..");
                    base_path.push("build");
                }
                base_path
            }
        };

        base_path.push(load_prefix);
        Loader(base_path)
    }

    pub fn load_binary(&self, name: &str) -> Bytes {
        let mut path = self.0.clone();
        path.push(name);
        let result = fs::read(&path);
        if result.is_err() {
            panic!("Binary {:?} is missing!", path);
        }
        result.unwrap().into()
    }
}

impl prelude::ContextExt for Context {
    fn should_be_passed(&self, tx: &TransactionView, max_cycles: u64) -> Result<Cycle, Error> {
        let result = self.verify_tx(tx, max_cycles);
        if let Err(err) = result {
            let mut path = env::current_dir().expect("current dir");
            path.push("failed_txs");
            std::fs::create_dir_all(&path).expect("create failed_txs dir");
            let mock_tx = self.dump_tx(tx).expect("dump failed tx");
            let json = serde_json::to_string_pretty(&mock_tx).expect("json");
            path.push(format!("0x{:x}.json", tx.hash()));
            println!("Failed tx written to {:?}", path);
            std::fs::write(path, json).expect("write");
            panic!("should be passed, but failed since {err}");
        }
        result
    }

    fn should_be_failed(&self, tx: &TransactionView, max_cycles: u64) -> Result<Cycle, Error> {
        let result = self.verify_tx(tx, max_cycles);
        if result.is_ok() {
            let mut path = env::current_dir().expect("current dir");
            path.push("failed_txs");
            std::fs::create_dir_all(&path).expect("create failed_txs dir");
            let mock_tx = self.dump_tx(tx).expect("dump failed tx");
            let json = serde_json::to_string_pretty(&mock_tx).expect("json");
            path.push(format!("0x{:x}.json", tx.hash()));
            println!("Failed tx written to {:?}", path);
            std::fs::write(path, json).expect("write");
            panic!("should be failed");
        }
        result
    }
}


pub struct ContractUtil {
    pub loader: Loader,
    pub context: Context,
    pub alway_contract: OutPoint,

}

impl ContractUtil {
    fn new() -> Self {
        let loader = Loader::default();
        let mut context = Context::default();

        let stack_reorder_bin = loader.load_binary("always_success");
        let out_point = context.deploy_cell(stack_reorder_bin);

        return Self {
            loader: loader,
            context: context,
            alway_contract: out_point,
        };
    }

    pub fn deploy_contract(&mut self, name: &str) -> OutPoint {
        // self.loader.load_binary("atomic-first-contract")
        let stack_reorder_bin = self.loader.load_binary(name);
        self.context.deploy_cell(stack_reorder_bin)
    }

    pub fn add_input(&mut self, tx_builder: TransactionView, lock_contract: OutPoint, type_contract: Option<OutPoint>, cell_tx: &dyn Cell, redundant_cap: usize) -> TransactionView {

        // lock script
        let lock_script = self.context.build_script(&lock_contract, cell_tx.get_lock_arg().into()).unwrap();

        let cell_output = {
            let mut cell_output = CellOutputBuilder::default()
                .lock(lock_script);

            // type
            match type_contract {
                None => {}
                Some(contract) => {
                    let script = self.context.build_script(&contract, cell_tx.get_type_arg().unwrap().into()).unwrap();
                    cell_output = cell_output.type_(ScriptOptBuilder::default()
                        .set(Some(script)).build());
                }
            }

            // capacity
            cell_output = cell_output.capacity((redundant_cap as u64).pack());
            cell_output
        };

        // data
        let out_point1 = self.context.create_cell(cell_output.build(), cell_tx.get_data().into());
        let input = CellInput::new_builder().previous_output(out_point1).build();
        tx_builder.as_advanced_builder()
            .input(input).build()
    }

    pub fn create_tx_cells(&mut self, tx_build: TransactionView) {
        self.context.should_be_passed(&tx_build, 10_000_000).unwrap();
        tx_build.outputs_with_data_iter()
            .for_each(|(cell, data)|
            {
                self.context.create_cell(cell, data);
            }
            )
    }


    pub fn create_cell_input_by_cell(&mut self, lock_contract: OutPoint, type_contract: Option<OutPoint>, cell_tx: &dyn Cell, redundant_cap: usize) -> CellInput {

        // lock script
        let lock_script = self.context.build_script(&lock_contract, cell_tx.get_lock_arg().into()).unwrap();

        let cell_output = {
            let mut cell_output = CellOutputBuilder::default()
                .lock(lock_script);

            // type
            match type_contract {
                None => {}
                Some(contract) => {
                    let script = self.context.build_script(&contract, cell_tx.get_type_arg().unwrap().into()).unwrap();
                    cell_output = cell_output.type_(ScriptOptBuilder::default()
                        .set(Some(script)).build());
                }
            }

            // capacity
            cell_output = cell_output.capacity((redundant_cap as u64).pack());
            cell_output
        };

        // data
        let out_point1 = self.context.create_cell(cell_output.build(), cell_tx.get_data().into());
        CellInput::new_builder().previous_output(out_point1).build()
    }

    pub fn add_outpoint(&mut self, tx_builder: TransactionView, lock_contract: OutPoint, type_contract: Option<OutPoint>, cell_tx: &dyn Cell, redundant_cap: usize) -> TransactionView {
        // lock script
        let lock_script = self.context.build_script(&lock_contract, cell_tx.get_lock_arg().into()).unwrap();


        let mut cell_output = {
            let mut cell_output = CellOutputBuilder::default()
                .lock(lock_script);
            match type_contract {
                None => {}
                Some(contract) => {
                    let script = self.context.build_script(&contract, cell_tx.get_type_arg().unwrap().into()).unwrap();
                    cell_output = cell_output.type_(ScriptOptBuilder::default()
                        .set(Some(script)).build());
                }
            }
            cell_output
        };

        // let data = cell_tx.get_data();
        cell_output = cell_output.capacity((redundant_cap as u64).pack());
        // let expected_length = cell_output.expected_length();
        // cell_output = cell_output.capacity(
        //     ((expected_length + data.len() + redundant_cap) as u64).pack()
        // );

        let witness = match cell_tx.get_witness() {
            None => {
                // BytesBuilder::default().build()
                Bytes::from(vec![])
            }
            Some(witness) => {
                Bytes::from(witness)
            }
        };

        return tx_builder
            .as_advanced_builder()
            .output(cell_output.build().clone())
            .output_data(Bytes::from(cell_tx.get_data()).pack())
            .witness(Pack::pack(&witness)).build();
    }


    // pub fn set_output_cell_by_cell(mut context: Context, tx_builder: TransactionView, lock_contract: OutPoint, type_contract: Option<OutPoint>, cell_tx: &dyn CellMessage, redundant_cap: usize, set_index: usize) -> TransactionView {
    //     return tx_builder;
    //     // let type_script = match contract {
    //     //     None => { ScriptOpt::default() }
    //     //     Some(contract) => {
    //     //         ScriptOptBuilder::default()
    //     //             .set(Some(context
    //     //                 .build_script(&contract, Default::default())
    //     //                 .expect("type script")
    //     //                 .as_builder()
    //     //                 .args(cell_tx.get_arg())
    //     //                 .build())).build()
    //     //     }
    //     // };
    //     //
    //     // let mut cell_output = {
    //     //     let mut cell_output = CellOutputBuilder::default()
    //     //         .lock(owner_lock);
    //     //     if type_script.is_some() {
    //     //         cell_output = cell_output.type_(type_script.to_opt().pack());
    //     //     }
    //     //     cell_output
    //     // };
    //     //
    //     // let data = cell_tx.get_data();
    //     //
    //     // cell_output = cell_output.capacity((redundant_cap as u64).pack());
    //     //
    //     //
    //     // let witness = {
    //     //     let witness_args = WitnessArgs::new_builder()
    //     //         .input_type(cell_tx.get_input_type())
    //     //         .output_type(cell_tx.get_output_type())
    //     //         .build();
    //     //     witness_args.as_bytes()
    //     // };
    //     //
    //     //
    //     // let mut output_cells: Vec<CellOutput> = tx_builder.outputs_with_data_iter()
    //     //     .map(|(cell, data)| cell)
    //     //     .collect();
    //     // output_cells.insert(set_index, cell_output.build());
    //     //
    //     //
    //     // let mut output_data = tx_builder.data().raw().outputs_data().unpack();
    //     // output_data.insert(set_index, data.raw_data());
    //     //
    //     // let mut witnessVec = tx_builder.data().witnesses().unpack();
    //     // witnessVec.insert(set_index, witness);
    //     // tx_builder.as_advanced_builder()
    //     //     .set_outputs_data(vec![])
    //     //     .outputs_data(output_data.pack())
    //     //     .set_outputs(vec![])
    //     //     .outputs(output_cells)
    //     //     .set_witnesses(vec![])
    //     //     .witnesses(witnessVec.pack())
    //     //     .build()
    // }
}