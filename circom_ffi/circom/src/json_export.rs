pub extern crate num_bigint_dig as num_bigint;
pub extern crate num_traits;
use num_bigint::BigInt;
use jsonbb::Builder;

use constraint_writers::sym_writer::SymElem;
use program_structure::ast::*;

use crate::constraint_system::*;

pub fn produce_constraint_evaluation_json(
    records: &LCRecords,
    constrained: &Vec<&SymElem>,
    unconstrained: &Vec<&SymElem>,
    assignments: &Vec<BigInt>,
) -> String {
    let mut builder = Builder::<Vec<u8>>::new();
    builder.begin_object();
    builder.add_string("field");
    builder.add_string(&records[0].field.to_string());

    // create json objects for symbols
    builder.add_string("symbols");
    builder.begin_object();

    builder.add_string("constrained");
    builder.begin_array();
    for sym in constrained {
        builder.begin_object();
        builder.add_string("original");
        builder.add_string(&sym.original.to_string());
        builder.add_string("witness");
        builder.add_string(&sym.witness.to_string());
        builder.add_string("node_id");
        builder.add_string(&sym.node_id.to_string());
        builder.add_string("symbol");
        builder.add_string(&sym.symbol);
        builder.end_object();
    }
    builder.end_array();

    builder.add_string("unconstrained");
    builder.begin_array();
    for sym in unconstrained {
        builder.begin_object();
        builder.add_string("original");
        builder.add_string(&sym.original.to_string());
        builder.add_string("witness");
        builder.add_string(&sym.witness.to_string());
        builder.add_string("node_id");
        builder.add_string(&sym.node_id.to_string());
        builder.add_string("symbol");
        builder.add_string(&sym.symbol);
        builder.end_object();
    }
    builder.end_array();

    builder.end_object();

    builder.add_string("assignments");
    builder.begin_array();
    for assignment in assignments {
        builder.add_string(&assignment.to_string());
    }
    builder.end_array();

    builder.add_string("constraints");
    builder.begin_array();
    for r in records {
        builder.begin_object();

        builder.add_string("a_constraints");
        builder.begin_array();
        for c in &r.a_constraints {
            builder.begin_array();
            builder.add_string(&c.0.to_string());
            builder.add_string(&c.1.to_string());
            builder.end_array();
        }
        builder.end_array();
        builder.add_string("b_constraints");
        builder.begin_array();
        for c in &r.b_constraints {
            builder.begin_array();
            builder.add_string(&c.0.to_string());
            builder.add_string(&c.1.to_string());
            builder.end_array();
        }
        builder.end_array();
        builder.add_string("c_constraints");
        builder.begin_array();
        for c in &r.c_constraints {
            builder.begin_array();
            builder.add_string(&c.0.to_string());
            builder.add_string(&c.1.to_string());
            builder.end_array();
        }
        builder.end_array();

        builder.add_string("arithmetization");
        builder.begin_array();

        builder.add_string(&r.arith.0.to_string());
        builder.add_string(&r.arith.1.to_string());
        builder.add_string(&r.arith.2.to_string());
        builder.add_string(&r.arith.3.to_string());

        builder.end_array();
        builder.add_string("satisfied");
        match &r.report {
            Some(r) => {
                builder.add_string("false");
            }
            None => {
                builder.add_string("true");
            }
        }
        builder.end_object();
    }
    builder.end_array();
    builder.end_object();

    let json = builder.finish();
    json.to_string()
}

