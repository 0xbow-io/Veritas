extern crate num_bigint_dig as num_bigint;
extern crate num_traits;

mod compute_constants;
mod environment_utils;
mod execute;
mod execution_data;

use ansi_term::Colour;
use circom_algebra::algebra::{ArithmeticError, ArithmeticExpression};
use compiler::hir::very_concrete_program::VCP;
use dag::DAG;
use execution_data::{executed_program::ExportResult, ExecutedProgram};
use program_structure::ast::{self};
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::program_archive::{ProgramArchive, ProgramID};
use std::rc::Rc;

pub fn build_circuit(library: ProgramArchive, prime: String) -> ExportResult {
    let mut reports = ReportCollection::new();

    match instantiation(&library, &prime) {
        Ok((exe, warnings)) => {
            reports.extend(warnings);
            // export DAG and VCP
            match exe.export(library) {
                Ok((mut dag, mut vcp, new_warnings)) => {
                    reports.extend(new_warnings);
                    // Fast-Flag
                    // No simplification
                    let witness = Rc::new(DAG::produce_witness(&mut dag));
                    VCP::add_witness_list(&mut vcp, Rc::clone(&witness));
                    return Ok((dag, vcp, reports));
                }
                // foward the error
                Err(e) => return Err(e),
            }
        }
        // foward the error
        Err(e) => return Err(e),
    }
}

fn instantiation(
    library: &ProgramArchive,
    prime: &String,
) -> Result<(ExecutedProgram, ReportCollection), ReportCollection> {
    match execute::constraint_execution(&library, prime) {
        Ok((program_exe, warnings)) => {
            let no_nodes = program_exe.number_of_nodes();
            let success = Colour::Green.paint("template instances");
            let nodes_created = format!("{}: {}", success, no_nodes);
            println!("{}", &nodes_created);
            Ok((program_exe, warnings))
        }
        Err(reports) => return Err(reports),
    }
}

/*


// use constraint_writers::ConstraintExporter;

// pub type ConstraintWriter = Box<dyn ConstraintExporter>;
// type BuildResponse = Result<(ConstraintWriter, VCP), ()>;
// removing the simplification process
// let list = simplification_process(&mut vcp, dag, &config);

   fn simplification_process(vcp: &mut VCP, dag: DAG, config: &BuildConfig) -> ConstraintList {
       use dag::SimplificationFlags;
       let flags = SimplificationFlags {
           flag_s: config.flag_s,
           parallel_flag: config.flag_p,
           port_substitution: config.flag_json_sub,
           json_substitutions: config.json_substitutions.clone(),
           no_rounds: config.no_rounds,
           flag_old_heuristics: config.flag_old_heuristics,
           prime: config.prime.clone(),
       };
       let list = DAG::map_to_list(dag, flags);
       VCP::add_witness_list(vcp, Rc::new(list.get_witness_as_vec()));
       list
   }
*/
