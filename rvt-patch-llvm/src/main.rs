// Copyright 2020-2021 The Propverify authors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use log::info;
use regex::Regex;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::{AnyType, FunctionType};
use inkwell::values::{AnyValue, BasicValue, BasicValueEnum};
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};
use inkwell::AddressSpace;

// Command line argument parsing
#[derive(StructOpt)]
#[structopt(
    name = "rvt-patch-llvm",
    about = "Preprocess rustc generated llvm code for verification.",
    // version number is taken automatically from Cargo.toml
)]
struct Opt {
    /// Input file.
    #[structopt(name = "INPUT", parse(from_os_str))]
    input: PathBuf,

    /// Output file
    #[structopt(
        short,
        long,
        name = "OUTPUT",
        parse(from_os_str),
        default_value = "out"
    )]
    output: PathBuf,

    /// Eliminate feature tests
    #[structopt(long)]
    features: bool,

    /// Call initializers from main
    #[structopt(long)]
    initializers: bool,

    /// Substitute function calls for SIMD intrinsic operations
    #[structopt(long)]
    intrinsics: bool,

    /// SeaHorn preparation (conflicts with --initializers)
    #[structopt(long, conflicts_with = "initializers")]
    seahorn: bool,

    /// Smack
    #[structopt(long)]
    smack: bool,

    /// Increase message verbosity
    #[structopt(short, long, parse(from_occurrences))]
    verbosity: usize,
}

fn main() {
    let opt = Opt::from_args();

    #[rustfmt::skip]
    stderrlog::new()
        .verbosity(opt.verbosity)
        .init()
        .unwrap();

    // Read the input file
    info!("Reading input from {}", opt.input.to_str().unwrap());
    let memory_buffer =
        MemoryBuffer::create_from_file(&opt.input).expect("ERROR: failed to open file.");

    let context = Context::create();
    let mut module = context
        .create_module_from_ir(memory_buffer)
        .expect("ERROR: failed to create module.");

    if opt.features {
        info!("Patching feature functions");
        stub_functions(
            &context,
            &module,
            &[
                "std::std_detect::detect::arch::__is_feature_detected::avx2",
                "std::std_detect::detect::arch::__is_feature_detected::ssse3",
                "std::std_detect::detect::check_for",
            ],
            |builder| {
                builder.build_return(Some(&context.bool_type().const_int(0, false)));
                ()
            },
        );
    }

    if opt.initializers {
        handle_initializers(&context, &mut module);
    }

    if opt.intrinsics {
        // intrinsics
        fn simd_intrinsic(name: &str) -> bool {
            name.starts_with("llvm.x86") || name.starts_with("llvm.experimental.vector")
        }
        let is = get_function(&module, simd_intrinsic);

        // simd_emulation functions
        fn simd_emulation(name: &str) -> bool {
            name.starts_with("llvm_x86") || name.starts_with("llvm_experimental.vector")
        }
        let rs = get_function(&module, simd_emulation);
        let rs: HashMap<&str, FunctionValue> = rs
            .iter()
            .filter_map(|f| f.get_name().to_str().ok().map(|nm| (nm, *f)))
            .collect();

        info!("Found simd_emulation functions {:?}", rs.keys());

        for i in is {
            let i_name = i.get_name();
            let r_name = i_name.to_str().expect("valid UTF8 symbol name");
            let r_name: String = r_name.replace(".", "_");
            if let Some(r) = rs.get(r_name.as_str()) {
                info!("Replacing intrinsic {:?} with {}", i_name, r_name);
                i.replace_all_uses_with(*r);
            } else {
                info!("Did not find replacement for {:?}", i_name);
            }
        }
    }

    if opt.seahorn {
        handle_main(&module);

        handle_panic(&context, &module);

        replace_def_with_dec(
            &module,
            &Regex::new(r"^_ZN3std2io5stdio7_eprint17h[a-f0-9]{16}E$").unwrap(),
        );
        replace_def_with_dec(
            &module,
            &Regex::new(r"^_ZN3std2io5stdio6_print17h[a-f0-9]{16}E$").unwrap(),
        );
    }

    if opt.smack {
        handle_main(&module);

        replace_def_with_dec(
            &module,
            &Regex::new(r"^__VERIFIER_(assume|assert)$").unwrap(),
        );
        replace_def_with_dec(
            &module,
            &Regex::new(r"^__VERIFIER_nondet_[iu]\d+$").unwrap(),
        );
        replace_def_with_dec(
            &module,
            &Regex::new(r"^_ZN3std2io5stdio7_eprint17h[a-f0-9]{16}E$").unwrap(),
        );
        replace_def_with_dec(
            &module,
            &Regex::new(r"^_ZN3std2io5stdio6_print17h[a-f0-9]{16}E$").unwrap(),
        );
    }

    // Write output file
    info!("Writing output to {}", opt.output.to_str().unwrap());
    if opt.output.extension() == Some(OsStr::new("bc")) {
        // output bitcode
        // TODO: this function returns bool but the doc doesn't say anything about it.
        module.write_bitcode_to_path(&opt.output);
    } else {
        // output disassembled bitcode
        module
            .print_to_file(&opt.output)
            .expect("ERROR: failed to write to file.");
    }
}