pub fn statement_json_builder(st: &Statement, builder: &mut Builder<Vec<u8>>) {
    match st {
        Statement::IfThenElse { meta, cond, if_case, else_case } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("IfThenElse");
            builder.add_string("condition");
            expression_json_builder(&cond, builder);
            builder.add_string("if_case");
            statement_json_builder(&if_case, builder);
            builder.add_string("else_case");
            match else_case {
                Some(else_case) => {
                    statement_json_builder(&else_case, builder);
                }
                None => {
                    builder.add_null();
                }
            }
            builder.end_object();
        }
        Statement::While { meta, cond, stmt } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("While");
            builder.add_string("condition");
            expression_json_builder(&cond, builder);
            builder.add_string("body");
            statement_json_builder(&stmt, builder);
            builder.end_object();
        }
        Statement::Return { meta, value } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("Return");
            builder.add_string("value");
            expression_json_builder(&value, builder);
            builder.end_object();
        }
        Statement::InitializationBlock { meta, xtype, initializations } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("InitializationBlock");
            builder.add_string("type");
            variable_type_json(&xtype, builder);
            builder.add_string("initializations");
            builder.begin_array();
            for stmt in initializations {
                statement_json_builder(stmt, builder);
            }
            builder.end_array();
            builder.end_object();
        }
        Statement::Declaration { meta, xtype, name, dimensions, is_constant } => {
            builder.begin_object();

            builder.add_string("statement");
            builder.add_string("Declaration");

            builder.add_string("type");
            variable_type_json(&xtype, builder);

            builder.add_string("name");
            builder.add_string(name);

            builder.add_string("dimensions");

            builder.begin_array();
            for dim in dimensions {
                expression_json_builder(&dim, builder);
            }
            builder.end_array();

            builder.add_string("is_constant");
            builder.add_bool(*is_constant);

            builder.end_object();
        }
        Statement::Substitution { meta, var, access, op, rhe } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("Substitution");
            builder.add_string("variable");
            builder.add_string(var);
            builder.add_string("access");
            builder.begin_array();
            for acc in access {
                access_json(acc, builder);
            }
            builder.end_array();
            builder.add_string("operator");
            builder.add_string(&op.to_string());
            builder.add_string("rhs");
            expression_json_builder(rhe, builder);
            builder.end_object();
        }
        Statement::MultSubstitution { meta, lhe, op, rhe } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("MultSubstitution");
            builder.add_string("lhs");
            expression_json_builder(lhe, builder);
            builder.add_string("operator");
            builder.add_string(&op.to_string());
            builder.add_string("rhs");
            expression_json_builder(rhe, builder);
            builder.end_object();
        }
        Statement::UnderscoreSubstitution { meta, op, rhe } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("UnderscoreSubstitution");
            builder.add_string("operator");
            builder.add_string(&op.to_string());
            builder.add_string("rhs");
            expression_json_builder(rhe, builder);
            builder.end_object();
        }
        Statement::ConstraintEquality { meta, lhe, rhe } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("ConstraintEquality");
            builder.add_string("lhs");
            expression_json_builder(lhe, builder);
            builder.add_string("rhs");
            expression_json_builder(rhe, builder);
            builder.end_object();
        }
        Statement::LogCall { meta, args } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("LogCall");
            builder.add_string("args");
            builder.begin_array();
            for arg in args {
                log_json(arg, builder);
            }
            builder.end_array();
            builder.end_object();
        }
        Statement::Block { meta, stmts } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("Block");
            builder.add_string("statements");
            builder.begin_array();
            for stmt in stmts {
                statement_json_builder(stmt, builder);
            }
            builder.end_array();
            builder.end_object();
        }
        Statement::Assert { meta, arg } => {
            builder.begin_object();
            builder.add_string("statement");
            builder.add_string("Assert");
            builder.add_string("arg");
            expression_json_builder(arg, builder);
            builder.end_object();
        }
    }
}

