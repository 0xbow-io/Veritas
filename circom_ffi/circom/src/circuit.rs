pub extern crate num_bigint_dig as num_bigint;
pub extern crate num_traits;
use constraint_writers::sym_writer::SymElem;
use num_bigint::BigInt;

use crate::constraint_system::*;
use compiler::compiler_interface::{Circuit, CompilationFlags};

use parser::{apply_sugar, generate_ast};
use type_analysis::check_types::check_types;
use program_structure::{
    ast::Definition,
    ast::MainComponent,
    constants::UsefulConstants,
    error_code::ReportCode,
    error_definition::{Report, ReportCollection},
    file_definition::{FileID, FileLibrary},
    program_archive::ProgramArchive,
};

use std::rc::Rc;
use constraint_generation::{FlagsExecution, execute::constraint_execution};
use compiler::compiler_interface::VCP;

use dag::SimplificationFlags;

use serde::Deserialize;
use ansi_term::Colour;

#[derive(Deserialize, Clone)]
pub struct Program {
    pub identity: String,
    pub src: String,
}

pub type Programs = Vec<Program>;

#[derive(Deserialize)]
pub struct CircuitPkg {
    pub target_version: String,
    pub field: String,
    pub programs: Programs,
}

impl Default for CircuitPkg {
    fn default() -> Self {
        CircuitPkg {
            target_version: "2.2.0".to_string(),
            field: "bn128".to_string(),
            programs: Vec::new(),
        }
    }
}

pub fn create_default_circuit_pkg(programs: &Programs) -> CircuitPkg {
    let pkg = CircuitPkg::default();
    CircuitPkg { programs: programs.to_vec(), ..pkg }
}

pub type Definitions = Vec<(FileID, Vec<Definition>)>;
pub type MainComponents = Vec<(FileID, MainComponent, bool)>;

pub struct ParserOutput {
    pub definitions: Definitions,
    pub main_components: MainComponents,
    pub reports: ReportCollection,
}
impl Default for ParserOutput {
    fn default() -> Self {
        ParserOutput { definitions: Vec::new(), main_components: Vec::new(), reports: Vec::new() }
    }
}

pub struct CircuitLibrary {
    target_version: String,
    prime_field: String,

    simplification_flags: SimplificationFlags,

    catalog: Vec<(FileID, String)>,

    wc: crate::witness::WitnessCalculator,
    constraint_system: ConstraintSystem,

    inner: FileLibrary,
}

impl Default for CircuitLibrary {
    fn default() -> Self {
        CircuitLibrary {
            target_version: "2.2.0".to_string(),
            prime_field: "bn128".to_string(),
            catalog: Vec::new(),
            inner: FileLibrary::new(),
            constraint_system: ConstraintSystem::default(),
            wc: crate::witness::WitnessCalculator::default(),
            simplification_flags: SimplificationFlags {
                no_rounds: 1,
                flag_s: true,
                parallel_flag: false,
                port_substitution: false,
                flag_old_heuristics: false,
                prime: "bn128".to_string(),
                json_substitutions: "".to_string(),
            },
        }
    }
}

impl CircuitLibrary {
    pub fn get_circuit_design(&self, id: FileID) -> (String, String) {
        let store = self.inner.to_storage();
        let program = store.get(id).unwrap();
        (program.name().clone(), program.source().clone())
    }
    pub fn store_circuit(&mut self, circuit_pkg: &CircuitPkg) {
        for program in circuit_pkg.programs.iter() {
            let file_id = self.inner.add_file(program.identity.clone(), program.src.clone());
            self.catalog.push((file_id, program.identity.clone()));
        }
        println!(
            "{} .. {} Programs Available",
            Colour::Green.paint("Pkg has been unpacked into Circuit Library"),
            self.catalog.len()
        );
    }

