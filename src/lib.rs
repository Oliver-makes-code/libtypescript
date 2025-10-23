use std::ffi::{c_char, CStr, CString};

use deno_ast::{
    parse_module, swc::parser::TsSyntax, MediaType, ModuleSpecifier, ParseParams, SourceMapOption,
};

#[unsafe(no_mangle)]
unsafe extern "C" fn ts_compile(
    input_raw: *const c_char,
    filename_raw: *const c_char,
    module_or_error: *mut *mut c_char,
) -> bool {
    unsafe {
        let input_cstr = CStr::from_ptr(input_raw);
        let filename_cstr = CStr::from_ptr(filename_raw);

        let input = input_cstr.to_str().unwrap().to_string();
        let filename = filename_cstr.to_str().unwrap().to_string();

        let module_res = compile_typescript(input, filename);

        let Ok(module) = module_res else {
            let err = module_res.unwrap_err();
            let err_cstr = CString::new(err).unwrap();
            *module_or_error = err_cstr.into_raw();
            return false;
        };

        let module_cstr = CString::new(module).unwrap();
        *module_or_error = module_cstr.into_raw();
        return true;
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn ts_compile_free(module: *mut c_char) {
    unsafe {
        if !module.is_null() {
            let _ = CString::from_raw(module);
        }
    }
}

fn compile_typescript(input: String, filename: String) -> Result<String, String> {
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
