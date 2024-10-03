use crate::witness;
use ansi_term::Colour;
use compiler::{
    circuit_design::circuit::{Circuit, CompilationFlags},
    hir::very_concrete_program::VCP,
};
use constraint_generation::build_circuit;
use dag::DAG;
use parser::parser::run_parser;
use program_structure::{
    constants::UsefulConstants,
    error_definition::{Report, ReportCollection},
    program_archive::ProgramArchive,
};
use std::io::BufWriter;
use type_analysis::check_types::check_types;

pub fn compile_circom_witness_calculator(
    version: String,
    prime: String,
    sources: Vec<(String, String)>,
) -> Result<(DAG, witness::WitnessCalculator), ReportCollection> {
    let mut wasm_buff = Vec::new();
    match compile_circom_circuit(version, prime, sources) {
        Ok((dag, _, circuit)) => {
            generate_wasm_bin(&circuit, &mut wasm_buff);
            match generate_witness_calculator(&mut wasm_buff) {
                Ok(witness_calculator) => Result::Ok((dag, witness_calculator)),
                Err(e) => Result::Err(vec![e]),
            }
        }
        Err(reports) => Result::Err(reports),
    }
}

pub fn generate_witness_calculator(wasm: &Vec<u8>) -> Result<witness::WitnessCalculator, Report> {
    let mut witness_calculator = witness::WitnessCalculator::default();
    match witness_calculator.load(wasm) {
        Ok(_) => {
            println!(
                "generate_witness_calculator {} ...",
                Colour::Green.paint("Successful"),
            );
            Ok(witness_calculator)
        }
        Err(e) => {
            println!(
                "generate_witness_calculator {} ...",
                Colour::Red.paint("Failed"),
            );
            Err(e)
        }
    }
}

pub fn generate_wasm_bin(circuit: &Circuit, buff: &mut Vec<u8>) {
    let mut c = BufWriter::new(buff);
    let _ = circuit.gen_wasm_bin(&mut c);
    println!(
        "generate_wasm_bin {} ...",
        Colour::Green.paint("Successful"),
    );
}

pub fn compile_circom_circuit(
    version: String,
    prime: String,
    sources: Vec<(String, String)>,
) -> Result<(DAG, VCP, Circuit), ReportCollection> {
    match preprocess_circom_circuit(version.clone(), prime.clone(), sources) {
        Ok((dag, vcp)) => match build_circom_circuit(vcp.clone(), String::from(version.as_str())) {
            Ok(circuit) => Result::Ok((dag, vcp, circuit)),
            Err(_) => Err(vec![]),
        },
        Err(reports) => Result::Err(reports),
    }
}

pub fn build_circom_circuit(vcp: VCP, version: String) -> Result<Circuit, ()> {
    let circuit = Circuit::build(vcp, CompilationFlags::default(), version.as_str());
    println!(
        "build_circom_circuit {} ...",
        Colour::Green.paint("Successful"),
    );
    Ok(circuit)
}

pub fn preprocess_circom_circuit(
    version: String,
    prime: String,
    sources: Vec<(String, String)>,
) -> Result<(DAG, VCP), ReportCollection> {
    match build_program_archive(version, prime.clone(), sources) {
        Ok(mut program_archive) => match do_type_analysis(&mut program_archive) {
            Ok(mut warnings) => match compile_vcp(prime, program_archive) {
                Ok((dag, vcp, reports)) => {
                    warnings.extend(reports);
                    if warnings.len() == 0 {
                        Result::Ok((dag, vcp))
                    } else {
                        Result::Err(warnings)
                    }
                }
                Err(errors) => {
                    warnings.extend(errors);
                    Result::Err(warnings)
                }
            },
            Err(errors) => Result::Err(errors),
        },
        Err(errors) => Result::Err(errors),
    }
}

pub fn build_program_archive(
    version: String,
    prime: String,
    sources: Vec<(String, String)>,
) -> Result<ProgramArchive, ReportCollection> {
    let prime_bigint = UsefulConstants::new(&prime).get_p().clone();
    // Parse the sources and return the program library.
    match run_parser(&version, sources, &prime_bigint) {
        Ok(program_archive) => {
            println!(
                "build_program_archive {} .. Got {} Reports.",
                Colour::Green.paint("Successful"),
                Colour::Yellow.paint("0")
            );
            Ok(program_archive)
        }
        Err(reports) => {
            println!(
                "build_program_archive {} .. Got {} Reports.",
                Colour::Red.paint("Failed"),
                Colour::Yellow.paint(reports.len().to_string()),
            );
            Err(reports)
        }
    }
}

