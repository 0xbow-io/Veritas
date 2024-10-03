use super::{ConstraintList, EncodingIterator, IteratorSignal, SignalMap};
use circom_algebra::num_traits::AsPrimitive;
use constraint_writers::sym_writer::*;

pub fn port_sym(list: &ConstraintList, buff: &mut Vec<u8>) -> Result<(), ()> {
    let iter = EncodingIterator::new(&list.dag_encoding);
    //let mut dot_sym = SymFile::new(file_name)?;
    signal_iteration(iter, &list.signal_map, buff)?;
    //SymFile::finish_writing(dot_sym)?;
    //SymFile::close(dot_sym);
    Ok(())
}

pub fn signal_iteration(
    mut iter: EncodingIterator,
    map: &SignalMap,
    buff: &mut Vec<u8>,
) -> Result<(), ()> {
    let (signals, _) = EncodingIterator::take(&mut iter);

    for signal in signals {
        let signal = IteratorSignal::new(signal, map);
        let sym_elem = SymElem {
            original: signal.original.as_(),
            witness: if signal.witness == map.len() {
                -1
            } else {
                signal.witness.as_()
            },
            node_id: iter.node_id.as_(),
            symbol: signal.name.clone(),
        };
        println!("{}", sym_elem.to_string());
        //SystemSymFile::write_sym_elem(dot_sym, sym_elem)?;
    }

    for edge in EncodingIterator::edges(&iter) {
        let next = EncodingIterator::next(&iter, edge);
        signal_iteration(next, map, buff)?;
    }
    Ok(())
}
