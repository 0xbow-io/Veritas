pub extern crate num_bigint_dig as num_bigint;
pub extern crate num_traits;

use program_structure::{error_code::ReportCode, error_definition::Report};
use circom_algebra::modular_arithmetic::*;
use constraint_writers::sym_writer::SymElem;
use num_bigint::BigInt;
use num::ToPrimitive;

use std::collections::HashMap;
use constraint_list::{EncodingIterator, IteratorSignal, ConstraintList};

pub type C = circom_algebra::algebra::Constraint<usize>;
pub type A = circom_algebra::algebra::ArithmeticExpression<usize>;
type LC<C> = HashMap<C, BigInt>;

pub struct ConstraintSystem {
    pub field: BigInt,
    pub no_labels: usize,
    pub no_wires: usize,
    pub no_private_inputs: usize,
    pub no_private_inputs_witness: usize,
    pub no_public_inputs: usize,
    pub no_public_outputs: usize,

    pub no_linear: usize,
    pub no_non_linear: usize,
    pub num_constraints: usize,

    pub symbols: Vec<SymElem>,
    pub constraints: Vec<C>,
}

impl Default for ConstraintSystem {
    fn default() -> Self {
        ConstraintSystem {
            field: BigInt::from(0),
            no_labels: 0,
            no_wires: 0,
            no_private_inputs: 0,
            no_private_inputs_witness: 0,
            no_public_inputs: 0,
            no_public_outputs: 0,
            no_linear: 0,
            no_non_linear: 0,
            num_constraints: 0,

            symbols: Vec::new(),
            constraints: Vec::new(),
        }
    }
}
impl ConstraintSystem {
    // sync with the generated constraints list
    pub fn sync(&mut self, list: &ConstraintList) {
        self.field = list.field.clone();
        self.no_labels = ConstraintList::no_labels(list);
        self.no_wires = ConstraintList::no_wires(list);
        self.no_private_inputs = list.no_private_inputs;
        self.no_private_inputs_witness = list.no_private_inputs_witness;
        self.no_public_inputs = list.no_public_inputs;
        self.no_public_outputs = list.no_public_outputs;

        self.sync_signals(list);

        let cids = list.constraints.get_ids();
        self.num_constraints = cids.len();
        for c_id in cids {
            let c = list.constraints.read_constraint(c_id).unwrap();
            let c = C::apply_correspondence(&c, &list.signal_map);
            if C::is_linear(&c) {
                self.no_linear += 1;
            } else {
                self.no_non_linear += 1;
            }
            self.constraints.push(c);
        }
    }

    fn sync_signals(&mut self, list: &ConstraintList) {
        pub fn signal_iteration(
            mut iter: EncodingIterator,
            list: &ConstraintList,
            sym: &mut Vec<SymElem>,
        ) {
            // ommit the constraints from the DAG encoding
            // refer to the constraints from the constraint list
            let (signals, _) = EncodingIterator::take(&mut iter);

            for siginfo in signals {
                let signal = IteratorSignal::new(siginfo, &list.signal_map);
                let sym_elem = SymElem {
                    original: signal.original.to_i64().unwrap(),
                    witness: if signal.witness == list.signal_map.len() {
                        -1
                    } else {
                        signal.witness.to_i64().unwrap()
                    },
                    node_id: iter.node_id.to_i64().unwrap(),
                    symbol: signal.name.clone(),
                };
                sym.push(sym_elem);
            }

            for edge in EncodingIterator::edges(&iter) {
                let next = EncodingIterator::next(&iter, edge);
                signal_iteration(next, list, sym);
            }
        }

        let iter = EncodingIterator::new(&list.dag_encoding);
        signal_iteration(iter, list, &mut self.symbols);
    }

    pub fn eval_constraints(&mut self, assignments: &Vec<BigInt>) -> LCRecords {
        let mut records = Vec::new();
        for c in &self.constraints {
            records.push(LCRecord::new(assignments, c.a(), c.b(), c.c(), &self.field));
        }
        records
    }

