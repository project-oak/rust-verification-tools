use log::info;
use regex::Regex;
use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;

use inkwell::context::Context;
use inkwell::memory_buffer::MemoryBuffer;
use inkwell::module::Linkage;
use inkwell::module::Module;
use inkwell::types::{AnyType, FunctionType};
use inkwell::values::{AnyValue, BasicValue, BasicValueEnum};
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};
use inkwell::AddressSpace;

// Command line argument parsing
#[derive(Debug, StructOpt)]
#[structopt(name = "rvt-patch-llvm", about = "Preprocess rustc generated llvm code for verification.")]
struct Opt {
    /// Input file.
    #[structopt(name = "INPUT", parse(from_os_str))]
    input: PathBuf,

    /// Output file
    #[structopt(short, long, name = "OUTPUT", parse(from_os_str), default_value = "out")]
    output: PathBuf,

    /// Call initializers from main
    #[structopt(short, long)]
    initializers: bool,

    /// SeaHorn preparation (conflicts with --initializers)
    #[structopt(short, long, conflicts_with = "initializers")]
    seahorn: bool,

    /// Select a specific test to run (instead of 'main')
    #[structopt(short, long, name = "TEST")]
    test: Option<String>,

    /// Increase message verbosity
    #[structopt(short, long, parse(from_occurrences))]
    verbosity: usize,
}

fn main() {
    let opt = Opt::from_args();

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

    if opt.initializers {
        handle_initializers(&context, &mut module);
    }

    if opt.seahorn {
        handle_main(&module, &opt.test);

        handle_panic(&module);

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
// Transformations associated with SeaHorn
////////////////////////////////////////////////////////////////

fn handle_main(module: &Module, otest: &Option<String>) {
    // Remove the main function rustc generates.
    if let Some(main) = module.get_function("main") {
        unsafe { main.delete(); }
        info!("Deleted 'main' (was added by rustc).");
    }

    let re = match otest {
        Some(test) => {
            Regex::new(&(test.len().to_string() + test + r"17h[a-f0-9]{16}E$")).unwrap()
        }
        None => Regex::new(r"4main17h[a-f0-9]{16}E$").unwrap()
    };

    // Print the entry point function name
    if let Some(fun) = get_function(module, &re) {
        // Change the linkage of mangled main function from internal to external.
        // fun.set_linkage(Linkage::External);
        println!("ENTRY: {}", fun.get_name().to_str().unwrap());
    }
}

fn get_function<'ctx>(module: &'ctx Module, re: &Regex) -> Option<FunctionValue<'ctx>> {
    let mut op_fun = module.get_first_function();
    while let Some(fun) = op_fun {
        if re.is_match(fun.get_name().to_str()
                       .expect("ERROR: function name is not in valid UTF-8")) {
            return Some(fun);
        }
        op_fun = fun.get_next_function();
    }
    None
}

fn handle_panic(module: &Module) {
    // TODO: make "spanic" a CL arg.
    if let Some(spanic) = module.get_function("spanic") {
        if let Some(unwind) = module.get_function("rust_begin_unwind") {
            unwind.replace_all_uses_with(spanic);
            info!("Replaced panic handling ('rust_begin_unwind') with 'spanic'.");
        }
    }
}

fn replace_def_with_dec(module: &Module, re: &Regex) {
    if let Some(fun) = get_function(module, re) {
        for bb in fun.get_basic_blocks() {
            unsafe { bb.delete().unwrap(); }
        }
        fun.remove_personality_function();
        fun.set_linkage(Linkage::External);
        info!("Removed the implementation of '{}'.", fun.get_name().to_str().unwrap());
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
