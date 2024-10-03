use num_bigint::BigInt;

use program_structure::ast::{
    produce_program_compiler_version_report, produce_program_version_warning_report,
    produce_report, Expression,
};
use std::str::FromStr;

use program_structure::error_code::ReportCode;
use program_structure::error_definition::Report;
use program_structure::error_definition::ReportCollection;
use program_structure::program_archive::{ProgramArchive, ProgramID};

use super::common::Version;
use super::parser_logic;
use super::syntax_sugar_remover::programs_apply_syntax_sugar;

/*
    Disabling custom gates for Plonk
    if program.custom_gates {
        check_custom_gates_version(
            path,
            program.compiler_version,
            parse_number_version(version),
        )
        .map_err(|e| (file_library.clone(), vec![e]))?
    }
*/

pub fn run_parser(
    version: &str,
    imports: Vec<(String, String)>,
    field: &BigInt,
) -> Result<ProgramArchive, ReportCollection> {
    let mut program_library = ProgramArchive::new();
    let mut definitions = Vec::new();
    let mut main_components = Vec::new();
    let mut reports = Vec::new();

    for (name, src) in imports {
        let program_id = program_library.import(name.clone(), src.clone());
        match parser_logic::parse_program(&src, program_id, field) {
            Err(v) => {
                reports.extend(v);
                continue;
            }
            Ok(ast) => {
                if let Some(main) = ast.main_component {
                    main_components.push((program_id, main, ast.custom_gates));
                }
                definitions.push((program_id, ast.definitions));
                match version_check(
                    program_id,
                    ast.compiler_version,
                    version_number_parser(version),
                ) {
                    Err(v) => {
                        reports.push(v);
                        continue;
                    }
                    Ok(_) => (),
                }
            }
        };
    }

    if main_components.len() == 0 {
        let report = produce_report(ReportCode::NoMainFoundInProject, 0..0, 0);
        reports.push(report);
        return Err(reports);
    } else if main_components.len() > 1 {
        let report = produce_report_for_program(main_components);
        reports.push(report);
        return Err(reports);
    } else {
        // TO-DO Handle Import Graph
        if reports.len() > 0 {
            return Err(reports);
        } else {
            // ignore custom gates for now
            let (main_id, main_component, _) = main_components.pop().unwrap();
            match program_library.merge(main_id, main_component, definitions) {
                Err(v) => {
                    reports.extend(v);
                    return Err(reports);
                }
                Ok(_) => match programs_apply_syntax_sugar(&mut program_library) {
                    Err(v) => {
                        reports.push(v);
                        return Err(reports);
                    }
                    Ok(_) => return Ok(program_library),
                },
            }
        }
    }
}

fn produce_report_for_program(
    main_components: Vec<(usize, (Vec<String>, Expression), bool)>,
) -> Report {
    let mut j = 0;
    let mut r = produce_report(ReportCode::MultipleMain, 0..0, 0);
    for (i, exp, _) in main_components {
        if j > 0 {
            r.add_secondary(
                exp.1.get_meta().location.clone(),
                i,
                Option::Some("Here it is another main component".to_string()),
            );
        } else {
            r.add_primary(
                exp.1.get_meta().location.clone(),
                i,
                "This is a main component".to_string(),
            );
        }
        j += 1;
    }
    r
}

fn version_number_parser(version: &str) -> Version {
    let version_splitted: Vec<&str> = version.split(".").collect();
    (
        usize::from_str(version_splitted[0]).unwrap(),
        usize::from_str(version_splitted[1]).unwrap(),
        usize::from_str(version_splitted[2]).unwrap(),
    )
}

