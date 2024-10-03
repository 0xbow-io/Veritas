use program_structure::ast::Statement;
use program_structure::error_code::ReportCode;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::program_archive::{generate_program_location, ProgramID};
use program_structure::template_data::TemplateData;

pub fn free_of_returns(template_data: &TemplateData) -> Result<(), ReportCollection> {
    let program_id = template_data.get_program_id();
    let template_body = template_data.get_body();
    let mut reports = ReportCollection::new();
    look_for_return(&template_body, program_id, &mut reports);
    if reports.is_empty() {
        Result::Ok(())
    } else {
        Result::Err(reports)
    }
}

fn look_for_return(stmt: &Statement, program_id: ProgramID, reports: &mut ReportCollection) {
    use Statement::*;
    match stmt {
        IfThenElse {
            if_case, else_case, ..
        } => {
            look_for_return(if_case, program_id, reports);
            if let Option::Some(else_block) = else_case {
                look_for_return(else_block, program_id, reports);
            }
        }
        While { stmt, .. } => {
            look_for_return(stmt, program_id, reports);
        }
        Block { stmts, .. } => {
            for stmt in stmts.iter() {
                look_for_return(stmt, program_id, reports);
            }
        }
        Return { meta, .. } => {
            let mut report = Report::error(
                "Return found in template".to_string(),
                ReportCode::TemplateWithReturnStatement,
            );
            report.add_primary(
                generate_program_location(meta.get_start(), meta.get_end()),
                program_id,
                "This return statement is inside a template".to_string(),
            );
            reports.push(report);
        }
        _ => {}
    };
}
