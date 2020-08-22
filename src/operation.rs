extern crate regex;

use std::fs::File;
use std::io::{Read, Write};
use self::regex::Regex;
use crate::operation::OperationType::*;
use crate::operation::OutputType::*;
use crate::intel_asm_writer::IntelAsmWriter;
use std::fmt::{Display, Formatter};
use core::fmt;
use std::ops::Index;
use std::borrow::Borrow;
use std::process::exit;
use std::time::Instant;

// Change this value to manage printing debugging info.
pub(crate) static DEBUG: bool = false;

#[macro_export]
macro_rules! debugp(
    ($str:expr,$($arg:tt)+) => (
        if DEBUG {
            debug!($str, $($arg)+);
        }
    );
    ($($arg:tt)+) => (
        if DEBUG {
            debug!($($arg)+)
        }
    )
);

macro_rules! matches(
    ($e:expr, $p:pat) => (
        match $e {
            $p => true,
            _ => false
        }
    )
);

pub struct Operation {
    pub op: OperationType,
    pub args: Vec<String>,
    pub orig: String,
    pub line: usize,
    pub file: String
}

impl ToString for Operation {
    fn to_string(&self) -> String {
        format!("({}, {})", self.op, self.args.join(", "))
    }
}

impl Index<usize> for Operation {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        self.args[index].borrow()
    }
}

pub fn parse(file: &str, inc_ban_list: Vec<String>) -> Vec<Box<Operation>> {
    info!("Started parsing {}...", file);
    let start_instant = Instant::now();
    let mut file_fs = File::open(file).unwrap();
    let contents = &mut String::new();
    file_fs.read_to_string(contents).unwrap();
    let lines = contents.split("\n");
    let mut ops: Vec<Operation> = vec![];
    let mut ok = true;
    for (index, line) in lines.enumerate() {
        let mut op: Option<Operation> = Option::None;
        push_to(&mut op, SetWriter, line, Regex::new(r"^#writer (?P<b>.+)$").unwrap(), index+1, file);
        push_to(&mut op, Move, line, Regex::new(r"^(?P<a>.+) = (?P<b>.+)$").unwrap(), index+1, file);
        push_to(&mut op, FunctionStart, line, Regex::new(r"^func begin (?P<a>.+)$").unwrap(), index+1, file);
        push_to(&mut op, FunctionEnd, line, Regex::new(r"^func end$").unwrap(), index+1, file);
        push_to(&mut op, FunctionEnter, line, Regex::new(r"^call (?P<a>.+)$").unwrap(), index+1, file);
        push_to(&mut op, SetStartFunction, line, Regex::new(r"^#start_func (?P<b>.+)$").unwrap(), index+1, file);
        push_to(&mut op, Add, line, Regex::new(r"^(?P<a>.+) \+= (?P<b>.+)$").unwrap(), index+1, file);
        push_to(&mut op, Subtract, line, Regex::new(r"^(?P<a>.+) -= (?P<b>.+)$").unwrap(), index+1, file);
        push_to(&mut op, Multiply, line, Regex::new(r"^(?P<a>.+) \*= (?P<b>.+)$").unwrap(), index+1, file);
        push_to(&mut op, Divide, line, Regex::new(r"^(?P<a>.+) /= (?P<b>.+)$").unwrap(), index+1, file);
        push_to(&mut op, TextSection, line, Regex::new(r"^#text$").unwrap(), index+1, file);
        push_to(&mut op, DataSection, line, Regex::new(r"^#data$").unwrap(), index+1, file);
        push_to(&mut op, UninitializedDataSection, line, Regex::new(r"^#udata$").unwrap(), index+1, file);
        push_to(&mut op, ExitProcess, line, Regex::new(r"^exit proc with (?P<code>.+)$").unwrap(), index+1, file);
        push_to(&mut op, PushToStack, line, Regex::new(r"^push value (?P<code>.+)$").unwrap(), index+1, file);
        push_to(&mut op, PopFromStack, line, Regex::new(r"^pop value (?P<code>.+)$").unwrap(), index+1, file);
        push_to(&mut op, ReadFromMemory, line, Regex::new(r"^read (?P<code>.+) into (?P<reg>.+)$").unwrap(), index+1, file);
        push_to(&mut op, WriteToMemory, line, Regex::new(r"^write (?P<code>.+) into (?P<codealso>.+)$").unwrap(), index+1, file);
        push_to(&mut op, DefineVariable, line, Regex::new(r"^variable (?P<code>.+): (?P<type>[bwdq]) set to (?P<codealso>.+)$").unwrap(), index+1, file);
        push_to(&mut op, DefineUninitVariable, line, Regex::new(r"^variable (?P<code>.+): (?P<type>[bwdq])$").unwrap(), index+1, file);
        push_to(&mut op, RunInterrupt, line, Regex::new(r"^run interrupt (?P<code>.+)$").unwrap(), index+1, file);
        push_to(&mut op, None, line, Regex::new(r"^\s*$").unwrap(), index+1, file);
        push_to(&mut op, None, line, Regex::new(r"^//.*$").unwrap(), index+1, file);
        push_to(&mut op, ReadFromMemoryRange, line, Regex::new(r"^read \[(?P<code>.+)\.\.(?P<code2>.+)] into (?P<reg>.+)$").unwrap(), index+1, file);
        if Regex::new(r"^#include (?P<b>.+)$").unwrap().is_match(line) {
            let args = Regex::new(r"^#include (?P<b>.+)$").unwrap().captures(line).unwrap();
            let mut vec: Vec<String> = vec![];
            for i in args.iter().skip(1) {
                vec.push(i.unwrap().as_str().parse().unwrap());
            }
            if inc_ban_list.contains(&vec[0]) {
                warn!("Caught include loop.");
                warn!("{0} attempted to include {1}, while {1} was including {0}", file, vec[0]);
                warn!("Include discarded. This may not be what you want.");
                continue;
            }
            info!("Including file {} in {}", vec[0], file);
            let mut new_ban_list = inc_ban_list.clone();
            new_ban_list.append(&mut vec![file.to_string()]);
            let inc = parse(&*vec[0], new_ban_list);
            for b in inc {
                ops.push(*b);
            }
        } else if op.is_none() {
            error!("Invalid statement at line {}", index);
            ok = false;
        } else {
            ops.push(op.expect("value is none? but i thought i checked for that..."));
        }
    }
    if !ok {
        error!("Please fix all errors and try again.");
        exit(1);
    }
    let mut final_ops: Vec<Box<Operation>> = vec![];
    debugp!("Transforming values...");
    for o in ops {
        final_ops.push(Box::from(Operation { op: o.op, args: o.args, orig: o.orig, line: o.line, file: o.file }));
    };
    let end_instant = Instant::now();
    info!("Successfully parsed {}. [took {:.02} seconds, {} objects included]", file, end_instant.checked_duration_since(start_instant).unwrap().as_secs_f64(), final_ops.len());
    final_ops
}