pub fn infix_op_string(op: &ExpressionInfixOpcode) -> String {
    match op {
        ExpressionInfixOpcode::Add => "+".to_string(),
        ExpressionInfixOpcode::Sub => "-".to_string(),
        ExpressionInfixOpcode::Mul => "*".to_string(),
        ExpressionInfixOpcode::Div => "/".to_string(),
        ExpressionInfixOpcode::Mod => "%".to_string(),
        ExpressionInfixOpcode::IntDiv => "//".to_string(),
        ExpressionInfixOpcode::Pow => "**".to_string(),
        ExpressionInfixOpcode::ShiftL => "<<".to_string(),
        ExpressionInfixOpcode::ShiftR => ">>".to_string(),
        ExpressionInfixOpcode::BitAnd => "&".to_string(),
        ExpressionInfixOpcode::BitOr => "|".to_string(),
        ExpressionInfixOpcode::BitXor => "^".to_string(),
        ExpressionInfixOpcode::BoolAnd => "&&".to_string(),
        ExpressionInfixOpcode::BoolOr => "||".to_string(),
        ExpressionInfixOpcode::Lesser => "<".to_string(),
        ExpressionInfixOpcode::LesserEq => "<=".to_string(),
        ExpressionInfixOpcode::Greater => ">".to_string(),
        ExpressionInfixOpcode::GreaterEq => ">=".to_string(),
        ExpressionInfixOpcode::Eq => "==".to_string(),
        ExpressionInfixOpcode::NotEq => "!=".to_string(),
    }
}

pub fn expression_json_builder(exp: &Expression, builder: &mut Builder<Vec<u8>>) {
    match exp {
        Expression::UniformArray { meta, value, dimension } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("UniformArray");
            builder.add_string("value");
            expression_json_builder(value, builder);
            builder.add_string("dimension");
            expression_json_builder(dimension, builder);
            builder.end_object();
        }
        Expression::Tuple { meta, values } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("Tuple");
            builder.add_string("values");
            builder.begin_array();
            for v in values {
                expression_json_builder(v, builder);
            }
            builder.end_array();
            builder.end_object();
        }
        Expression::ArrayInLine { meta, values } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("ArrayInLine");
            builder.add_string("values");
            builder.begin_array();
            for v in values {
                expression_json_builder(v, builder);
            }
            builder.end_array();
            builder.end_object();
        }
        Expression::InfixOp { lhe, infix_op, rhe, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("InfixOp");
            builder.add_string("lhs");
            expression_json_builder(lhe, builder);
            builder.add_string("operator");
            let op = infix_op_string(&infix_op);
            builder.add_string(op.as_str());
            builder.add_string("rhs");
            expression_json_builder(rhe, builder);
            builder.end_object();
        }
        Expression::PrefixOp { prefix_op, rhe, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("PrefixOp");
            builder.add_string("operator");

            let operator = match prefix_op {
                ExpressionPrefixOpcode::Complement => '~',
                ExpressionPrefixOpcode::BoolNot => '!',
                ExpressionPrefixOpcode::Sub => '-',
            };
            builder.add_string(operator.to_string().as_str());
            builder.add_string("rhs");
            expression_json_builder(rhe, builder);
            builder.end_object();
        }
        Expression::InlineSwitchOp { cond, if_true, if_false, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("InlineSwitchOp");
            builder.add_string("condition");
            expression_json_builder(cond, builder);
            builder.add_string("if_true");
            expression_json_builder(if_true, builder);
            builder.add_string("if_false");
            expression_json_builder(if_false, builder);
            builder.end_object();
        }
        Expression::ParallelOp { rhe, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("ParallelOp");
            builder.add_string("rhs");
            expression_json_builder(rhe, builder);
            builder.end_object();
        }
        Expression::Variable { name, access, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("Variable");
            builder.add_string("name");
            builder.add_string(name);
            builder.add_string("access");
            builder.begin_array();
            for a in access {
                match a {
                    Access::ComponentAccess(s) => {
                        builder.add_string(&format!(".{}", s));
                    }
                    Access::ArrayAccess(e) => {
                        expression_json_builder(e, builder);
                    }
                }
            }
            builder.end_array();
            builder.end_object();
        }
        Expression::Number(_, n) => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("Number");
            builder.add_string("value");
            builder.add_string(&n.to_string());
            builder.end_object();
        }
        Expression::Call { id, args, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("Call");
            builder.add_string("id");
            builder.add_string(id);
            builder.add_string("args");
            builder.begin_array();
            for a in args {
                expression_json_builder(a, builder);
            }
            builder.end_array();
            builder.end_object();
        }
        Expression::BusCall { id, args, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("BusCall");
            builder.add_string("id");
            builder.add_string(id);
            builder.add_string("args");
            builder.begin_array();
            for a in args {
                expression_json_builder(a, builder);
            }
            builder.end_array();
            builder.end_object();
        }
        Expression::AnonymousComp { id, params, signals, names, .. } => {
            builder.begin_object();
            builder.add_string("expression");
            builder.add_string("AnonymousComp");
            builder.add_string("id");
            builder.add_string(id);
            builder.add_string("params");
            builder.begin_array();
            for p in params {
                expression_json_builder(p, builder);
            }
            builder.end_array();
            builder.add_string("signals");
            builder.begin_array();
            for s in signals {
                expression_json_builder(s, builder);
            }
            builder.end_array();
            builder.add_string("names");
            builder.begin_array();
            match names {
                Some(n) => {
                    for (op, name) in n.iter() {
                        builder.begin_object();
                        builder.add_string(op.to_string().as_str());
                        builder.add_string(name);
                        builder.end_object();
                    }
                }
                None => {}
            }
            builder.end_array();
            builder.end_object();
        }
    }
}

