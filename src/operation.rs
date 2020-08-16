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
    pub line: usize
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

pub fn parse(file: &str) -> Vec<Box<Operation>> {
    let mut file = File::open(file).unwrap();
    let contents = &mut String::new();
    file.read_to_string(contents).unwrap();
    let lines = contents.split("\n");
    let mut ops: Vec<Operation> = vec![];
    for (index, line) in lines.enumerate() {
        ops.push(check(SetWriter, line, Regex::new(r"^#writer (?P<b>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(Move, line, Regex::new(r"^(?P<a>.+) = (?P<b>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(FunctionStart, line, Regex::new(r"^func begin (?P<a>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(FunctionEnd, line, Regex::new(r"^func end$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(FunctionEnter, line, Regex::new(r"^call (?P<a>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(SetStartFunction, line, Regex::new(r"^#start_func (?P<b>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(Add, line, Regex::new(r"^(?P<a>.+) \+= (?P<b>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(Subtract, line, Regex::new(r"^(?P<a>.+) -= (?P<b>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(Multiply, line, Regex::new(r"^(?P<a>.+) \*= (?P<b>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(Divide, line, Regex::new(r"^(?P<a>.+) /= (?P<b>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(TextSection, line, Regex::new(r"^#text$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(DataSection, line, Regex::new(r"^#data$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(UninitializedDataSection, line, Regex::new(r"^#udata$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(ExitProcess, line, Regex::new(r"^exit proc with (?P<code>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(PushToStack, line, Regex::new(r"^push value (?P<code>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(PopFromStack, line, Regex::new(r"^pop value (?P<code>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(ReadFromMemory, line, Regex::new(r"^read (?P<code>.+) into (?P<reg>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(WriteToMemory, line, Regex::new(r"^write (?P<code>.+) into (?P<codealso>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(DefineVariable, line, Regex::new(r"^variable (?P<code>.+): (?P<type>[bwdq]) set to (?P<codealso>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(DefineUninitVariable, line, Regex::new(r"^variable (?P<code>.+): (?P<type>[bwdq])$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
        ops.push(check(RunInterrupt, line, Regex::new(r"^run interrupt (?P<code>.+)$").unwrap(), index+1).unwrap_or_else(||{Operation {op: None, args: vec![], orig: line.parse().unwrap(), line: index+1 }}));
    }
    let mut final_ops: Vec<Box<Operation>> = vec![];
    for o in ops {
        final_ops.push(Box::from(Operation { op: o.op, args: o.args, orig: o.orig, line: o.line }));
    };
    final_ops
}

fn check(operation_type: OperationType, line: &str, re: Regex, index: usize) -> Option<Operation> {
    let line = &*Regex::new(r"^\s+").unwrap().replace_all(line, "");
    let line = &*Regex::new(r"\s+$").unwrap().replace_all(line, "");
    let line = &*Regex::new(r"\s+").unwrap().replace_all(line, " ");
    if re.is_match(line) {
        let args = re.captures(line)?;
        let mut vec: Vec<String> = vec![];
        for i in args.iter().skip(1) {
            vec.push(i.unwrap().as_str().parse().unwrap());
        }
        println!("[{:>3}] {} : {} [{}]", index, line, operation_type, vec.join(", "));
        return Some(Operation { op: operation_type, args: vec, orig: line.parse().unwrap(), line: index });
    }
    Option::None
}

#[derive(std::cmp::PartialEq, Copy, Clone)]
pub enum OutputType { IntelAsm, Dynamic }
pub(crate) trait OutputWriter {
    fn generate(operation: Operation) -> Vec<u8>;
}

#[allow(unused)]
pub fn write(file: &str, ops: Vec<Operation>, output_type: OutputType) {
    let mut file = File::create(file).unwrap();
    let mut current_writer: fn(Operation) -> Vec<u8> = IntelAsmWriter::generate;
    for op in ops {
        if matches!(op.op, SetWriter) && op.args.len() > 1 {
            current_writer = match &*op.args[0] {
                "intel_asm" => IntelAsmWriter::generate,
                _ => |op| { vec![] }
            };
            println!("    Set writer to: {}", &*op.args[0]);
        }
        let buf_gen = match output_type {
            IntelAsm => IntelAsmWriter::generate,
            _ => current_writer
        };
        file.write(&*buf_gen(op));
    }
    file.flush().unwrap();
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
            RunInterrupt => "INTERRUPT "
        })
    }
}
