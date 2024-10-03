use program_structure::program_archive::ProgramArchive;
use std::ffi::{c_char, c_void, CStr};

/// FFIProgramArchive
/// Contains a pointer to a ProgramArchive, a version string, and a prime string
#[repr(C)]
pub struct FFIProgramArchive {
    inner: *mut c_void,
}

impl FFIProgramArchive {
    pub fn inner(&self) -> *mut c_void {
        self.inner
    }
}

extern "C" {
    fn VeritasIncludeProgArch(ctx_handle: usize, arch: *const FFIProgramArchive);
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ffi_build_program_archive(ctx_handle: usize, ctx_json: *const c_char) {
    let ctx_json_src = unsafe { CStr::from_ptr(ctx_json) }.to_str().unwrap();
    let ctx: Result<crate::RuntimeCtx, serde_json::Error> = serde_json::from_str(ctx_json_src);
    let ctx = match ctx {
        Ok(ctx) => ctx,
        Err(e) => {
            crate::report_error(ctx_handle, &e.to_string());
            return;
        }
    };

    match crate::compile::build_program_archive(ctx.version.clone(), ctx.prime.clone(), ctx.src) {
        Ok(prog_lib) => {
            let ffi_prog_arch = Box::new(FFIProgramArchive {
                inner: Box::into_raw(Box::new((
                    prog_lib,
                    String::from(ctx.version.as_str()),
                    ctx.prime,
                ))) as *mut c_void,
            });
            unsafe {
                VeritasIncludeProgArch(ctx_handle, Box::into_raw(ffi_prog_arch));
            }
        }
        Err(reports) => {
            let mut buff = Vec::with_capacity(100_000);
            for report in reports {
                crate::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff);
            }
        }
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn ffi_type_analysis(ctx_handle: usize, ffi_prog_arch: *mut FFIProgramArchive) {
    let ffi_prog_arch = unsafe { &mut *ffi_prog_arch };
    let (program_archive, _, _) =
        unsafe { &mut *(ffi_prog_arch.inner as *mut (ProgramArchive, String, String)) };

    match crate::compile::do_type_analysis(program_archive) {
        Ok(_) => {}
        Err(reports) => {
            let mut buff = Vec::with_capacity(100_000);
            for report in reports {
                crate::report_diagnostic(ctx_handle, &report.to_diagnostic(), &mut buff);
            }
        }
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn free_prog_arch(ffi_prog_arch: *mut FFIProgramArchive) {
    if !ffi_prog_arch.is_null() {
        unsafe {
            let _ = Box::from_raw(ffi_prog_arch);
        }
    }
}