fn version_check(
    program_id: ProgramID,
    version_file: Option<Version>,
    version_compiler: Version,
) -> Result<ReportCollection, Report> {
    if let Some(required_version) = version_file {
        if required_version.0 == version_compiler.0
            && (required_version.1 < version_compiler.1
                || (required_version.1 == version_compiler.1
                    && required_version.2 <= version_compiler.2))
        {
            Ok(vec![])
        } else {
            Err(produce_program_compiler_version_report(
                program_id,
                required_version,
                version_compiler,
            ))
        }
    } else {
        let report = produce_program_version_warning_report(program_id, version_compiler);
        Ok(vec![report])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use program_structure::constants::UsefulConstants;

    #[test]
    fn correct_template_single() {
        let ver = "2.0.0";
        let field = "bn128";
        let prime = UsefulConstants::new(&field.to_string()).get_p().clone();
        let test_template = indoc::indoc! {"
        pragma circom 2.0.0;

        template A(){
            signal input in1;
            signal input in2;
            signal output out;
            out <== in1 * in2;
        }

        component main {public [in1]}= A();
        "};
        let sources = vec![("test".to_string(), test_template.to_string())];
        match run_parser(ver, sources, &prime) {
            Ok(program_library) => {
                // check source code is stored correctly
                match program_library
                    .repo()
                    .get_src(program_library.program_id_main)
                {
                    Ok(src) => {
                        assert_eq!(src, test_template);
                    }
                    Err(_) => assert!(false),
                }
                // check that id_max is not 0
                assert_ne!(program_library.id_max, 0);

                // check template exists
                assert_eq!(program_library.contains_template("A"), true);

                // check main public inputs
                let public_inputs = program_library.get_public_inputs_main_component();
                assert_eq!(public_inputs.len(), 1);
                assert_eq!(public_inputs[0], "in1".to_string());
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn correct_template_multi() {
        let ver = "2.0.0";
        let field = "bn128";
        let prime = UsefulConstants::new(&field.to_string()).get_p().clone();
        let sources = vec![
            (
                "test1.circom".to_string(),
                indoc::indoc! {r#"
        pragma circom 2.0.0;

        include "test2.circom";

        template Mul(Param1, Param2){
            signal input in1;
            signal input in2;
            signal output out;

            component b = B(Param1, Param2);
            b.in1 <== in1;
            b.in2 <== in2;
            out <== b.out;
        }

        template A(Param1, Param2, Param3){
            signal input in1;
            signal input in2;
            signal input in3;
            signal output out;

            component mul = Mul(Param3, Param2);
            mul.in1 <== in1;
            mul.in2 <== in2;

            out <== mul.out;

            sqrd <== Mul()(in3, in3);
        }

        component main {public [in1, in2]}= A(7,8,9);
        "#}
                .to_string(),
            ),
            (
                "test2.circom".to_string(),
                indoc::indoc! {r#"
        pragma circom 2.0.0;

        template B(Param1, Param2){

            assert(Param1 > 0);
            assert(Param2 > 0);

            signal input in1;
            signal input in2;
            signal output out;
            out <== in1 * in2;
        }
        "#}
                .to_string(),
            ),
        ];
        match run_parser(ver, sources, &prime) {
            Ok(program_library) => {
                // check that id_max is not 0
                assert_ne!(program_library.id_max, 0);

                // check template exists

                // Template A
                assert_eq!(program_library.contains_template("A"), true);
                let template_body = program_library.get_template_data("A").get_body_as_vec();
                assert_eq!(template_body.len() > 0, true);

                let template_params = program_library.get_template_data("A").get_name_of_params();
                assert_eq!(template_params.len() == 3, true);
                assert_eq!(template_params[0], "Param1".to_string());
                assert_eq!(template_params[1], "Param2".to_string());
                assert_eq!(template_params[2], "Param3".to_string());

                // Template Mul
                assert_eq!(program_library.contains_template("Mul"), true);
                let template_body = program_library.get_template_data("Mul").get_body_as_vec();
                assert_eq!(template_body.len() > 0, true);

                let template_params = program_library
                    .get_template_data("Mul")
                    .get_name_of_params();
                assert_eq!(template_params.len() == 2, true);
                assert_eq!(template_params[0], "Param1".to_string());
                assert_eq!(template_params[1], "Param2".to_string());

                // Template B
                assert_eq!(program_library.contains_template("B"), true);
                let template_body = program_library.get_template_data("B").get_body_as_vec();
                assert_eq!(template_body.len() > 0, true);

                let template_params = program_library.get_template_data("B").get_name_of_params();
                assert_eq!(template_params.len() == 2, true);
                assert_eq!(template_params[0], "Param1".to_string());
                assert_eq!(template_params[1], "Param2".to_string());

                // check main public inputs
                let public_inputs = program_library.get_public_inputs_main_component();
                assert_eq!(public_inputs.len(), 2);
                assert_eq!(public_inputs[0], "in1".to_string());
                assert_eq!(public_inputs[1], "in2".to_string());
            }
            Err(warnings) => {
                for w in warnings {
                    println!("caught warning: {:?}", w.get_message());
                }
                assert!(false)
            }
        }
    }

    #[test]
    fn template_non_existent() {
        let ver = "2.0.0";
        let field = "bn128";
        let prime = UsefulConstants::new(&field.to_string()).get_p().clone();
        let sources = vec![(
            "test1.circom".to_string(),
            indoc::indoc! {r#"
        pragma circom 2.0.0;

        include "test2.circom";

        template A(Param1, Param2, Param3){
            signal input in1;
            signal input in2;
            signal output out;

            sqrd <== Mul()(in3, in3);
        }

        component main {public [in1, in2]}= A(7,8,9);
        "#}
            .to_string(),
        )];
        match run_parser(ver, sources, &prime) {
            Ok(_) => {
                assert!(false)
            }
            Err(warnings) => {
                assert_eq!(warnings.len(), 1);
                assert_eq!(
                    warnings[0].get_message().to_string(),
                    "The template Mul does not exist".to_string()
                );
            }
        }
    }
    #[test]
    fn main_non_existent() {
        let ver = "2.0.0";
        let field = "bn128";
        let prime = UsefulConstants::new(&field.to_string()).get_p().clone();
        let sources = vec![(
            "test1.circom".to_string(),
            indoc::indoc! {r#"
        pragma circom 2.0.0;

        template A(Param1, Param2, Param3){
            signal input in1;
            signal input in2;
            signal output out;
            out <== in * in2;
        }
        "#}
            .to_string(),
        )];
        match run_parser(ver, sources, &prime) {
            Ok(_) => {
                assert!(false)
            }
            Err(warnings) => {
                assert_eq!(warnings.len(), 1);
                assert_eq!(
                    warnings[0].get_message().to_string(),
                    "No main specified in the project structure".to_string()
                );
            }
        }
    }
    #[test]
    fn multiple_main() {
        let ver = "2.0.0";
        let field = "bn128";
        let prime = UsefulConstants::new(&field.to_string()).get_p().clone();
        let sources = vec![(
            "test1.circom".to_string(),
            indoc::indoc! {r#"
        pragma circom 2.0.0;

        template A(Param1, Param2, Param3){
            signal input in1;
            signal input in2;
            signal output out;
            out <== in * in2;
        }

        component main {public [in1, in2]}= A(7,8,9);

        template B(Param1, Param2, Param3){
            signal input in1;
            signal input in2;
            signal output out;
            out <== in * in2;
        }

        component main {public [in1, in2]}= B(7,8,9);
        "#}
            .to_string(),
        )];
        match run_parser(ver, sources, &prime) {
            Ok(_) => {
                assert!(false)
            }
            Err(warnings) => {
                assert_eq!(warnings.len(), 6);
                assert_eq!(
                    warnings[0].get_message().to_string(),
                    "illegal expression".to_string()
                );
                for error in warnings {
                    println!("caught: {}", error.get_message().to_string())
                }
            }
        }
    }
}
