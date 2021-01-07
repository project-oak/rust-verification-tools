use clap::{Arg, App};
use std::path::Path;
use std::ffi::OsStr;

use inkwell::memory_buffer::MemoryBuffer;
use inkwell::context::Context;
use inkwell::module::Module;

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


    let memory_buffer = MemoryBuffer::create_from_file(path_in)
        .expect("ERROR: failed to open file.");


    let context = Context::create();
    let module = context.create_module_from_ir(memory_buffer)
        .expect("ERROR: failed to create module.");

    handle_initializers(&module);

    if path_out.extension() == Some(OsStr::new("bc")) {
        // output bitcode
        module.write_bitcode_to_path(path_out);
    } else {
        // output disassembled bitcode
        // TODO: this function returns bool but the doc doesn't say anything about it.
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

////////////////////////////////////////////////////////////////
// End
////////////////////////////////////////////////////////////////
