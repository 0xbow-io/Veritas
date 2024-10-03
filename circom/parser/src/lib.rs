extern crate indoc;
extern crate num_bigint_dig as num_bigint;
extern crate num_traits;
extern crate serde;
extern crate serde_derive;
//mod include_logic;
mod parser_logic;
mod syntax_sugar_remover;

pub mod common;
pub mod parser;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub lang);
