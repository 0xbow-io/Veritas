use codespan_reporting::diagnostic::{Diagnostic, Severity};
use program_structure::{
    error_code::ReportCode,
    error_definition::Report,
    file_definition::{FileID, FileLocation},
};

use super::ffi::ffi_pass_report;

pub(crate) fn report_diagnostic(
    ctx_handle: usize,
    diagnostic: &Diagnostic<FileID>,
    buff: &mut Vec<u8>,
) {
    buff.clear();
    serde_json::to_writer(&mut *buff, diagnostic).unwrap();
    ffi_pass_report(ctx_handle, buff.as_ptr(), buff.len());
}

pub(crate) fn report_error(ctx_handle: usize, err_msg: &str, buff: &mut Vec<u8>) {
    let report: Diagnostic<FileID> = Diagnostic::new(Severity::Error).with_message(err_msg);
    report_diagnostic(ctx_handle, &report, buff);
}

pub fn produce_report(error_code: ReportCode, location: FileLocation, file_id: FileID) -> Report {
    use ReportCode::*;
    let report = match error_code {
        UnclosedComment => {
            let mut report =
                Report::error("unterminated /* */".to_string(), ReportCode::UnclosedComment);
            report.add_primary(location, file_id, "Comment starts here".to_string());
            report
        }
        NoMainFoundInProject => Report::error(
            "No main specified in the project structure".to_string(),
            ReportCode::NoMainFoundInProject,
        ),
        MultipleMain => Report::error(
            "Multiple main components in the project structure".to_string(),
            ReportCode::MultipleMain,
        ),
        MissingSemicolon => {
            let mut report =
                Report::error(format!("Missing semicolon"), ReportCode::MissingSemicolon);
            report.add_primary(location, file_id, "A semicolon is needed here".to_string());
            report
        }
        UnrecognizedInclude => {
            let mut report = Report::error(
                "unrecognized argument in include directive".to_string(),
                ReportCode::UnrecognizedInclude,
            );
            report.add_primary(location, file_id, "this argument".to_string());
            report
        }
        UnrecognizedPragma => {
            let mut report = Report::error(
                "unrecognized argument in pragma directive".to_string(),
                ReportCode::UnrecognizedPragma,
            );
            report.add_primary(location, file_id, "this argument".to_string());
            report
        }
        UnrecognizedVersion => {
            let mut report = Report::error(
                "unrecognized version argument in pragma directive".to_string(),
                ReportCode::UnrecognizedVersion,
            );
            report.add_primary(location, file_id, "this argument".to_string());
            report
        }
        IllegalExpression => {
            let mut report =
                Report::error("illegal expression".to_string(), ReportCode::IllegalExpression);
            report.add_primary(location, file_id, "here".to_string());
            report
        }
        MultiplePragma => {
            let mut report =
                Report::error("Multiple pragma directives".to_string(), ReportCode::MultiplePragma);
            report.add_primary(location, file_id, "here".to_string());
            report
        }
        ExpectedIdentifier => {
            let mut report = Report::error(
                "An identifier is expected".to_string(),
                ReportCode::ExpectedIdentifier,
            );
            report.add_primary(location, file_id, "This should be an identifier".to_string());
            report
        }
        _ => unreachable!(),
    };
    report
}

pub fn produce_report_with_main_components(
    main_components: &crate::circuit::MainComponents,
) -> Report {
    let mut j = 0;
    let mut r = produce_report(ReportCode::MultipleMain, 0..0, 0);
    for (i, exp, _) in main_components {
        if j > 0 {
            r.add_secondary(
                exp.1.get_meta().location.clone(),
                i.clone(),
                Option::Some("Here it is another main component".to_string()),
            );
        } else {
            r.add_primary(
                exp.1.get_meta().location.clone(),
                i.clone(),
                "This is a main component".to_string(),
            );
        }
        j += 1;
    }
    r
}