    pub fn parse(&self) -> ParserOutput {
        let mut output = ParserOutput::default();
        let prime_field_bigint = UsefulConstants::new(&self.prime_field).get_p().clone();
        let store = self.inner.to_storage();
        for (id, _) in self.catalog.iter() {
            let program = store.get(*id).unwrap();
            // Parse the sources and return the program library.
            match generate_ast(*id, program.source(), &prime_field_bigint) {
                Ok(ast) => {
                    if let Some(main) = ast.main_component {
                        output.main_components.push((*id, main, ast.custom_gates));
                    }
                    output.definitions.push((*id, ast.definitions));
                }
                Err(report) => {
                    for report in report.iter() {
                        output.reports.push(report.clone());
                    }
                }
            }
        }

        if output.main_components.len() == 0 {
            let report =
                crate::reporting::produce_report(ReportCode::NoMainFoundInProject, 0..0, 0);
            output.reports.push(report);
        } else if output.main_components.len() > 1 {
            let report =
                crate::reporting::produce_report_with_main_components(&output.main_components);
            output.reports.push(report);
        }
        output
    }

    pub fn build_program_archive(&self) -> Result<ProgramArchive, ReportCollection> {
        let mut parsed_data = self.parse();
        if parsed_data.reports.len() > 0 {
            Report::print_reports(&parsed_data.reports, &self.inner);
            return Err(parsed_data.reports);
        } else {
            let (main_id, main_component, custom_gates) =
                parsed_data.main_components.pop().unwrap();
            let result_program_archive = ProgramArchive::new(
                self.inner.clone(),
                main_id,
                main_component,
                parsed_data.definitions,
                custom_gates,
            );
            match result_program_archive {
                Err((lib, rep)) => {
                    Report::print_reports(&rep, &lib);
                    Err(rep)
                }
                Ok(mut program_archive) => {
                    let program_archive_result = apply_sugar(&mut program_archive);
                    match program_archive_result {
                        Result::Err(v) => {
                            Report::print_reports(&v, &self.inner);
                            Err(v)
                        }
                        Result::Ok(_) => Ok(program_archive),
                    }
                }
            }
        }
    }

    pub fn generate_constraints(
        &mut self,
        program: ProgramArchive,
    ) -> Result<(VCP, ReportCollection), ReportCollection> {
        let flags = FlagsExecution { verbose: true, inspect: true };
        let execution_result = constraint_execution(&program, flags, &self.prime_field);
        match execution_result {
            Ok((program_exe, warnings)) => {
                Report::print_reports(&warnings, &self.inner);
                match program_exe.export(program, flags) {
                    Ok((dag, mut vcp, warnings)) => {
                        let list = dag.map_to_list(SimplificationFlags {
                            flag_s: self.simplification_flags.flag_s,
                            parallel_flag: self.simplification_flags.parallel_flag,
                            port_substitution: self.simplification_flags.port_substitution,
                            json_substitutions: self
                                .simplification_flags
                                .json_substitutions
                                .clone(),
                            no_rounds: self.simplification_flags.no_rounds,
                            flag_old_heuristics: self.simplification_flags.flag_old_heuristics,
                            prime: self.prime_field.clone(),
                        });
                        VCP::add_witness_list(&mut vcp, Rc::new(list.get_witness_as_vec()));
                        self.constraint_system.sync(&list);
                        Ok((vcp, warnings))
                    }
                    Err(reports) => {
                        return Err(reports);
                    }
                }
            }
            Err(reports) => {
                Report::print_reports(&reports, &self.inner);
                Err(reports)
            }
        }
    }

