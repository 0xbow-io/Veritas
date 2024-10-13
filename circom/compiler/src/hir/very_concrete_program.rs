use num_bigint_dig::BigInt;
use program_structure::ast::{SignalType, Statement};
use program_structure::program_archive::{ProgramArchive, Repository};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::rc::Rc;

pub type VCT = Vec<usize>;
pub type Length = usize;
pub type Code = Statement;

pub type TagInfo = BTreeMap<String, Option<BigInt>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Argument {
    pub name: String,
    pub values: Vec<BigInt>,
    pub lengths: Vec<Length>,
}
impl PartialEq for Argument {
    fn eq(&self, other: &Self) -> bool {
        self.values.eq(&other.values)
    }
}

#[derive(Clone)]
pub enum Wire{
    TSignal(Signal),
    TBus(Bus)
}
impl Wire {

    pub fn xtype(&self) -> SignalType {
        match self{
            Wire::TSignal(s) => {
                s.xtype
            },
            Wire::TBus(s) => {
                s.xtype
            },
        }
    }
    pub fn name(&self) -> &String {
        match self{
            Wire::TSignal(s) => {
                &s.name
            },
            Wire::TBus(s) => {
                &s.name
            },
        }
    }
    pub fn lengths(&self) -> &Vec<usize> {
        match self{
            Wire::TSignal(s) => {
                &s.lengths
            },
            Wire::TBus(s) => {
                &s.lengths
            },
        }
    }
    pub fn local_id(&self) -> usize {
        match self{
            Wire::TSignal(s) => {
                s.local_id
            },
            Wire::TBus(s) => {
                s.local_id
            },
        }
    }
    pub fn dag_local_id(&self) -> usize {
        match self{
            Wire::TSignal(s) => {
                s.dag_local_id
            },
            Wire::TBus(s) => {
                s.dag_local_id
            },
        }
    }
    pub fn size(&self) -> usize {
        match self{
            Wire::TSignal(s) => {
                s.size
            },
            Wire::TBus(s) => {
                s.size
            },
        }
    }
    pub fn bus_id(&self) -> Option<usize> {
        match self{
            Wire::TSignal(_s) => {
                None
            },
            Wire::TBus(s) => {
                Some(s.bus_id)
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Signal {
    pub name: String,
    pub lengths: Vec<Length>,
    pub xtype: SignalType,
    pub local_id: usize,
    pub dag_local_id: usize,
    pub size: usize,
}

#[derive(Clone)]
pub struct Bus{
    pub name: String,
    pub lengths: Vec<Length>,
    pub xtype: SignalType,
    pub local_id: usize,
    pub dag_local_id: usize,
    pub bus_id: usize, // position of the bus in the table of the buses
    pub size: usize,
}

#[derive(Clone, Debug)]
pub struct FieldInfo{
    pub field_id: usize,
    pub offset: usize,
    pub dimensions: Vec<usize>,
    pub size: usize,
    pub bus_id: Option<usize>, // indicates the position of the bus in the table    
}

#[derive(Clone, Debug)]
pub struct BusInstance{
    pub name: String,
    pub size: usize,
    pub fields: BTreeMap<String, FieldInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Component {
    pub name: String,
    pub lengths: Vec<Length>,
}

impl Component {
    pub fn size(&self) -> usize {
        self.lengths.iter().fold(1, |p, c| p * (*c))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trigger {
    pub runs: String,
    pub offset: usize,
    pub component_offset: usize,
    pub template_id: usize,
    pub component_name: String,
    pub indexed_with: Vec<usize>,
    pub external_wires: Vec<Wire>,
    pub has_inputs: bool,
    pub is_parallel: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ClusterType {
    Mixed {
        tmp_name: String,
    },
    Uniform {
        offset_jump: usize,
        component_offset_jump: usize,
        instance_id: usize,
        header: String,
    },
}
#[derive(Serialize, Deserialize, Clone)]
pub struct TriggerCluster {
    pub cmp_name: String,
    pub slice: Range<usize>,
    pub length: usize,
    pub xtype: ClusterType,
    pub defined_positions: Vec<Vec<usize>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TemplateInstance {
    pub is_parallel: bool,
    pub is_parallel_component: bool,
    pub is_not_parallel_component: bool,
    pub has_parallel_sub_cmp: bool,
    pub template_name: String,
    pub template_header: String,
    pub template_id: usize,
    pub header: Vec<Argument>,
    pub number_of_inputs: usize,
    pub number_of_outputs: usize,
    pub number_of_intermediates: usize,
    pub wires: Vec<Wire>,
    pub signals_to_tags: BTreeMap<String, TagInfo>,
    pub components: Vec<Component>,
    pub number_of_components: usize,
    pub triggers: Vec<Trigger>,
    pub clusters: Vec<TriggerCluster>,
    pub code: Code,
}

pub struct TemplateConfig {
    pub is_parallel: bool,
    pub has_parallel_sub_cmp: bool,
    pub name: String,
    pub header: String,
    pub id: usize,
    pub code: Statement,
    pub number_of_components: usize,
    pub triggers: Vec<Trigger>,
    pub clusters: Vec<TriggerCluster>,
    pub components: Vec<Component>,
    pub arguments: Vec<Argument>,
    pub signals_to_tags: BTreeMap<String, TagInfo>,
}
impl TemplateInstance {
    pub fn new(config: TemplateConfig) -> TemplateInstance {
        TemplateInstance {
            is_parallel: config.is_parallel,
            is_parallel_component: false,
            is_not_parallel_component: false,
            has_parallel_sub_cmp: config.has_parallel_sub_cmp,
            code: config.code,
            template_name: config.name,
            template_header: config.header,
            template_id: config.id,
            header: config.arguments,
            number_of_inputs: 0,
            number_of_outputs: 0,
            number_of_intermediates: 0,
            number_of_components: config.number_of_components,
            wires: Vec::new(),
            components: config.components,
            triggers: config.triggers,
            clusters: config.clusters,
            signals_to_tags: config.signals_to_tags,
        }
    }

    pub fn add_signal(&mut self, wire: Wire) {
        use SignalType::*;
        let new_signals = wire.size();
        match wire.xtype() {
            Input => {
                self.number_of_inputs += new_signals;
            }
            Output => {
                self.number_of_outputs += new_signals;
            }
            Intermediate => {
                self.number_of_intermediates += new_signals;
            }
        }
        self.wires.push(wire);
    }
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub length: VCT,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VCF {
    pub name: String,
    pub header: String,
    pub params_types: Vec<Param>,
    pub return_type: VCT,
    pub body: Statement,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Stats {
    pub all_signals: usize,
    pub io_signals: usize,
    pub all_created_components: usize,
    pub all_needed_subcomponents_indexes: usize,
}

#[derive(Clone)]
pub struct VCPConfig {
    pub stats: Stats,
    pub main_id: usize,
    pub repository: Repository,
    pub templates: Vec<TemplateInstance>,
    pub templates_in_mixed: Vec<usize>,
    pub library: ProgramArchive,
    pub prime: String,
    pub buses: Vec<BusInstance>,
}

pub type WitnessList = Rc<Vec<usize>>;

mod rc_vec_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::rc::Rc;

    pub fn serialize<S>(list: &Rc<Vec<usize>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (**list).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Rc<Vec<usize>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::deserialize(deserializer).map(Rc::new)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VCP {
    pub stats: Stats,
    pub main_id: usize,
    pub functions: Vec<VCF>,
    pub repository: Repository,
    #[serde(with = "rc_vec_serde")]
    pub witness_list: WitnessList,
    pub templates: Vec<TemplateInstance>,
    pub quick_knowledge: HashMap<String, VCT>,
    pub templates_in_mixed: Vec<usize>,
    pub prime: String,
    pub buses: Vec<BusInstance>,
}

impl VCP {
    pub fn new(config: VCPConfig) -> VCP {
        let mut vcp = VCP {
            stats: config.stats,
            main_id: config.main_id,
            witness_list: Rc::new(Vec::with_capacity(0)),
            repository: config.repository,
            templates: config.templates,
            templates_in_mixed: config.templates_in_mixed,
            functions: vec![],
            quick_knowledge: HashMap::new(),
            prime: config.prime,
            buses: config.buses
        };
        super::merger::run_preprocessing(&mut vcp, config.library);
        vcp
    }
    pub fn add_witness_list(&mut self, witness: Rc<Vec<usize>>) {
        self.witness_list = witness;
    }
    pub fn get_main_instance(&self) -> Option<&TemplateInstance> {
        self.templates.last()
    }
    pub fn get_main_id(&self) -> usize {
        self.main_id
    }
    pub fn get_witness_list(&self) -> &Vec<usize> {
        &self.witness_list
    }
    pub fn get_stats(&self) -> &Stats {
        &self.stats
    }
    pub fn num_templates(&self) -> usize {
        self.templates.len()
    }
}
