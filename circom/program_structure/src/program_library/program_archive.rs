use super::ast::{Definition, Expression, MainComponent};
use std::ops::Range;

use super::function_data::{FunctionData, FunctionInfo};
use super::program_merger::Merger;
use super::template_data::{TemplateData, TemplateInfo};
use crate::abstract_syntax_tree::ast::FillMeta;
use codespan_reporting::files::{Error, Files, SimpleFiles};
use std::collections::{HashMap, HashSet};
pub type ProgramLocation = Range<usize>;
pub type ProgramID = usize;
use super::error_definition::ReportCollection;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

type Contents = Vec<(ProgramID, Vec<Definition>)>;

#[derive(Clone)]
pub struct ProgramArchive {
    pub id_max: usize,
    pub program_id_main: ProgramID,
    pub functions: FunctionInfo,
    pub templates: TemplateInfo,
    pub function_keys: HashSet<String>,
    pub template_keys: HashSet<String>,
    pub public_inputs: Vec<String>,
    pub initial_template_call: Expression,
    pub custom_gates: bool,
    pub repository: Repository,
}

impl Default for ProgramArchive {
    fn default() -> Self {
        ProgramArchive {
            id_max: 0,
            program_id_main: 0,
            functions: FunctionInfo::new(),
            templates: TemplateInfo::new(),
            function_keys: HashSet::new(),
            template_keys: HashSet::new(),
            public_inputs: Vec::new(),
            initial_template_call: Expression::Default {},
            custom_gates: false,
            repository: Repository::default(),
        }
    }
}

impl ProgramArchive {
    pub fn new() -> ProgramArchive {
        ProgramArchive::default()
    }
    pub fn import(&mut self, name: String, src: String) -> ProgramID {
        self.get_mut_repository().push_src(name, src)
    }
    pub fn repo(&self) -> &Repository {
        &self.get_repo()
    }
    pub fn get_repo(&self) -> &Repository {
        &self.repository
    }
    pub fn get_mut_repository(&mut self) -> &mut Repository {
        &mut self.repository
    }

    pub fn merge(
        &mut self,
        main_program_id: ProgramID,
        main_program_component: MainComponent,
        program_contents: Contents,
    ) -> Result<(), ReportCollection> {
        let mut merger = Merger::new();
        let mut reports = vec![];

        for (program_id, definitions) in program_contents {
            if let Err(mut errs) = merger.add_definitions(program_id, definitions) {
                reports.append(&mut errs);
            }
        }
        if reports.is_empty() {
            (self.id_max, self.functions, self.templates) = merger.decompose();
            for key in self.functions.keys() {
                self.function_keys.insert(key.clone());
            }
            for key in self.templates.keys() {
                self.template_keys.insert(key.clone());
            }

            (self.public_inputs, self.initial_template_call) = main_program_component;
            self.initial_template_call
                .fill(main_program_id, &mut self.id_max);
            return Ok(());
        }
        // Properly handle reports
        return Err(reports);
    }
    pub fn get_main_program_id(&self) -> &ProgramID {
        &self.program_id_main
    }
    //template functions
    pub fn contains_template(&self, template_name: &str) -> bool {
        self.templates.contains_key(template_name)
    }
    pub fn get_template_data(&self, template_name: &str) -> &TemplateData {
        assert!(self.contains_template(template_name));
        self.templates.get(template_name).unwrap()
    }
    pub fn get_mut_template_data(&mut self, template_name: &str) -> &mut TemplateData {
        assert!(self.contains_template(template_name));
        self.templates.get_mut(template_name).unwrap()
    }
    pub fn get_template_names(&self) -> &HashSet<String> {
        &self.template_keys
    }
    pub fn get_templates(&self) -> &TemplateInfo {
        &self.templates
    }
    pub fn get_mut_templates(&mut self) -> &mut TemplateInfo {
        &mut self.templates
    }

    pub fn remove_template(&mut self, id: &str) {
        self.template_keys.remove(id);
        self.templates.remove(id);
    }

    //functions functions
    pub fn contains_function(&self, function_name: &str) -> bool {
        self.get_functions().contains_key(function_name)
    }
    pub fn get_function_data(&self, function_name: &str) -> &FunctionData {
        assert!(self.contains_function(function_name));
        self.get_functions().get(function_name).unwrap()
    }
    pub fn get_mut_function_data(&mut self, function_name: &str) -> &mut FunctionData {
        assert!(self.contains_function(function_name));
        self.functions.get_mut(function_name).unwrap()
    }
    pub fn get_function_names(&self) -> &HashSet<String> {
        &self.function_keys
    }
    pub fn get_functions(&self) -> &FunctionInfo {
        &self.functions
    }
    pub fn get_mut_functions(&mut self) -> &mut FunctionInfo {
        &mut self.functions
    }
    pub fn remove_function(&mut self, id: &str) {
        self.function_keys.remove(id);
        self.functions.remove(id);
    }

    //main_component functions
    pub fn get_public_inputs_main_component(&self) -> &Vec<String> {
        &self.public_inputs
    }
    pub fn get_main_expression(&self) -> &Expression {
        &self.initial_template_call
    }
}

#[derive(Clone)]
pub struct Repository {
    pub latest_id: ProgramID,
    pub store: SimpleFiles<String, String>,
}

// Custom serialization for Repository
impl Serialize for Repository {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = HashMap::new();
        for id in 0..self.latest_id {
            let name = self.store.name(id);
            match name {
                Ok(name) => {
                    let source = self.store.source(id);
                    match source {
                        Ok(source) => {
                            map.insert(name.to_string(), source.to_string());
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
        }
        map.serialize(serializer)
    }
}

// Custom deserialization for Repository
impl<'de> Deserialize<'de> for Repository {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
        let mut repo = Repository::default();
        for (name, source) in map {
            repo.latest_id = repo.push_src(name, source);
        }
        Ok(repo)
    }
}

impl Default for Repository {
    fn default() -> Self {
        Repository {
            latest_id: 0,
            store: SimpleFiles::new(),
        }
    }
}

impl Repository {
    pub fn push_src(&mut self, name: String, src: String) -> ProgramID {
        self.latest_id = self.get_mut_store().add(name, src);
        self.latest_id
    }
    pub fn get_src(&self, id: ProgramID) -> Result<&str, Error> {
        self.store.source(id)
    }
    pub fn get_line(&self, start: usize, program_id: ProgramID) -> Result<usize, Error> {
        self.get_store().line_index(program_id, start)
    }
    pub fn get_mut_store(&mut self) -> &mut SimpleFiles<String, String> {
        &mut self.store
    }
    pub fn get_store(&self) -> &SimpleFiles<String, String> {
        &self.store
    }
}
pub fn generate_program_location(start: usize, end: usize) -> ProgramLocation {
    start..end
}
