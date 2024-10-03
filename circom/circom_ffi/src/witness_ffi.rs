use crate::witness;
use ansi_term::Colour;
use compiler::compiler_interface::{Circuit, VCP};
use dag::DAG;
use program_structure::error_definition::Report;
use std::ffi::{c_char, c_void, CStr, CString};
use std::io::BufWriter;

// TODO:
// use std::panic::catch_unwind;
use tokio::runtime::Runtime;

#[repr(C)]
pub struct FFIWitnessCalculator {
    inner: *mut c_void,
}

extern "C" {
    fn VeritasIncludeWC(ctx_handle: usize, wc_ptr: *const FFIWitnessCalculator);
    fn VeritasIncludeWitness(ctx_handle: usize, witness_json: *const c_char);
}

fn report_wintess(ctx_handle: usize, wit_json: &str) {
    let wit_json_cstring = CString::new(wit_json).unwrap();
    unsafe {
        VeritasIncludeWitness(ctx_handle, wit_json_cstring.as_ptr());
    };
}

#[no_mangle]
pub extern "C" fn ffi_generate_witness_calculator(
    ctx_handle: usize,
    ffi_circuit: *mut crate::compile_ffi::FFICircomCircuit,
) {
    let ffi_circuit = unsafe { &mut *ffi_circuit };
    let (_, _, circuit) = unsafe { &mut *(ffi_circuit.inner() as *mut (DAG, VCP, Circuit)) };

    let mut wasm_buff = Vec::new();
    let _ = circuit.gen_wasm_bin(&mut BufWriter::new(&mut wasm_buff));

    let mut buff = Vec::with_capacity(100_000);
    let runtime = Runtime::new().unwrap();

    let witness_calculator = match runtime.block_on(async {
        let mut witness_calculator = witness::WitnessCalculator::default();
        match witness_calculator.load(&wasm_buff) {
            Ok(_) => {}
            Err(report) => {
                return Err(report);
            }
        }
        Ok::<_, Report>(witness_calculator)
    }) {
        Ok(wc) => wc,
        Err(report) => {
            return crate::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff)
        }
    };

    let ffi_wc = Box::new(FFIWitnessCalculator {
        inner: Box::into_raw(Box::new((witness_calculator, runtime))) as *mut c_void,
    });

    unsafe {
        VeritasIncludeWC(ctx_handle, Box::into_raw(ffi_wc));
    }

    println!(
        "ffi_generate_witness_calculator {} ...",
        Colour::Green.paint("Successful"),
    );
}

#[no_mangle]
pub extern "C" fn ffi_calculate_witness(
    ctx_handle: usize,
    ffi_circuit: *mut crate::compile_ffi::FFICircomCircuit,
    ffi_wc: *mut FFIWitnessCalculator,
    inputs_json: *const c_char,
) {
    let ffi_wc = unsafe { &mut *ffi_wc };
    let ffi_circuit = unsafe { &mut *ffi_circuit };
    let mut buff = Vec::with_capacity(100_000);

    let inputs_json_str = unsafe { CStr::from_ptr(inputs_json) }.to_str().unwrap();
    let circuit_inputs = witness::parse_inputs(inputs_json_str);

    let (witness_calculator, runtime) =
        unsafe { &mut *(ffi_wc.inner as *mut (witness::WitnessCalculator, Runtime)) };

    let (dag, _, _) = unsafe { &mut *(ffi_circuit.inner() as *mut (DAG, VCP, Circuit)) };

    let result = runtime.block_on(async { witness_calculator.calculate_witness(circuit_inputs) });

    match result {
        Ok(witness) => {
            let wit_json = crate::json_export::build_json_output(&dag, &witness);
            report_wintess(ctx_handle, &wit_json);
        }
        Err(report) => crate::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff),
    }
    println!(
        "ffi_calculate_witness {} ...",
        Colour::Green.paint("Successful"),
    );
}

#[no_mangle]
pub extern "C" fn free_witness_calculator(ffi_wc: *mut FFIWitnessCalculator) {
    if !ffi_wc.is_null() {
        unsafe {
            let _ = Box::from_raw(ffi_wc);
        }
    }
}
