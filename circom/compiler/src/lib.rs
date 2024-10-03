#[allow(dead_code)]
pub extern crate num_bigint_dig as num_bigint;
pub extern crate num_traits;
mod translating_traits;

pub mod circuit_design;
pub mod compiler_interface;
pub mod hir;
pub mod intermediate_representation;
pub mod ir_processing;
