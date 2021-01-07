use clap::{Arg, App};
use std::path::Path;

use inkwell::memory_buffer::MemoryBuffer;
use inkwell::context::Context;
// use inkwell::module::Module;

fn main() {
    let matches = App::new("Test inkwell")
        // .version("0.1.0")
        // .author("")
        // .about("")
        .arg(Arg::with_name("output")
                 .short("o")
                 .long("output")
                 .takes_value(true)
                 .help("Output file name"))
        .arg(Arg::with_name("INPUT")
                 .required(true)
                 .index(1)
                 .help("The input file"))
        .get_matches();

    let path_in  = matches.value_of("INPUT").expect("ERROR: missing input file name argument.");
    let path_in  = Path::new(path_in);
    let path_out = matches.value_of("output").unwrap_or("out");
    let path_out = Path::new(path_out);

    let memory_buffer = MemoryBuffer::create_from_file(path_in)
        .expect("ERROR: failed to open file.");
    let context = Context::create();
    let module = context.create_module_from_ir(memory_buffer)
        .expect("ERROR: failed to create module.");
    let mut og = module.get_first_global();
    while let Some(g) = og {
        println!("Global {}", g.get_name().to_str().unwrap());
        og = g.get_next_global()
    }
    module.print_to_file(path_out)
        .expect("ERROR: failed to write to file.");
}
