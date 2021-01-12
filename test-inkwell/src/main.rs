use clap::{Arg, App};
use std::path::Path;
use std::ffi::OsStr;
use regex::Regex;

use inkwell::memory_buffer::MemoryBuffer;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::module::Linkage;
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};
use inkwell::values::{AnyValue, BasicValueEnum};
use inkwell::types::{FunctionType};

fn main() {
    // Command line argument parsing (using clap)
    let matches = App::new("Test inkwell")
        // .version("0.1.0")
        // .author("")
        // .about("")
        .arg(Arg::with_name("initializers")
             .short("i")
             .help("Call initializers from main"))
        .arg(Arg::with_name("seahorn")
             .short("s")
             .help("SeaHorn preparation (conflicts with --initializers)"))
        .arg(Arg::with_name("INPUT")
             .help("Input file name")
             .required(true)
             .index(1))
        .arg(Arg::with_name("OUTPUT")
             .help("Output file name")
             .short("o")
             .long("output")
             .takes_value(true)
             .default_value("out"))
        .get_matches();

    let path_in  = matches.value_of("INPUT")
        .expect("ERROR: missing input file name argument.");
    let path_in  = Path::new(path_in);

    let path_out = matches.value_of("OUTPUT")
        .expect("ERROR: missing output file name argument.");
    let path_out = Path::new(path_out);


    // Read the input file
    let memory_buffer = MemoryBuffer::create_from_file(path_in)
        .expect("ERROR: failed to open file.");

    let context = Context::create();
    let mut module = context.create_module_from_ir(memory_buffer)
        .expect("ERROR: failed to create module.");

    if matches.is_present("initializers") {
        handle_initializers(&context, &mut module);
    }

    if matches.is_present("seahorn") {
        handle_main(&module);

        handle_panic(&module);

        replace_def_with_dec(&module, &Regex::new(r"^_ZN3std2io5stdio7_eprint17h[a-f0-9]{16}E$").unwrap());
        replace_def_with_dec(&module, &Regex::new(r"^_ZN3std2io5stdio6_print17h[a-f0-9]{16}E$").unwrap());
    }


    // Write output file
    if path_out.extension() == Some(OsStr::new("bc")) {
        // output bitcode
        // TODO: this function returns bool but the doc doesn't say anything about it.
        module.write_bitcode_to_path(path_out);
    } else {
        // output disassembled bitcode
        module.print_to_file(path_out)
            .expect("ERROR: failed to write to file.");
    }
}

////////////////////////////////////////////////////////////////
// Transformations associated with initializers
////////////////////////////////////////////////////////////////

fn handle_initializers(context: &Context, module: &mut Module) {
    let _ = collect_initializers(context, module, ".init_array", "my_initializer");
}

/// Collect all the initializers in a section (whose name starts with 'prefix')
/// into a single function that calls all the initializers.
fn collect_initializers<'a>(context: &Context, module: &mut Module<'a>, prefix: &str, nm: &str) -> Option<FunctionValue<'a>> {
    let vs = collect_variables_in_section(module, prefix);
    // println!("Global name: {} initializer: {:?}", v.get_name().to_str().unwrap(), fp);

    let fps : Vec<PointerValue> = vs.iter().map(get_initializer_function).collect();

    if ! fps.is_empty() {
        let fp = fps[0];

        // dereference the pointer type
        let fp_type = fp.get_type().get_element_type().into_function_type();
        println!("type {:?}", fp_type);

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
fn build_fanout<'a>(context: &Context, module: &mut Module<'a>, nm: &str, ty: FunctionType<'a>, fps: Vec<PointerValue<'a>>) -> FunctionValue<'a> {
    let function = module.add_function(nm, ty, None);
    let args : Vec<BasicValueEnum> = (0 .. function.count_params()).map(|i| function.get_nth_param(i).unwrap()).collect();
    let basic_block = context.append_basic_block(function, "entry");
    let builder = context.create_builder();
    builder.position_at_end(basic_block);

    for fp in fps {
        builder.build_call(fp, &args, "");
        builder.build_return(None);
        println!("Built function {:?}", function)
    }

    function
}

////////////////////////////////////////////////////////////////
// Transformations associated with SeaHorn
////////////////////////////////////////////////////////////////

fn handle_main(module: &Module) {
    // Remove the main function rustc generates.
    if let Some(main) = module.get_function("main") {
        unsafe { main.delete(); }
    }

    // Change the linkage of mangled main function from internal to external.
    if let Some(main) = get_function(module, &Regex::new(r"4main17h[a-f0-9]{16}E$").unwrap()) {
        // main.set_linkage(Linkage::External);
        println!("MAIN: {}", main.get_name().to_str().unwrap());
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
    }
}

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