////////////////////////////////////////////////////////////////
// Transformations associated with initializers
////////////////////////////////////////////////////////////////

fn handle_initializers(context: &Context, module: &mut Module) {
    if let Some(initializer) =
        collect_initializers(context, module, ".init_array", "__init_function")
    {
        info!(
            "Combined .init_array* initializers into '{}'",
            initializer.get_name().to_str().unwrap()
        );

        let main = module
            .get_function("main")
            .expect("Unable to find 'main' function");
        let mut args = get_fn_args(main);
        assert!(args.len() == 2); // We expect "i32 @main(i32 %0, i8** %1)"
        let i8_type = context.i8_type();
        let pi8_type = i8_type.ptr_type(AddressSpace::Generic);
        let ppi8_type = pi8_type.ptr_type(AddressSpace::Generic);
        args.push(ppi8_type.const_null().as_basic_value_enum());
        insert_call_at_head(context, initializer, args, main);
        info!(
            "Inserted call to '{}' into 'main'",
            initializer.get_name().to_str().unwrap()
        )
    } else {
        info!("No initializers to handle")
    }
}

/// Collect all the initializers in a section (whose name starts with 'prefix')
/// into a single function that calls all the initializers.
fn collect_initializers<'a>(
    context: &Context,
    module: &mut Module<'a>,
    prefix: &str,
    nm: &str,
) -> Option<FunctionValue<'a>> {
    let vs = collect_variables_in_section(module, prefix);
    for v in &vs {
        info!("Found initializer {:?}", v.get_name().to_str().unwrap());
    }

    let fps: Vec<PointerValue> = vs.iter().map(get_initializer_function).collect();

    if !fps.is_empty() {
        let fp = fps[0];

        // dereference the pointer type
        let fp_type = fp.get_type().get_element_type().into_function_type();
        info!("Initializer type {:?}", fp_type.print_to_string());

        Some(build_fanout(context, module, nm, fp_type, fps))
    } else {
        None
    }
}

/// Collect variables that are assigned to a section whose name matches prefix
fn collect_variables_in_section<'a>(module: &Module<'a>, prefix: &str) -> Vec<GlobalValue<'a>> {
    let mut vs = Vec::new();
    // todo: should implement an iterator for global values so that the following loop
    // becomes just iter().filter_map().filter_map().filter()
    let mut og = module.get_first_global();
    while let Some(g) = og {
        if let Some(s) = g.get_section() {
            if let Ok(s) = s.to_str() {
                if s.starts_with(prefix) {
                    vs.push(g)
                }
            }
        }
        og = g.get_next_global();
    }
    vs
}

/// Convert the contents of an initializer section to a function pointer.
///
/// Initializer sections contain structs where the first field is a function pointer cast to
/// some other type.
fn get_initializer_function<'a>(v: &GlobalValue<'a>) -> PointerValue<'a> {
    let i = v.get_initializer().unwrap().into_struct_value();
    assert!(i.get_num_operands() == 2); // expecting two fields in struct
    let i = i.get_operand(0).unwrap();
    assert!(i.get_num_operands() == 1); // expecting bitcast
    let fp = i.get_operand(0).unwrap();
    fp.into_pointer_value()
}

/// Given a list of functions of type 'ty', build a function that calls each function in order
///
/// Assumes (without checking) that return type is void
///
///     define void @fanout(i32 %0, i32 %1) {
///     entry:
///       call void @f1(i32 %0, i32 %1)
///       call void @f2(i32 %0, i32 %1)
///       call void @f3(i32 %0, i32 %1)
///       ret void
///     }
///
fn build_fanout<'a>(
    context: &Context,
    module: &mut Module<'a>,
    nm: &str,
    ty: FunctionType<'a>,
    fps: Vec<PointerValue<'a>>,
) -> FunctionValue<'a> {
    let function = module.add_function(nm, ty, None);
    let args = get_fn_args(function);
    let basic_block = context.append_basic_block(function, "entry");
    let builder = context.create_builder();
    builder.position_at_end(basic_block);

    for fp in fps {
        builder.build_call(fp, &args, "");
        builder.build_return(None);
        // println!("Built function {:?}", function)
    }

    function
}

fn get_fn_args<'a>(function: FunctionValue<'a>) -> Vec<BasicValueEnum<'a>> {
    (0..function.count_params())
        .map(|i| function.get_nth_param(i).unwrap())
        .collect()
}