pub fn log_json(l: &LogArgument, builder: &mut Builder<Vec<u8>>) {
    builder.begin_object();
    builder.add_string("log");
    match l {
        LogArgument::LogStr(s) => {
            builder.add_string(s);
        }
        LogArgument::LogExp(e) => {
            expression_json_builder(e, builder);
        }
    }
    builder.end_object();
}

pub fn signal_type_string(l: &SignalType) -> String {
    match l {
        SignalType::Output => "output",
        SignalType::Input => "input",
        SignalType::Intermediate => "intermediate",
    }
    .to_string()
}

pub fn access_json(l: &Access, builder: &mut Builder<Vec<u8>>) {
    builder.begin_object();
    builder.add_string("access");
    match l {
        Access::ComponentAccess(s) => {
            builder.add_string(s);
        }
        Access::ArrayAccess(e) => {
            expression_json_builder(e, builder);
        }
    }
    builder.end_object();
}

pub fn variable_type_json(v: &VariableType, builder: &mut Builder<Vec<u8>>) {
    match v {
        VariableType::Var => {
            builder.begin_object();
            builder.add_string("variable_type");
            builder.add_string("variable");
            builder.end_object();
        }
        VariableType::Signal(sig_type, tags) => {
            builder.begin_object();
            builder.add_string("variable_type");
            builder.add_string("signal");
            builder.add_string("signal_type");
            builder.add_string(signal_type_string(sig_type).as_str());
            builder.add_string("tags");
            builder.begin_array();
            for t in tags {
                builder.add_string(t);
            }
            builder.end_array();
            builder.end_object();
        }
        VariableType::Component => {
            builder.begin_object();
            builder.add_string("variable_type");
            builder.add_string("component");
            builder.end_object();
        }
        VariableType::AnonymousComponent => {
            builder.begin_object();
            builder.add_string("variable_type");
            builder.add_string("anonymous_component");
            builder.end_object();
        }
        VariableType::Bus(name, sig_type, tags) => {
            builder.begin_object();
            builder.add_string("variable_type");
            builder.add_string("bus");
            builder.add_string("name");
            builder.add_string(name);
            builder.add_string("signal_type");
            builder.add_string(signal_type_string(sig_type).as_str());
            builder.add_string("tags");
            builder.begin_array();
            for t in tags {
                builder.add_string(t);
            }
            builder.end_array();
            builder.end_object();
        }
    }
}