fn push_to(op: &mut Option<Operation>, operation_type: OperationType, line: &str, re: Regex, index: usize, file: &str) -> () {
    let line = &*Regex::new(r"^\s+").unwrap().replace_all(line, "");
    let line = &*Regex::new(r"\s+$").unwrap().replace_all(line, "");
    let line = &*Regex::new(r"\s+").unwrap().replace_all(line, " ");
    if re.is_match(line) {
        let args = re.captures(line).unwrap();
        let mut vec: Vec<String> = vec![];
        for i in args.iter().skip(1) {
            vec.push(i.unwrap().as_str().parse().unwrap());
        }
        debugp!("[{:>3}] {} : {} [{}]", index, line, operation_type, vec.join(", "));
        let r = Some(Operation { op: operation_type, args: vec, orig: line.parse().unwrap(), line: index, file: file.to_string() });
        *op = r;
    }
}

#[derive(std::cmp::PartialEq, Copy, Clone)]
pub enum OutputType { IntelAsm, Dynamic }
pub(crate) trait OutputWriter {
    fn generate(operation: Operation) -> Vec<u8>;
}

#[allow(unused)]
pub fn write(file: &str, ops: Vec<Operation>, output_type: OutputType) {
    info!("Started writing file...");
    let start_instant = Instant::now();
    let mut file = File::create(file).unwrap();
    let mut current_writer: fn(Operation) -> Vec<u8> = IntelAsmWriter::generate;
    for op in ops {
        if matches!(op.op, SetWriter) && op.args.len() > 1 {
            current_writer = match &*op.args[0] {
                "intel_asm" => IntelAsmWriter::generate,
                _ => |op| { vec![] }
            };
            info!("Set writer to: {}", &*op.args[0]);
        }
        let buf_gen = match output_type {
            IntelAsm => IntelAsmWriter::generate,
            _ => current_writer
        };
        let out = &*buf_gen(op);
        if out.len() > 0 {
            debugp!("Adding {} bytes to output...", out.len());
        }
        file.write(out);
    }
    debugp!("Flushing...");
    file.flush().unwrap();
    let end_instant = Instant::now();
    // Writing a file can be so fast that the seconds value isnt very useful.
    // To fix this, if the seconds value is less than .01, show nanos instead.
    let dur = end_instant.checked_duration_since(start_instant).unwrap();
    if dur.as_secs_f64() < 0.01 {
        info!("Finished writing file. [took {} microseconds]", dur.as_micros());
    } else {
        info!("Finished writing file. [took {:.02} seconds]", dur.as_secs_f64());
    }
}

pub enum OperationType {
    SetWriter,
    Move,
    FunctionStart,
    FunctionEnd,
    FunctionEnter,
    Add,
    Subtract,
    Multiply,
    Divide,
    TextSection,
    DataSection,
    UninitializedDataSection,
    SetStartFunction,
    ExitProcess,
    PushToStack,
    PopFromStack,
    ReadFromMemory,
    ReadFromMemoryRange,
    WriteToMemory,
    DefineVariable,
    DefineUninitVariable,
    RunInterrupt,
    None
}

impl Display for OperationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Move => "MOV",
            None => "NONE",
            FunctionStart => "FUNC START",
            FunctionEnd => "FUNC END",
            SetStartFunction => "FUNC BEGIN",
            FunctionEnter => "FUNC CALL",
            Add => "ADD",
            Subtract => "SUB",
            Multiply => "MUL",
            Divide => "DIV",
            TextSection => "SECT TEXT",
            DataSection => "SECT DATA",
            UninitializedDataSection => "SECT UDATA",
            ExitProcess => "EXIT PROC",
            SetWriter => "SET WRITER",
            PushToStack => "PUSH",
            PopFromStack => "POP",
            ReadFromMemory => "READ",
            WriteToMemory => "WRITE",
            DefineVariable => "VARIABLE",
            DefineUninitVariable => "UVARIABLE",
            RunInterrupt => "INTERRUPT",
            ReadFromMemoryRange => "READ RANGE"
        })
    }
}
