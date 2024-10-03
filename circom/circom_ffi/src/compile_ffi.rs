use program_structure::program_archive::ProgramArchive;
use std::ffi::c_void;

#[repr(C)]
pub struct FFICircomCircuit {
    inner: *mut c_void,
}

impl FFICircomCircuit {
    pub fn inner(&self) -> *mut c_void {
        self.inner
    }
}

extern "C" {
    fn VeritasIncludeCircomCircuit(ctx_handle: usize, circuit: *const FFICircomCircuit);
}

#[no_mangle]
pub extern "C" fn ffi_compile_circom_circuit(
    ctx_handle: usize,
    ffi_prog_arch: *mut crate::program_archive_ffi::FFIProgramArchive,
) {
    let ffi_prog_arch = unsafe { &mut *ffi_prog_arch };
    let mut buff = Vec::with_capacity(100_000);

    let (program_archive, version, prime) =
        unsafe { &mut *(ffi_prog_arch.inner() as *mut (ProgramArchive, String, String)) };

    match crate::compile::do_type_analysis(program_archive) {
        Ok(_) => match crate::compile::compile_vcp(prime.clone(), program_archive.clone()) {
            Ok((dag, vcp, reports)) => {
                for report in &reports {
                    crate::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff);
                }
                if reports.len() == 0 {
                    let circuit =
                        crate::compile::build_circom_circuit(vcp.clone(), version.clone());

                    let ffi_circuit = Box::new(FFICircomCircuit {
                        inner: Box::into_raw(Box::new((dag, vcp, circuit))) as *mut c_void,
                    });

                    unsafe {
                        VeritasIncludeCircomCircuit(ctx_handle, Box::into_raw(ffi_circuit));
                    }
                }
            }
            Err(reports) => {
                for report in reports {
                    crate::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff);
                }
            }
        },
        Err(reports) => {
            for report in reports {
                crate::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn free_circom_circuit(ffi_circuit: *mut FFICircomCircuit) {
    if !ffi_circuit.is_null() {
        unsafe {
            let _ = Box::from_raw(ffi_circuit);
        }
    }
}
