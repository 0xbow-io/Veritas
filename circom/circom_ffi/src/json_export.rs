use circom_algebra::num_traits::AsPrimitive;
use dag::{Tree, DAG};
use jsonbb::Builder;
use num_bigint::BigInt;
use std::collections::HashMap;

/*
{
  "main": {
    "constraints": [
      [
        {

        },
        {

        },
        {
          "0": "1",
          "1": "21888242871839275222246405745257275088548364400416034343698204186575808495616",
          "6": "1"
        }
      ],

    ],
    "signals": [
      {
        "id": 1,
        "name": "out"
      },

    ],
    "witness": [
      {
        "id": 2,
        "value": "1"
      },

    ]
  }
}
*/

pub fn build_json_output(dag: &DAG, witness: &Vec<BigInt>) -> String {
    let mut builder = Builder::default();

    builder.begin_object();

    let tree = Tree::new(dag);
    builder.add_string("main");
    builder.begin_object();
    map_components(&tree, witness, &mut builder);
    builder.end_object();
    let value = builder.finish();
    value.to_string()
}

fn map_components(tree: &Tree, witness: &Vec<BigInt>, builder: &mut Builder<Vec<u8>>) {
    let mut wits: HashMap<usize, BigInt> = HashMap::new();

    builder.add_string("signals");
    builder.begin_array();
    for signal in &tree.signals {
        match HashMap::get(&tree.id_to_name, signal) {
            Some(name) => {
                /*
                "witness": {
                    "in": {
                        "id": 0,
                        "value": 10
                    }
                },
                */
                wits.insert(*signal, witness[*signal].clone());

                builder.begin_object();
                builder.add_string("name");
                builder.add_string(&name);
                builder.add_string("id");
                builder.add_i64(signal.as_());
                builder.end_object();
            }
            None => {
                println!("Error: signal name not found");
            }
        }
    }
    builder.end_array();

    builder.add_string("witness");
    builder.begin_array();
    for (key, value) in wits {
        builder.begin_object();
        builder.add_string("id");
        builder.add_i64(key.as_());
        builder.add_string("value");
        builder.add_string(&value.to_str_radix(10));
        builder.end_object();
    }
    builder.end_array();

    map_constraints(tree, builder);

    builder.end_object();

    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);

        builder.add_string(edge.get_label());
        builder.begin_object();
        builder.add_string("id");
        builder.add_i64(subtree.node_id.as_());

        map_components(&subtree, witness, builder);
    }
}

fn map_constraints(tree: &Tree, builder: &mut Builder<Vec<u8>>) {
    builder.add_string("constraints");
    builder.begin_array();
    for c in &tree.constraints {
        builder.begin_array();
        for i in 0..3 {
            let constraint_map = match i {
                0 => c.a(),
                1 => c.b(),
                2 => c.c(),
                _ => panic!("Invalid constraint index"),
            };
            let mut order: Vec<&usize> = constraint_map.keys().collect();
            order.sort();
            builder.begin_object();
            for i in order {
                match constraint_map.get_key_value(i) {
                    Some((key, value)) => {
                        builder.add_string(&key.to_string());
                        builder.add_string(&value.to_str_radix(10));
                    }
                    None => {
                        panic!("Error: constraint key not found")
                    }
                }
            }
            builder.end_object();
        }
        builder.end_array();
    }
    builder.end_array();
}
