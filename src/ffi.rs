//! C ABI for in-process embedding (host-ffi, SPEC §8).
//!
//! Build `--crate-type cdylib` (libcurt) and call:
//!
//! ```c
//! typedef char* (*curt_tool_fn)(const char* arg, void* userdata);
//! typedef struct { const char* name; curt_tool_fn call; void* userdata; } CurtTool;
//! // returns 0 ok / 1 program failure / 2 parse-or-check failure;
//! // *out is always set to a JSON C string — free it with curt_free.
//! int curt_eval_tools(const char* src, unsigned char fs, unsigned char net,
//!                     const CurtTool* tools, size_t n_tools, char** out);
//! int curt_eval(const char* src, unsigned char fs, unsigned char net, char** out);
//! void curt_free(char* ptr);
//! ```
//!
//! Result JSON: `{"ok":true,"stdout":"..."}` or
//! `{"ok":false,"diag":{...}}` (the SPEC §7 diagnostic, parse/check) or
//! `{"ok":false,"error":"..."}` (runtime failure message).
//!
//! String ownership: tool callbacks return a C string that must stay valid
//! until the callback returns — the library copies it synchronously and
//! never frees it. Strings the library returns are freed with `curt_free`.
//! Reentrancy (a tool calling back into curt) is unsupported in v1.
//!
//! Registered tools surface in curt as `host.<name> arg` (str -> str);
//! unregistered names yield a rescuable err value — deny-by-default holds.

use crate::eval::{Caps, HostFn, Interp};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};

pub type CurtToolFn =
    unsafe extern "C" fn(arg: *const c_char, userdata: *mut c_void) -> *mut c_char;

#[repr(C)]
pub struct CurtTool {
    pub name: *const c_char,
    pub call: CurtToolFn,
    pub userdata: *mut c_void,
}

fn out_json(out: *mut *mut c_char, json: String) {
    let c = CString::new(json).unwrap_or_else(|_| CString::new("{\"ok\":false}").unwrap());
    unsafe { *out = c.into_raw() };
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t")
}

/// # Safety
/// `src` must be a valid NUL-terminated UTF-8 string; `tools` must point to
/// `n_tools` valid entries (or be null with n_tools == 0); `out` must be a
/// valid pointer. Tool callbacks must follow the ownership contract above.
#[no_mangle]
pub unsafe extern "C" fn curt_eval_tools(
    src: *const c_char,
    fs: u8,
    net: u8,
    tools: *const CurtTool,
    n_tools: usize,
    out: *mut *mut c_char,
) -> i32 {
    if src.is_null() || out.is_null() {
        return 2;
    }
    let Ok(src) = CStr::from_ptr(src).to_str() else {
        out_json(out, "{\"ok\":false,\"error\":\"source is not UTF-8\"}".into());
        return 2;
    };
    let mut host: HashMap<String, HostFn> = HashMap::new();
    if !tools.is_null() {
        for i in 0..n_tools {
            let t = &*tools.add(i);
            let Ok(name) = CStr::from_ptr(t.name).to_str() else { continue };
            let (call, userdata) = (t.call, t.userdata as usize);
            host.insert(
                name.to_string(),
                Box::new(move |arg: &str| {
                    let c_arg = CString::new(arg).map_err(|_| "arg contains NUL".to_string())?;
                    let ret = unsafe { call(c_arg.as_ptr(), userdata as *mut c_void) };
                    if ret.is_null() {
                        return Err("tool returned null".into());
                    }
                    // copy synchronously; the host keeps ownership (contract)
                    let s = unsafe { CStr::from_ptr(ret) }.to_string_lossy().into_owned();
                    Ok(s)
                }),
            );
        }
    }
    match crate::parse_source_spanned(src)
        .and_then(|(ast, pos)| crate::infer::check_at(&ast, &pos).map(|_| ast))
    {
        Ok(ast) => {
            match Interp::run_hosted(&ast, Caps { fs: fs != 0, net: net != 0 }, vec!["curt".into()], host) {
                Ok(stdout) => {
                    out_json(out, format!("{{\"ok\":true,\"stdout\":\"{}\"}}", esc(&stdout)));
                    0
                }
                Err(m) => {
                    out_json(out, format!("{{\"ok\":false,\"error\":\"{}\"}}", esc(&m)));
                    1
                }
            }
        }
        Err(mut d) => {
            d.replacement = crate::repair::synthesize(src, &d);
            out_json(out, format!("{{\"ok\":false,\"diag\":{d}}}"));
            2
        }
    }
}

/// # Safety
/// Same contract as `curt_eval_tools` with no tools.
#[no_mangle]
pub unsafe extern "C" fn curt_eval(
    src: *const c_char,
    fs: u8,
    net: u8,
    out: *mut *mut c_char,
) -> i32 {
    curt_eval_tools(src, fs, net, std::ptr::null(), 0, out)
}

/// # Safety
/// `ptr` must have been returned by this library (or be null).
#[no_mangle]
pub unsafe extern "C" fn curt_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{Caps, Interp};
    use std::collections::HashMap;

    #[test]
    fn run_hosted_captures_stdout() {
        let ast = crate::parse_source("print (1 + 2)\nprint \"hi\"\n").unwrap();
        let out = Interp::run_hosted(&ast, Caps { fs: false, net: false }, vec![], HashMap::new())
            .unwrap();
        assert_eq!(out, "3\nhi\n");
    }

    #[test]
    fn host_tool_callable_and_deny_by_default() {
        let ast = crate::parse_source(
            "print (host.ask \"q\")\nprint (host.nope \"x\" ? \"denied\")\n",
        )
        .unwrap();
        let mut host: HashMap<String, crate::eval::HostFn> = HashMap::new();
        host.insert("ask".into(), Box::new(|arg: &str| Ok(format!("echo:{arg}"))));
        let out =
            Interp::run_hosted(&ast, Caps { fs: false, net: false }, vec![], host).unwrap();
        assert_eq!(out, "echo:q\ndenied\n");
    }

    #[test]
    fn host_tool_error_is_rescuable() {
        let ast = crate::parse_source("print (host.flaky \"x\" ? \"fallback\")\n").unwrap();
        let mut host: HashMap<String, crate::eval::HostFn> = HashMap::new();
        host.insert("flaky".into(), Box::new(|_: &str| Err("boom".into())));
        let out =
            Interp::run_hosted(&ast, Caps { fs: false, net: false }, vec![], host).unwrap();
        assert_eq!(out, "fallback\n");
    }

    #[test]
    fn ffi_roundtrip() {
        let src = std::ffi::CString::new("print (40 + 2)\n").unwrap();
        let mut out: *mut c_char = std::ptr::null_mut();
        let rc = unsafe { curt_eval(src.as_ptr(), 0, 0, &mut out) };
        assert_eq!(rc, 0);
        let s = unsafe { CStr::from_ptr(out) }.to_str().unwrap().to_string();
        unsafe { curt_free(out) };
        assert_eq!(s, "{\"ok\":true,\"stdout\":\"42\\n\"}");
    }
}
