mod intel_asm_writer;
mod operation;

use crate::operation::*;
use std::process::exit;

fn main() {
    if std::env::args().len() < 2 {
        eprintln!("usage: ingles <input> [writer]");
        exit(1);
    }
    let parse: Vec<Box<Operation>> = parse(&*std::env::args().collect::<Vec<String>>()[1]);
    let mut args: Vec<Operation> = vec![];

    for p in parse {
        args.push(*p);
    }

    let output_type_str = "dynamic";

    if std::env::args().len() > 2 {
        let output_type = match &*std::env::args().collect::<Vec<String>>()[2] {
            "intel_asm" => OutputType::IntelAsm,
            "dynamic" => OutputType::Dynamic,
            _ => OutputType::Dynamic
        };
        write("test.asm", args, output_type);
    } else {
        let output_type = match output_type_str {
            "intel_asm" => OutputType::IntelAsm,
            "dynamic" => OutputType::Dynamic,
            _ => OutputType::Dynamic
        };
        write("test.asm", args, output_type);
    }
}
