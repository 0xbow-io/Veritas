use std::{
    ffi::{c_char, c_void, CStr, CString},
};
use crate::circuit::CircuitLibrary;
use super::reporting::report_error;

#[repr(C)]
pub struct FFICircom {
    inner: *mut c_void,
}

impl FFICircom {
    pub fn inner(&self) -> *mut c_void {
        self.inner
    }
}

extern "C" {
    fn share_evaluations(ctx_handle: usize, ceval_json: *const c_void, len: usize);
    fn share_report(ctx_handle: usize, report: *const c_void, len: usize);
    fn share_circom_ptr(ctx_handle: usize, ptr: *const FFICircom);
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ffi_compile_library(ctx_handle: usize, pkg_json_raw: *const c_char) {
    let mut buff = Vec::with_capacity(100_000);

    // Deserialize the JSON string into a CircuitPkg struct
    let pkg_json_str = unsafe { CStr::from_ptr(pkg_json_raw) }.to_str().unwrap();
    let pkg_json: Result<crate::circuit::CircuitPkg, serde_json::Error> =
        serde_json::from_str(pkg_json_str);

    let circuit_pkg = match pkg_json {
        Ok(circuit_pkg) => circuit_pkg,
        Err(e) => {
            report_error(ctx_handle, &e.to_string(), &mut buff);
            return;
        }
    };

    let mut library = crate::circuit::CircuitLibrary::default();
    match library.compile(&circuit_pkg) {
        Ok(warnings) => {
            for w in warnings.iter() {
                crate::reporting::report_diagnostic(ctx_handle, &w.to_diagnostic(), &mut buff);
            }
        }
        Err(v) => {
            for r in v.iter() {
                crate::reporting::report_diagnostic(ctx_handle, &r.to_diagnostic(), &mut buff);
            }
        }
    }
    // TODO: Optimize this
    let ffi_lib = Box::new(FFICircom { inner: Box::into_raw(Box::new(library)) as *mut c_void });
    unsafe {
        share_circom_ptr(ctx_handle, Box::into_raw(ffi_lib));
    }
}

#[no_mangle]
pub extern "C" fn ffi_circuit_execution(
    ctx_handle: usize,
    ffi_circom: *mut FFICircom,
    inputs_json: *const c_char,
) {
    pub fn ffi_pass_evals(ctx_handle: usize, ptr: *const u8, len: usize) {
        unsafe {
            share_evaluations(ctx_handle, ptr as *const c_void, len);
        };
    }
    let ffi_circom = unsafe { &mut *ffi_circom };
    let mut buff = Vec::with_capacity(100_000);

    let inputs_json_str = unsafe { CStr::from_ptr(inputs_json) }.to_str().unwrap();

    let library = unsafe { &mut *(ffi_circom.inner as *mut CircuitLibrary) };
    match library.execute(inputs_json_str) {
        Ok((witness, records)) => {
            let (x, y) = library.get_signals();
            let constraint_evaluation =
                crate::json_export::produce_constraint_evaluation_json(&records, &x, &y, &witness);
            ffi_pass_evals(ctx_handle, constraint_evaluation.as_ptr(), constraint_evaluation.len());
        }
        Err(report) => {
            for report in report.iter() {
                crate::reporting::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff);
            }
        }
    }
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

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
/// This function is called in GO code to free the memory allocated by the Rust code.
pub extern "C" fn free_circom(ffi_circom: *mut FFICircom) {
    if !ffi_circom.is_null() {
        unsafe {
            let _ = Box::from_raw(ffi_circom);
        }
    }
}

pub fn ffi_pass_report(ctx_handle: usize, ptr: *const u8, len: usize) {
    unsafe {
        share_report(ctx_handle, ptr as *const c_void, len);
    };
}
