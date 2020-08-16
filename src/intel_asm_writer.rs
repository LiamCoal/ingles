use crate::operation::{OutputWriter, Operation};
use crate::operation::OperationType::*;

pub struct IntelAsmWriter {}

impl OutputWriter for IntelAsmWriter {
    fn generate<'a>(operation: Operation) -> Vec<u8> {
        let rs = match operation.op {
            Move => format!("     mov {}, {}", operation.args[0], operation.args[1]),
            FunctionStart => format!("{}:", operation.args[0]),
            FunctionEnd => format!("     ret"),
            SetStartFunction => format!("  global _start \n _start: jmp {}", operation.args[0]),
            FunctionEnter => format!("    call {}", operation.args[0]),
            Add => format!("     add {}, {}", operation.args[0], operation.args[1]),
            Subtract => format!("     sub {}, {}", operation.args[0], operation.args[1]),
            Multiply => format!("     mul {}, {}", operation.args[0], operation.args[1]),
            Divide => format!("     div {}, {}", operation.args[0], operation.args[1]),
            TextSection => format!(" section .text"),
            DataSection => format!(" section .data"),
            UninitializedDataSection => format!(" section .bss"),
            ExitProcess => format!("     mov rax, 60\n     mov rdi, {}\n syscall", operation.args[0]),
            SetWriter | None => "".to_string(),
            PushToStack => format!("    push {}", operation.args[0]),
            PopFromStack => format!("     pop {}", operation.args[0]),
            ReadFromMemory => format!("     mov {}, [{}]", operation.args[1], operation.args[0]),
            WriteToMemory => format!("     mov [{}], {}", operation.args[1], operation.args[0]),
            DefineVariable => format!("{}: d{} {}", operation.args[0], operation.args[1], operation.args[2]),
            DefineUninitVariable => format!("{}: res{} 1", operation.args[0], operation.args[1]),
            RunInterrupt => format!("     int {}h", operation.args[0])
        };
        let mut r = String::new();
        let spl = rs.split('\n').collect::<Vec<&str>>();
        if !rs.is_empty() {
            for i in 0..(&*spl).len() {
                r += &*format!("{:<60} ; [{:>3}] {}\n", spl[i], operation.line, operation.orig);
            }
        }
        r.into_bytes()
    }
}