    pub fn compile(
        &mut self,
        circuit_pkg: &CircuitPkg,
    ) -> Result<ReportCollection, ReportCollection> {
        // store the circuit designs
        self.store_circuit(circuit_pkg);

        // build the program archive
        let program_archive = self.build_program_archive();
        match program_archive {
            Ok(mut program_archive) => {
                // do type checking
                match do_type_analysis(&mut program_archive) {
                    // generate constraints
                    Ok(warnings) => {
                        Report::print_reports(&warnings, &self.inner);
                        match self.generate_constraints(program_archive) {
                            Ok((vcp, warnings)) => {
                                // compile the circuit
                                let circuit = Circuit::build(
                                    vcp,
                                    CompilationFlags { main_inputs_log: false, wat_flag: false },
                                    &self.target_version,
                                );
                                // build the witness calculator
                                match crate::witness::WitnessCalculator::new(&circuit) {
                                    Ok(wc) => {
                                        self.wc = wc;
                                        Ok(warnings)
                                    }
                                    Err(v) => Err(vec![v]),
                                }
                            }
                            Err(v) => Err(v),
                        }
                    }
                    Err(v) => Err(v),
                }
            }
            Err(v) => Err(v),
        }
    }
    pub fn execute(
        &mut self,
        input_json: &str,
    ) -> Result<(Vec<BigInt>, LCRecords), ReportCollection> {
        // parse inputs
        let circuit_inputs = crate::witness::parse_inputs(input_json);
        // calculate witness
        let witness = self.wc.calculate_witness(circuit_inputs);
        // evaluate constraints
        match witness {
            Ok(w) => Ok((w.clone(), self.constraint_system.eval_constraints(&w))),
            Err(e) => return Err(vec![e]),
        }
    }

    pub fn get_signals(&self) -> (Vec<&SymElem>, Vec<&SymElem>) {
        self.constraint_system.signals()
    }
}

pub fn do_type_analysis(
    program_archive: &mut ProgramArchive,
) -> Result<ReportCollection, ReportCollection> {
    // Perform type checks
    match check_types(program_archive) {
        Ok(warnings) => Ok(warnings),
        Err(errors) => Err(errors),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn generate_witness_multi_temp() {
        let inputs_str = r#"{"a":1, "b":2, "nonce": "3"}"#;

        let mut library = CircuitLibrary::default();
        let progs: Programs = vec![
            Program {
                identity: "main".to_string(),
                src: indoc::indoc! {"
                        component main {public[a]}= A(5,2);
                    "}
                .to_string(),
            },
            Program {
                identity: "A".to_string(),
                src: indoc::indoc! {"
                template A(A, B){
                    signal input a;
                    signal input b;

                    signal input nonce;
                    signal output out;

                    var c = B()(a, b, nonce);
                    0 === c - A;
                    0 === a * b - B;
                    out <== 1;
                }"
                }
                .to_string(),
            },
            Program {
                identity: "B".to_string(),
                src: indoc::indoc! {"
                template B(){
                    signal input a;
                    signal input b;
                    signal input nonce;
                    signal output out;

                    _ <== C()(nonce);

                    0 === nonce - (a + b);
                    out <== 1;
                }"
                }
                .to_string(),
            },
            Program {
                identity: "VARS".to_string(),
                src: indoc::indoc! {"
                function var_a(){
                    return 3;
                }
                function var_b(){
                    return 3;
                }
                function var_c(){
                    return 3;
                }
                "
                }
                .to_string(),
            },
            Program {
                identity: "C".to_string(),
                src: indoc::indoc! {"
                template C(){
                    var a = var_a();
                    var b = var_b();
                    var c = var_c();

                    signal input nonce;
                    signal output out;

                    signal sqrd <== nonce * nonce;
                    0 === sqrd - (a + b + c);

                    out <== 1;
                }"
                }
                .to_string(),
            },
        ];
        let pkg = create_default_circuit_pkg(&progs);
        match library.compile(&pkg) {
            Ok(warnings) => {
                Report::print_reports(&warnings, &library.inner);
                match library.execute(inputs_str) {
                    Ok((witness, records)) => {
                        let (x, y) = library.get_signals();
                        let r_str = crate::json_export::produce_constraint_evaluation_json(
                            &records, &x, &y, &witness,
                        );
                        println!("{}", r_str);
                    }
                    Err(v) => {
                        Report::print_reports(&v, &library.inner);
                        assert!(false);
                    }
                }
            }
            Err(v) => {
                Report::print_reports(&v, &library.inner);
                assert!(false);
            }
        }
    }
}
