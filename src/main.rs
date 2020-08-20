#[macro_use]
extern crate log;
extern crate simple_logger;

mod intel_asm_writer;
#[macro_use]
mod operation;

use crate::operation::*;
use std::process::exit;

fn main() {
    simple_logger::init().unwrap();
    debugp!("Started ingles with args: {}", std::env::args().skip(1).collect::<Vec<String>>().join(" "));
    if std::env::args().len() < 2 {
        error!("usage: ingles <input> [writer]");
        exit(1);
    }
    let parse: Vec<Box<Operation>> = parse(&*std::env::args().collect::<Vec<String>>()[1], vec![]);
    let mut args: Vec<Operation> = vec![];

    debugp!("Transforming values...");
    for p in parse {
        args.push(*p);
    }

    let output_type_str = "dynamic";

    if std::env::args().len() > 2 {
        debugp!("Found writer type: {} [args]", &*std::env::args().collect::<Vec<String>>()[2]);
        let output_type = match &*std::env::args().collect::<Vec<String>>()[2] {
            "intel_asm" => OutputType::IntelAsm,
            "dynamic" => OutputType::Dynamic,
            _ => OutputType::Dynamic
        };
        write("test.asm", args, output_type);
    } else {
        debugp!("Found writer type: {} [default]", output_type_str);
        let output_type = match output_type_str {
            "intel_asm" => OutputType::IntelAsm,
            "dynamic" => OutputType::Dynamic,
            _ => OutputType::Dynamic
        };
        write("test.asm", args, output_type);
    }
}
