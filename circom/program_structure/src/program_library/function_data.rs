use super::ast::{FillMeta, Statement};
use super::program_archive::ProgramID;
use super::program_archive::ProgramLocation;
use std::collections::HashMap;

pub type FunctionInfo = HashMap<String, FunctionData>;

#[derive(Clone)]
pub struct FunctionData {
    name: String,
    program_id: ProgramID,
    num_of_params: usize,
    name_of_params: Vec<String>,
    param_location: ProgramLocation,
    body: Statement,
}

impl FunctionData {
    pub fn new(
        name: String,
        program_id: ProgramID,
        mut body: Statement,
        num_of_params: usize,
        name_of_params: Vec<String>,
        param_location: ProgramLocation,
        elem_id: &mut usize,
    ) -> FunctionData {
        body.fill(program_id, elem_id);
        FunctionData {
            name,
            program_id,
            body,
            name_of_params,
            param_location,
            num_of_params,
        }
    }
    pub fn get_program_id(&self) -> ProgramID {
        self.program_id
    }
    pub fn get_body(&self) -> &Statement {
        &self.body
    }
    pub fn get_body_as_vec(&self) -> &Vec<Statement> {
        match &self.body {
            Statement::Block { stmts, .. } => stmts,
            _ => panic!("Function body should be a block"),
        }
    }
    pub fn get_mut_body(&mut self) -> &mut Statement {
        &mut self.body
    }
    pub fn set_body(&mut self, body: Statement) {
        self.body = body;
    }
    pub fn replace_body(&mut self, new: Statement) -> Statement {
        std::mem::replace(&mut self.body, new)
    }
    pub fn get_mut_body_as_vec(&mut self) -> &mut Vec<Statement> {
        match &mut self.body {
            Statement::Block { stmts, .. } => stmts,
            _ => panic!("Function body should be a block"),
        }
    }
    pub fn get_param_location(&self) -> ProgramLocation {
        self.param_location.clone()
    }
    pub fn get_num_of_params(&self) -> usize {
        self.num_of_params
    }
    pub fn get_name_of_params(&self) -> &Vec<String> {
        &self.name_of_params
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