fn insert_call_at_head<'a>(
    context: &Context,
    f: FunctionValue<'a>,
    args: Vec<BasicValueEnum<'a>>,
    insertee: FunctionValue<'a>,
) {
    let bb = insertee
        .get_first_basic_block()
        .expect("Unable to find function to insert function call into");
    let first_instruction = bb
        .get_first_instruction()
        .expect("Unable to find where to insert function call into function");
    let builder = context.create_builder();
    builder.position_before(&first_instruction);
    builder.build_call(f, &args, "");
}

////////////////////////////////////////////////////////////////
// Function transformation functions
////////////////////////////////////////////////////////////////

/// Apply Rust mangling rule to a function name like 'foo::bar'.
/// Does not insert a unique hash at the end or the final 'E' character.
fn rust_mangle(rust_name: &str) -> String {
    let mangled = &rust_name
        .split("::")
        .into_iter()
        .map(|x| format!("{}{}", x.len(), x))
        .collect::<Vec<String>>()
        .concat();
    "_ZN".to_string() + mangled
}

/// Delete the body of a function
fn delete_body<'ctx>(fun: &FunctionValue<'ctx>) {
    for bb in fun.get_basic_blocks() {
        unsafe {
            // Any use of `bb` after calling `delete` is unsafe
            bb.delete().expect("delete basic block");
        }
    }
}

/// Stub out a list of functions by deleting the existing
/// body of each function and using mk_stub to construct a new body.
///
/// Functions are identified specified by names like "foo::bar"
/// and exclude the uniquifying hash code at the end.
fn stub_functions<F>(context: &Context, module: &Module, functions: &[&str], mk_stub: F)
where
    F: Fn(&Builder),
{
    let builder = context.create_builder();
    for function in functions {
        for fv in get_function_by_unmangled_name(&module, function) {
            info!("Stubbing out function function {}", function);
            delete_body(&fv);
            let basic_block = context.append_basic_block(fv, "entry");
            builder.position_at_end(basic_block);
            mk_stub(&builder)
        }
    }
}

////////////////////////////////////////////////////////////////
// Transformations associated with SeaHorn
////////////////////////////////////////////////////////////////

fn handle_main(module: &Module) {
    // Remove the main function rustc generates. I don't know why, but passing
    // --entry=.. to seahorn is not enough, we also have to remove main.
    if let Some(main) = module.get_function("main") {
        unsafe {
            // Any use of `main` after calling `delete` is unsafe
            main.delete();
        }
        info!("Deleted 'main' (was added by rustc).");
    }
}

/// Find a function whose name matches regex `re`
fn get_function_by_regex<'ctx>(module: &'ctx Module, re: &Regex) -> Vec<FunctionValue<'ctx>> {
    get_function(module, |name| re.is_match(name))
}

/// Find a function whose name matches a Rust-mangled name
fn get_function_by_unmangled_name<'ctx>(
    module: &'ctx Module,
    prefix: &str,
) -> Vec<FunctionValue<'ctx>> {
    let prefix = format!("{}17h", rust_mangle(prefix));
    get_function(module, |name| name.starts_with(&prefix))
}

/// Find a function whose name satisfies predicate `is_match`
fn get_function<'ctx, P>(module: &'ctx Module, is_match: P) -> Vec<FunctionValue<'ctx>>
where
    P: Fn(&str) -> bool,
{
    let mut funs = vec![];
    let mut op_fun = module.get_first_function();
    while let Some(fun) = op_fun {
        if fun.get_name().to_str().map(&is_match).unwrap_or(false) {
            funs.push(fun);
        }
        op_fun = fun.get_next_function();
    }
    funs
}

fn handle_panic(context: &Context, module: &Module) {
    // TODO: make "spanic" a CL arg.
    if let Some(spanic) = module.get_function("spanic") {
        let builder = context.create_builder();

        // Delete the body of panic functions, and replace it with a call to
        // `spanic`.

        // Note that std::panicking::begin_panic can have multiple instances
        // with different hash suffix, I'm not sure why.
        for fv in module
            .get_function("rust_begin_unwind")
            .into_iter()
            .chain(get_function_by_unmangled_name(
                &module,
                "std::panicking::begin_panic",
            ))
            .chain(get_function_by_unmangled_name(
                &module,
                "core::panicking::panic",
            ))
        {
            delete_body(&fv);
            let basic_block = context.append_basic_block(fv, "entry");
            builder.position_at_end(basic_block);
            builder.build_call(spanic, &[], "call");
            builder.build_return(None);

            info!(
                "Replaced the body of '{}' with a call to '{}'.",
                fv.get_name().to_string_lossy(),
                spanic.get_name().to_string_lossy(),
            );
        }
    }
}

/// Change a function to a declaration by
/// deleting all basic blocks and modifying metadata
/// such as personality_function, linkage, etc.
fn replace_def_with_dec(module: &Module, re: &Regex) {
    for fun in get_function_by_regex(module, re) {
        delete_body(&fun);
        fun.remove_personality_function();
        fun.set_linkage(Linkage::External);
        info!(
            "Removed the implementation of '{}'.",
            fun.get_name().to_str().unwrap()
        );
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
