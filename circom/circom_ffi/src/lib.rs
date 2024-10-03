pub mod compile;
pub mod compile_ffi;
pub mod json_export;
pub mod program_archive_ffi;
pub mod witness;
pub mod witness_ffi;

use codespan_reporting::diagnostic::Diagnostic;
use program_structure::program_archive::ProgramID;
use serde::Deserialize;

use std::ffi::{c_char, c_void, CString};

extern "C" {
    fn VeritasIncludeError(ctx_handle: usize, msg: *const c_char);
    fn VeritasIncludeDiagnostic(ctx_handle: usize, diagnostic_json: *const c_void, len: usize);
}

#[derive(Deserialize)]
pub(crate) struct RuntimeCtx {
    pub version: String,
    pub prime: String,
    pub src: Vec<(String, String)>,
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// freeString is a helper function to free the memory allocated by the C code.
/// This function is called in GO code to free the memory allocated by the Rust code.
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

fn report_diagnostic(ctx_handle: usize, diagnostic: &Diagnostic<ProgramID>, buff: &mut Vec<u8>) {
    buff.clear();
    serde_json::to_writer(&mut *buff, diagnostic).unwrap();

    let ptr = buff.as_ptr();
    let len = buff.len();

    unsafe {
        VeritasIncludeDiagnostic(ctx_handle, ptr as *const c_void, len);
    };
}

fn report_error(ctx_handle: usize, msg: &str) {
    let err_msg = CString::new(msg).unwrap();
    unsafe {
        VeritasIncludeError(ctx_handle, err_msg.as_ptr());
    };
}

/*
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ffi_compile_circom(ctx_handle: usize, inputs_json: *const c_char) {
    let mut buff = Vec::with_capacity(100_000);
    let inputs_json_str = unsafe { CStr::from_ptr(inputs_json) }.to_str().unwrap();
    let inputs: Result<Inputs, serde_json::Error> = serde_json::from_str(inputs_json_str);
    match inputs {
        Ok(inputs) => match compile::preprocess_circom_circuit(
            String::from(inputs.version.as_str()),
            inputs.prime,
            inputs.sources,
        ) {
            Ok((dag, vcp)) => {
                report_dag(ctx_handle, &dag, &mut buff);
                report_vcp(ctx_handle, &vcp, &mut buff);

                let circuit = compile::build_circom_circuit(vcp, inputs.version);
                match circuit {
                    Ok(circuit) => {
                        let mut wasm_buff = Vec::new();
                        compile::generate_wasm_bin(&circuit, &mut wasm_buff);
                        let wasm_ptr = wasm_buff.as_ptr();
                        let wasm_len = wasm_buff.len();
                        unsafe {
                            VeritasIncludeWASM(ctx_handle, wasm_ptr as *const c_void, wasm_len);
                        }
                        wasm_buff.clear();
                    }
                    Err(_) => {
                        report_error(ctx_handle, "Failed to build circuit");
                    }
                }
            }
            Err(reports) => {
                for report in reports.iter() {
                    report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff);
                }
            }
        },
        Err(e) => {
            report_error(ctx_handle, e.to_string().as_str());
        }
    }
}
 */
