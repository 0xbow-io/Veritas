use code_producers::wasm_elements::wasm_code_generator::*;
use compiler::compiler_interface::Circuit;
use compiler::translating_traits::*;
use wasmer::wat2wasm;
use program_structure::{error_code::ReportCode, error_definition::Report};

pub fn generate_wat_instructions(circuit: &Circuit) -> Vec<String> {
    let mut code = vec![];
    code.push("(module".to_string());
    let mut code_aux = generate_imports_list();
    code.append(&mut code_aux);
    code_aux = generate_memory_def_list(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = fr_types(&circuit.wasm_producer.prime_str);
    code.append(&mut code_aux);

    code_aux = generate_types_list();
    code.append(&mut code_aux);
    code_aux = generate_exports_list();
    code.append(&mut code_aux);

    code_aux = fr_code(&circuit.wasm_producer.prime_str);
    code.append(&mut code_aux);

    code_aux = desp_io_subcomponent_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_version_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_shared_rw_memory_start_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = read_shared_rw_memory_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = write_shared_rw_memory_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = reserve_stack_fr_function_generator();
    code.append(&mut code_aux);

    code_aux = init_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = set_input_signal_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_input_signal_size_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_raw_prime_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_field_num_len32_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_input_size_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_witness_size_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_witness_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = copy_32_in_shared_rw_memory_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = copy_fr_in_shared_rw_memory_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = get_message_char_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = build_buffer_message_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = build_log_message_generator(&circuit.wasm_producer);
    code.append(&mut code_aux);

    for f in &circuit.functions {
        code.append(&mut f.produce_wasm(&circuit.wasm_producer))
    }

    for t in &circuit.templates {
        code.append(&mut t.produce_wasm(&circuit.wasm_producer))
    }

    code_aux = generate_table_of_template_runs(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code_aux = fr_data(&circuit.wasm_producer.prime_str);
    code.append(&mut code_aux);

    code_aux = generate_data_list(&circuit.wasm_producer);
    code.append(&mut code_aux);

    code.push(")".to_string());
    code
}

pub fn generate_circuit_wasm(circuit: &Circuit) -> Result<Vec<u8>, Report> {
    let instructions = generate_wat_instructions(circuit);
    let wat = format!("{}\n", instructions.join("\n"));
    //print!("Wasm code: {}", wat);

    match wat2wasm(&wat.as_bytes()) {
        Ok(wasm) => Ok(wasm.into_owned()),
        Err(e) => {
            let err = Report::error(e.to_string(), ReportCode::RuntimeError);
            Err(err)
        }
    }
}
