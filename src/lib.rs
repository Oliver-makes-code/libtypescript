#![feature(decl_macro)]

use core::slice;
use std::{
    alloc::{Layout, alloc, dealloc},
    ffi::c_char,
    ptr,
};

use deno_ast::{
    MediaType, ModuleSpecifier, ParseParams, SourceMapOption, parse_module, swc::parser::TsSyntax,
};

macro c_enum {
    (
        $ty:ident : $base_ty:ty {
            $($rest:tt),* $(,)?
        }
    ) => {
        type $ty = $base_ty;
        c_enum!(@impl $base_ty, 0, $($rest),*);
    },

    (
        $ty:ident {
            $($rest:tt),* $(,)?
        }
    ) => {
        type $ty = i32;
        c_enum!(@impl i32, 0, $($rest),*);
    },

    (@impl $base_ty:ty, $_idx:expr,) => {},

    (@impl $base_ty:ty, $_idx:expr,
        $name:ident = $val:literal, $($rest:tt),*
    ) => {
        const $name: $base_ty = $val;
        c_enum!(@impl $base_ty, ($val + 1), $($rest),*);
    },

    (@impl $base_ty:ty, $idx:expr,
        $name:ident, $($rest:tt),*
    ) => {
        const $name: $base_ty = $idx;
        c_enum!(@impl $base_ty, ($idx + 1), $($rest),*);
    },

    (@impl $base_ty:ty, $_idx:expr,
        $name:ident = $val:literal
    ) => {
        const $name: $base_ty = $val;
    },

    (@impl $base_ty:ty, $idx:expr,
        $name:ident
    ) => {
        const $name: $base_ty = $idx;
    },
}

unsafe fn ptr_to_ref_mut<'a, T>(ptr: *mut T) -> Option<&'a mut T> {
    unsafe {
        if !ptr.is_aligned() || ptr.is_null() {
            return None;
        }

        return Some(&mut *ptr);
    }
}

unsafe fn move_slice_to_heap<T: Copy>(slice: &[T], size: &mut usize) -> *mut T {
    *size = slice.len();

    unsafe {
        let layout = Layout::array::<T>(slice.len()).unwrap();

        let ptr = alloc(layout) as *mut T;

        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }

        ptr::copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());

        return ptr;
    }
}

unsafe fn drop_slice_from_heap<T>(ptr: *mut T, size: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }

    let layout = Layout::array::<T>(size).unwrap();

    unsafe {
        dealloc(ptr as *mut u8, layout);
    }
}

unsafe fn cstr_to_str<'a>(cstr: *const c_char, len: usize) -> Option<&'a str> {
    if cstr.is_null() {
        return None;
    }

    let bytes = unsafe { slice::from_raw_parts(cstr as *const u8, len) };

    return std::str::from_utf8(bytes).ok();
}

unsafe fn disown_str_to_cstr(s: &str, len: &mut usize) -> *const c_char {
    unsafe {
        return move_slice_to_heap(s.as_bytes(), len) as *const c_char;
    }
}

c_enum!(CompileStatus: u8 {
    OK,
    INVALID_POINTER,
    TYPESCRIPT_COMPILE_ERROR
});

#[unsafe(no_mangle)]
unsafe extern "C" fn ts_compile(
    input: *const c_char,
    input_len: usize,
    filename: *const c_char,
    filename_len: usize,
    module_or_error: *mut *const c_char,
    module_or_error_len: *mut usize,
) -> CompileStatus {
    unsafe {
        let Some(module_or_error) = ptr_to_ref_mut(module_or_error) else {
            return INVALID_POINTER;
        };

        let Some(module_or_error_len) = ptr_to_ref_mut(module_or_error_len) else {
            return INVALID_POINTER;
        };

        let Some(input) = cstr_to_str(input, input_len) else {
            return INVALID_POINTER;
        };

        let Some(filename) = cstr_to_str(filename, filename_len) else {
            return INVALID_POINTER;
        };

        let (message, success) = compile_typescript(input, filename)
            .map(|ok| (ok, true))
            .unwrap_or_else(|err| (err, false));

        *module_or_error = disown_str_to_cstr(&message, module_or_error_len);

        if success {
            return OK;
        } else {
            return TYPESCRIPT_COMPILE_ERROR;
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn ts_compile_free(str: *const c_char, size: usize) {
    unsafe {
        drop_slice_from_heap(str as *mut u8, size);
    }
}

fn compile_typescript(input: &str, filename: &str) -> Result<String, String> {
    let specifier_res = ModuleSpecifier::parse(&format!("file://{}", filename));

    let Ok(specifier) = specifier_res else {
        let err = specifier_res.unwrap_err();
        return Err(err.to_string());
    };

    let parsed_source_res = parse_module(ParseParams {
        specifier,
        media_type: MediaType::TypeScript,
        text: input.into(),
        capture_tokens: true,
        maybe_syntax: Some(deno_ast::swc::parser::Syntax::Typescript(TsSyntax {
            tsx: false,
            decorators: true,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: true,
        })),
        scope_analysis: false,
    });

    let Ok(parsed_source) = parsed_source_res else {
        return Err(parsed_source_res.unwrap_err().to_string());
    };

    let transpiled_res = parsed_source.transpile(
        &deno_ast::TranspileOptions {
            imports_not_used_as_values: deno_ast::ImportsNotUsedAsValues::Remove,
            ..Default::default()
        },
        &deno_ast::TranspileModuleOptions {
            module_kind: Some(deno_ast::ModuleKind::Esm),
        },
        &deno_ast::EmitOptions {
            source_map: SourceMapOption::Inline,
            inline_sources: true,
            ..Default::default()
        },
    );

    let Ok(transpiled) = transpiled_res else {
        return Err(transpiled_res.unwrap_err().to_string());
    };

    let transpiled_source = transpiled.into_source();

    let transpiled_text = transpiled_source.text;

    return Ok(transpiled_text);
}
