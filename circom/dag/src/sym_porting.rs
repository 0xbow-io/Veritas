use super::{Tree, DAG};
use circom_algebra::num_traits::AsPrimitive;
use jsonbb::Builder;
use std::collections::HashMap;

pub fn write(dag: &DAG, buff: &mut Vec<u8>) -> Result<(), ()> {
    let mut builder = Builder::<&mut Vec<u8>>::new(buff);
    builder.begin_object();
    builder.add_string("main");

    let tree = Tree::new(dag);
    builder.begin_object();
    build_from_tree(&tree, &mut builder);
    builder.end_object();
    builder.end_object();
    let _ = builder.finish();
    Ok(())
}

fn build_from_tree(tree: &Tree, builder: &mut Builder<&mut Vec<u8>>) {
    for signal in &tree.signals {
        // signal name, i.e., in, out.
        match HashMap::get(&tree.id_to_name, signal) {
            Some(name) => {
                /*
                    "out": {
                        "witness_id": 0,
                        "witness_value": 0,
                        "type": "signal"
                    }
                */
                builder.add_string(&name);
                builder.begin_object();
                builder.add_string("witness_id");
                builder.add_i64(signal.as_());
                builder.add_string("witness_value");
                builder.add_i64(0);
                builder.add_string("type");
                builder.add_string("signal");
                builder.end_object();
            }
            None => {
                println!("Error: signal name not found");
            }
        }
    }
    for edge in Tree::get_edges(tree) {
        /*
        "c": {
            "out": {
              "type": "signal",
              "witness_id": 6,
              "witness_value": 289
            },
            "id": 6,
            "type": "component"
        }
        */
        builder.add_string(&edge.label);
        builder.begin_object();

        let subtree = Tree::go_to_subtree(tree, edge);

        build_from_tree(&subtree, builder);

        builder.add_string("id");
        builder.add_i64(subtree.node_id.as_());
        builder.add_string("type");
        builder.add_string("component");
        builder.end_object();
    }
}

/*


fn visit_tree(tree: &Tree, buff: &mut Vec<u8>) {
    for signal in &tree.signals {
        let name = HashMap::get(&tree.id_to_name, signal).unwrap();
        let symbol = format!("{}.{}", tree.path, name);
        let original = signal.as_();
        let witness = original;
        let node_id = tree.node_id.as_();
        let sym_elem = SymElem {
            original,
            witness,
            node_id,
            symbol,
        };
        //  format!("{},{},{},{}", self.original, self.witness, self.node_id, self.symbol)
        println!("{}", sym_elem.to_string());
    }
    for edge in Tree::get_edges(tree) {
        let subtree = Tree::go_to_subtree(tree, edge);
        visit_tree(&subtree)?;
    }
}
 */