pub fn do_type_analysis(
    program_archive: &mut ProgramArchive,
) -> Result<ReportCollection, ReportCollection> {
    // Perform type checks
    match check_types(program_archive) {
        Ok(warnings) => {
            println!(
                "do_type_analysis {} .. Got {} Reports.",
                Colour::Green.paint("Successful"),
                Colour::Yellow.paint(warnings.len().to_string()),
            );
            Ok(warnings)
        }
        Err(errors) => {
            println!(
                "do_type_analysis {} .. Got {} Reports.",
                Colour::Red.paint("Failed"),
                Colour::Yellow.paint(errors.len().to_string()),
            );
            Err(errors)
        }
    }
}

pub fn compile_vcp(
    prime: String,
    program_archive: ProgramArchive,
) -> Result<(DAG, VCP, ReportCollection), ReportCollection> {
    // Build the circuit
    match build_circuit(program_archive, prime) {
        Ok((dag, vcp, reports)) => {
            println!(
                "compile_vcp {} .. Got {} Reports.",
                Colour::Green.paint("Successful"),
                Colour::Yellow.paint(reports.len().to_string()),
            );
            Result::Ok((dag, vcp, reports))
        }
        Err(reports) => {
            println!(
                "compile_vcp {} .. Got {} Reports.",
                Colour::Red.paint("Failed"),
                Colour::Yellow.paint(reports.len().to_string()),
            );
            Result::Err(reports)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn generate_witness_joined_temp() {
        let inputs_str = r#"{"in": ["1", "2"]}"#;
        let circuit_inputs = witness::parse_inputs(inputs_str);

        let sources = vec![(
            "main.circom".to_string(),
            indoc::indoc! {"
            pragma circom 2.0.0;

            template Internal() {
               signal input in[2];
               signal output out;
               out <== in[0]*in[1];
            }

            template Test() {
               signal input in[2];
               signal output out;
               component c = Internal ();
               c.in[0] <== in[0];
               c.in[1] <== in[1]+2*in[0]+1;
               c.out ==> out;
            }

            component main {public[in]}= Test();
        "}
            .to_string(),
        )];

        match compile_circom_witness_calculator(
            String::from("2.0.0"),
            String::from("bn128"),
            sources,
        ) {
            Ok((_, mut witness_calculator)) => {
                match witness_calculator.calculate_witness(circuit_inputs) {
                    Ok(_) => {
                        /*
                            let mut sym_buff = Vec::new();

                            json_export::build_json_output(&dag, &witness, &mut sym_buff);

                            let sym = jsonbb::ValueRef::from_bytes(&sym_buff);
                            println!("{}", sym.to_string());

                            for value in witness {
                                println!("{}", value.to_string());
                            }
                        */
                        assert!(true);
                    }
                    Err(e) => {
                        println!("{}", e.to_string());
                        assert!(false);
                    }
                }
            }
            Err(e) => {
                for error in e {
                    println!("{}", error.to_string());
                }
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn generate_witness_multi_temp() {
        let inputs_str = r#"{"in": ["1", "2"], "nonce": "0x011"}"#;
        let circuit_inputs = witness::parse_inputs(inputs_str);

        let sources = vec![
            (
                "template_a.circom".to_string(),
                indoc::indoc! {"
            pragma circom 2.0.0;
            include \"template_b.circom\";
            include \"template_sink.circom\";

            template A(ParamA, ParamB){
                signal input in[2];
                signal input nonce;
                signal output out;

                component b = B(ParamA, ParamB);
                var i = 0;
                while(i < 2){
                    b.in[i] <== in[i];
                    i++;
                }

                out <== b.out;
                Sink()(nonce);
            }
            component main {public[in]}= A(9, 10);
        "}
                .to_string(),
            ),
            (
                "template_b.circom".to_string(),
                indoc::indoc! {"
            pragma circom 2.0.0;

            include \"template_d.circom\";

            function sum(a, b) {
                var c = a + b;
                return c;
            }

            template B(ParamA, ParamB){
                assert(ParamA > 0);
                assert(ParamB > 0);

                signal input in[2];
                signal output out;

                out <== D(sum(ParamA, ParamB))(in[0], in[1]);
            }
        "}
                .to_string(),
            ),
            (
                "template_d.circom".to_string(),
                indoc::indoc! {"
            pragma circom 2.0.0;
            template D(SUM){
                signal input a, b;
                signal output out;

                var x = a * SUM;
                var y = b * SUM;

                out <== x + y;
            }
        "}
                .to_string(),
            ),
            (
                "sink.circom".to_string(),
                indoc::indoc! {"
            pragma circom 2.0.0;
            template Sink(){
                signal input in;
                _ <== in * in;
            }
        "}
                .to_string(),
            ),
        ];

        match compile_circom_witness_calculator(
            String::from("2.0.0"),
            String::from("bn128"),
            sources,
        ) {
            Ok((_, mut witness_calculator)) => {
                match witness_calculator.calculate_witness(circuit_inputs) {
                    Ok(_) => {
                        /*
                            let mut sym_buff = Vec::new();
                            let mut sym_buff = Vec::new();

                            json_export::build_json_output(&dag, &witness, &mut sym_buff);

                            let sym = jsonbb::ValueRef::from_bytes(&sym_buff);
                            println!("{}", sym.to_string());

                            for value in witness {
                                println!("{}", value.to_string());
                            }
                        */
                    }
                    Err(e) => {
                        println!("{}", e.to_string());
                        assert!(false);
                    }
                }
            }
            Err(e) => {
                for error in e {
                    println!("{}", error.to_string());
                }
                assert!(false);
            }
        }
    }

    #[test]
    fn undeclared_symbol() {
        let sources = vec![(
            "test".to_string(),
            indoc::indoc! {"
        pragma circom 2.0.0;

        template A(){
            signal input in1;
            signal input in2;
            signal output out;
            out <== in1 * in3;
        }

        component main {public [in1]}= A();
        "}
            .to_string(),
        )];

        match preprocess_circom_circuit(String::from("2.0.0"), String::from("bn128"), sources) {
            Ok(_) => {
                assert!(false);
            }
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(
                    errors[0].get_message().to_string(),
                    "Undeclared symbol".to_string()
                );
            }
        }
    }

    #[test]
    fn invalid_public_list() {
        let sources = vec![(
            "test".to_string(),
            indoc::indoc! {"
        pragma circom 2.0.0;

        template A(){
            signal input in1;
            signal input in2;
            signal output out;
            out <== in1 * in2;
        }

        component main {public [in3]}= A();
        "}
            .to_string(),
        )];

        match preprocess_circom_circuit(String::from("2.0.0"), String::from("bn128"), sources) {
            Ok(_) => {
                assert!(false);
            }
            Err(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(
                    errors[0].get_message().to_string(),
                    "Invalid public list".to_string()
                );
            }
        }
    }
}

/*
let debug = DebugWriter::new(config.json_constraints).unwrap();
    if config.r1cs_flag {
        generate_output_r1cs(&config.r1cs, exporter.as_ref(), custom_gates)?;
    }
    if config.sym_flag {
        generate_output_sym(&config.sym, exporter.as_ref())?;
    }
    if config.json_constraint_flag {
        generate_json_constraints(&debug, exporter.as_ref())?;
    }
*/

/*
fn generate_output_r1cs(
    file: &str,
    exporter: &dyn ConstraintExporter,
    custom_gates: bool,
) -> Result<(), ()> {
    if let Result::Ok(()) = exporter.r1cs(file, custom_gates) {
        println!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        eprintln!(
            "{}",
            Colour::Red.paint("Could not write the output in the given path")
        );
        Result::Err(())
    }
}

fn generate_output_sym(file: &str, exporter: &dyn ConstraintExporter) -> Result<(), ()> {
    if let Result::Ok(()) = exporter.sym(file) {
        println!("{} {}", Colour::Green.paint("Written successfully:"), file);
        Result::Ok(())
    } else {
        eprintln!(
            "{}",
            Colour::Red.paint("Could not write the output in the given path")
        );
        Result::Err(())
    }
}

fn generate_json_constraints(
    debug: &DebugWriter,
    exporter: &dyn ConstraintExporter,
) -> Result<(), ()> {
    if let Ok(()) = exporter.json_constraints(&debug) {
        println!(
            "{} {}",
            Colour::Green.paint("Constraints written in:"),
            debug.json_constraints
        );
        Result::Ok(())
    } else {
        eprintln!(
            "{}",
            Colour::Red.paint("Could not write the output in the given path")
        );
        Result::Err(())
    }
}
*/
