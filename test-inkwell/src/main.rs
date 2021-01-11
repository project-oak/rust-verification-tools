use clap::{Arg, App};
use std::path::Path;
use std::ffi::OsStr;
use regex::Regex;

use inkwell::memory_buffer::MemoryBuffer;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::module::Linkage;
use inkwell::values::FunctionValue;

fn main() {
    // Command line argument parsing (using clap)
    let matches = App::new("Test inkwell")
        // .version("0.1.0")
        // .author("")
        // .about("")
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
    let module = context.create_module_from_ir(memory_buffer)
        .expect("ERROR: failed to create module.");

    handle_initializers(&module);

    handle_main(&module);

    handle_panic(&module);

    replace_def_with_dec(&module, &Regex::new(r"^_ZN3std2io5stdio7_eprint17h[a-f0-9]{16}E$").unwrap());
    replace_def_with_dec(&module, &Regex::new(r"^_ZN3std2io5stdio6_print17h[a-f0-9]{16}E$").unwrap());


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

fn handle_initializers(module: &Module) {
    let mut og = module.get_first_global();
    while let Some(g) = og {
        if let Some(s) = g.get_section() {
            if let Ok(s) = s.to_str() {
                if s.starts_with(".init_array") {
                    let i = g.get_initializer().unwrap().into_struct_value();
                    println!("Global name: {} section: {} initializer: {:?}", g.get_name().to_str().unwrap(), s, i)
                }
            }
        }
        og = g.get_next_global();
    }
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