    pub fn signals(&self) -> (Vec<&SymElem>, Vec<&SymElem>) {
        let mut mapped_signals = Vec::new();
        let mut unmapped_signals = Vec::new();
        let mut mapping = vec![0; self.no_wires];
        for (i, s) in self.symbols.iter().enumerate() {
            if s.witness == -1 {
                unmapped_signals.push(s);
            } else {
                mapping[s.witness as usize] = i;
            }
        }
        for i in 0..self.no_wires {
            mapped_signals.push(&self.symbols[mapping[i]]);
        }
        (mapped_signals, unmapped_signals)
    }
}

pub type LCRecords = Vec<LCRecord>;
pub struct LCRecord {
    pub field: BigInt,
    pub a_constraints: Vec<(usize, BigInt)>,
    pub b_constraints: Vec<(usize, BigInt)>,
    pub c_constraints: Vec<(usize, BigInt)>,
    pub arith: (BigInt, BigInt, BigInt, BigInt),
    pub report: Option<Report>,
}

fn val(elem: &BigInt, field: &BigInt) -> BigInt {
    let c = (field / &BigInt::from(2)) + 1;
    if &c <= elem && elem < field {
        elem - field
    } else {
        elem.clone()
    }
}

impl LCRecord {
    pub fn new(
        assignments: &Vec<BigInt>,
        a_constraints: &LC<usize>,
        b_constraints: &LC<usize>,
        c_constraints: &LC<usize>,
        field: &BigInt,
    ) -> Self {
        let mut a = BigInt::from(0);
        let mut b = BigInt::from(0);
        let mut c = BigInt::from(0);

        for (k, v) in a_constraints.iter() {
            let ass = assignments.get(*k).unwrap();
            a += mul(v, &ass, &field);
        }
        for (k, v) in b_constraints.iter() {
            let ass = assignments.get(*k).unwrap();
            b += mul(v, &ass, &field);
        }
        for (k, v) in c_constraints.iter() {
            let ass = assignments.get(*k).unwrap();
            c += mul(v, &ass, &field);
        }
        let x = mul(&a, &b, &field);
        let y = sub(&x, &c, &field);
        let is_satisfied = y == BigInt::from(0);

        let arith = (val(&a, field), val(&b, field), val(&c, field), val(&y, field));
        if !is_satisfied {
            let warn_msg = format!("Constraint is not satisfied");
            let mut r = Report::error(warn_msg, ReportCode::IllegalExpression);
            let note = format!(
                " Unsatisfied Constraint => {} * {} - {} != 0 got {} instead",
                arith.0, arith.1, arith.2, arith.3
            );
            r.add_note(note);
            Self {
                field: field.clone(),
                a_constraints: a_constraints.iter().map(|(k, v)| (*k, v.clone())).collect(),
                b_constraints: b_constraints.iter().map(|(k, v)| (*k, v.clone())).collect(),
                c_constraints: c_constraints.iter().map(|(k, v)| (*k, v.clone())).collect(),
                arith,
                report: Some(r),
            }
        } else {
            Self {
                field: field.clone(),
                a_constraints: a_constraints.iter().map(|(k, v)| (*k, v.clone())).collect(),
                b_constraints: b_constraints.iter().map(|(k, v)| (*k, v.clone())).collect(),
                c_constraints: c_constraints.iter().map(|(k, v)| (*k, v.clone())).collect(),
                arith,
                report: None,
            }
        }
    }

    pub fn linear_string(&self, symbols: &Vec<&SymElem>, assignments: &Vec<BigInt>) -> String {
        let linear_a = self.a_constraints.iter().fold(String::new(), |acc, (k, v)| {
            format!(
                "{} + ({} * {})[{}] ",
                acc,
                val(v, &self.field),
                val(&assignments[*k], &self.field),
                symbols[*k].symbol
            )
        });
        let linear_b = self.b_constraints.iter().fold(String::new(), |acc, (k, v)| {
            format!(
                "{} + ({} * {})[{}] ",
                acc,
                val(v, &self.field),
                val(&assignments[*k], &self.field),
                symbols[*k].symbol
            )
        });
        let linear_c = self.c_constraints.iter().fold(String::new(), |acc, (k, v)| {
            format!(
                "{} + ({} * {})[{}] ",
                acc,
                val(v, &self.field),
                val(&assignments[*k], &self.field),
                symbols[*k].symbol
            )
        });

        format!(
            "\nA: {linear_a} = {}\nB: {linear_b} = {}\nC: {linear_c} = {}\n",
            self.arith.0, self.arith.1, self.arith.2
        )
    }
}
