
use ckb_std::ckb_types::packed::{ BytesOpt};

use ckb_testtool::ckb_types::prelude::{Builder, Entity, Reader};



// pub fn replace_output_cell_by_cell(mut context: &mut Context, tx_builder: TransactionView, lock_contract: OutPoint, type_contract: Option<OutPoint>, cell_tx: &dyn CellMessage, redundant_cap: usize, replace_index: usize) -> TransactionView {
//     return tx_builder;
//     //let type_script = match contract {
//     //         None => { ScriptOpt::default() }
//     //         Some(contract) => {
//     //             ScriptOptBuilder::default()
//     //                 .set(Some(context
//     //                     .build_script(&contract, Default::default())
//     //                     .expect("type script")
//     //                     .as_builder()
//     //                     .args(cell_tx.get_arg())
//     //                     .build())).build()
//     //         }
//     //     };
//     //
//     //     let mut cell_output = {
//     //         let mut cell_output = CellOutputBuilder::default()
//     //             .lock(owner_lock);
//     //         if type_script.is_some() {
//     //             cell_output = cell_output.type_(type_script.to_opt().pack());
//     //         }
//     //         cell_output
//     //     };
//     //
//     //     let data = cell_tx.get_data();
//     //
//     //     cell_output = cell_output.capacity((redundant_cap as u64).pack());
//     //
//     //
//     //     let witness = {
//     //         let witness_args = WitnessArgs::new_builder()
//     //             .input_type(cell_tx.get_input_type())
//     //             .output_type(cell_tx.get_output_type())
//     //             .build();
//     //         witness_args.as_bytes()
//     //     };
//     //
//     //
//     //     let mut output_cells: Vec<CellOutput> = tx_builder.outputs_with_data_iter()
//     //         .map(|(cell, data)| cell)
//     //         .collect();
//     //     if let Some(old_element) = output_cells.get_mut(replace_index) {
//     //         *old_element = cell_output.build();
//     //     } else {
//     //         println!("Index {} is out of bounds", replace_index);
//     //         return tx_builder;
//     //     }
//     //
//     //
//     //     // output_cells.push(output_cells.first().unwrap().clone());
//     //     let mut output_data = tx_builder.data().raw().outputs_data().unpack();
//     //
//     //     if let Some(old_element) = output_data.get_mut(replace_index) {
//     //         *old_element = data.raw_data();
//     //     } else {
//     //         println!("Index {} is out of bounds", replace_index);
//     //         return tx_builder;
//     //     }
//     //
//     //     let mut witnessVec = tx_builder.data().witnesses().unpack();
//     //     if let Some(old_element) = witnessVec.get_mut(replace_index) {
//     //         *old_element = witness;
//     //     } else {
//     //         witnessVec.push(witness);
//     //         // println!("witness Index {} is out of bounds", replace_index);
//     //         // return tx_builder;
//     //     }
//     //
//     //     tx_builder.as_advanced_builder()
//     //         .set_outputs_data(vec![])
//     //         .outputs_data(output_data.pack())
//     //         .set_outputs(vec![])
//     //         .outputs(output_cells)
//     //         .set_witnesses(vec![])
//     //         .witnesses(witnessVec.pack())
//     //         .build()
// }
