use crate::lir::{Arguments, HlvmInstruction, HlvmValue, LocalPreAlloc};
use hashbrown::HashMap;

type CodeBlock = Vec<HlvmHirInstruction>;
type Expression = Vec<HlvmHirInstruction>;
pub(crate) type Module = HashMap<String, HlvmValue>;

pub enum HlvmHirInstruction {
    Push(HlvmValue),

    /// Pops the value from the stack and calls it.
    /// Top of the stack must be a function, otherwise HLVM will panic.
    /// The arguments passed into the function must be in reversed order.
    Call,
    CallPrimitive(usize, usize),

    /// Returns HlvmValue::Int32(0)
    Return,
    /// Returns the value at the top of the stack
    ReturnValue,

    Get(String),
    GetLocal(String),
    GetGlobal(String),
    SetLocal(String),
    SetGlobal(String),

    /// Gets the value of attribute `n` of the value at the top of the stack.
    GetAttribute(String),

    /// This is a wierd one because in order to win against the borrow checker,
    /// the value you want to set the attribute on must be loaded before the desired
    /// value of the attribute itself.
    SetAttribute(String),

    /// Instantiate a value. Calls the `instantiate` method defined in the Instantiable trait.
    /// (look at `dev.rs` if you want to view the default implementation or change it)
    ///
    /// Unlike SetAttribute, the object you want to initialize must be on top of the stack,
    /// followed by any attributes.
    Instantiate,

    /// Assigns a `Module` to a variable.
    ///
    /// * `LoadModule` expands to `Push(HlvmValue::StructInstance(module.1)), SetGlobal(module.0), GetGlobal(module.0), GetAttribute(module.0, "<hlvm:main>"), Call`.
    /// * The module must contain a key named `<hlvm:main>`. This is the main function, and may be empty. It is called when the module is loaded.
    LoadModule(String, Module),

    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    And,
    Or,
    BinaryAnd,
    BinaryOr,
    Not,
    Negate,
    Typeof,

    IfStatement {
        /// The code to execute if the value on top of the stack is truthy
        ontrue: CodeBlock,

        /// The first element of each element in this vec is the expression that determines
        /// whether the respective branch should be ran. The second element is the code to execute.
        onelseif: Option<Vec<(CodeBlock, CodeBlock)>>,

        /// If all other conditions are false, execute this code
        onfalse: CodeBlock,
    },

    /// Note: HLVM does not provide for loops, and all for loops in your program must be
    /// disembodied into while statements.
    ///
    /// * Expression -> The expression to evaluate at on every iteration.
    /// * CodeBlock -> The block of code to execute if the value on top of the stack is truthy
    WhileStatement(Expression, CodeBlock),
}

/// Converts HIR (High \[Level] Intermediate Representation) to LIR (Low \[Level] Intermediate Representation),
/// which can be understood by the HLVM.
pub fn from_hir(source: Vec<HlvmHirInstruction>) -> Vec<HlvmInstruction> {
    let mut instructions = vec![];

    for instruction in source {
        match instruction {
            HlvmHirInstruction::Push(value) => instructions.push(HlvmInstruction::Push(value)),
            HlvmHirInstruction::Call => instructions.push(HlvmInstruction::Call),
            HlvmHirInstruction::CallPrimitive(index, args) => instructions.push(HlvmInstruction::CallPrimitive(index, args)),
            HlvmHirInstruction::Return => instructions.push(HlvmInstruction::Return),
            HlvmHirInstruction::ReturnValue => instructions.push(HlvmInstruction::ReturnValue),
            HlvmHirInstruction::GetLocal(name) => {
                instructions.push(HlvmInstruction::GetLocal(name))
            }
            HlvmHirInstruction::GetGlobal(name) => {
                instructions.push(HlvmInstruction::GetGlobal(name))
            }
            HlvmHirInstruction::Get(name) => {
                instructions.push(HlvmInstruction::Get(name))
            }
            HlvmHirInstruction::SetLocal(name) => {
                instructions.push(HlvmInstruction::SetLocal(name))
            }
            HlvmHirInstruction::SetGlobal(name) => {
                instructions.push(HlvmInstruction::SetGlobal(name))
            }
            HlvmHirInstruction::GetAttribute(name) => {
                instructions.push(HlvmInstruction::GetAttribute(name))
            }
            HlvmHirInstruction::SetAttribute(name) => {
                instructions.push(HlvmInstruction::SetAttribute(name))
            }
            HlvmHirInstruction::Instantiate => instructions.push(HlvmInstruction::Instantiate),
            HlvmHirInstruction::Add => instructions.push(HlvmInstruction::Add),
            HlvmHirInstruction::Subtract => instructions.push(HlvmInstruction::Subtract),
            HlvmHirInstruction::Multiply => instructions.push(HlvmInstruction::Multiply),
            HlvmHirInstruction::Divide => instructions.push(HlvmInstruction::Divide),
            HlvmHirInstruction::Equal => instructions.push(HlvmInstruction::Equal),
            HlvmHirInstruction::NotEqual => instructions.push(HlvmInstruction::NotEqual),
            HlvmHirInstruction::GreaterThan => instructions.push(HlvmInstruction::GreaterThan),
            HlvmHirInstruction::GreaterThanOrEqual => {
                instructions.push(HlvmInstruction::GreaterThanOrEqual)
            }
            HlvmHirInstruction::LessThan => instructions.push(HlvmInstruction::LessThan),
            HlvmHirInstruction::LessThanOrEqual => {
                instructions.push(HlvmInstruction::LessThanOrEqual)
            }
            HlvmHirInstruction::And => instructions.push(HlvmInstruction::And),
            HlvmHirInstruction::Or => instructions.push(HlvmInstruction::Or),
            HlvmHirInstruction::BinaryAnd => instructions.push(HlvmInstruction::BinaryAnd),
            HlvmHirInstruction::BinaryOr => instructions.push(HlvmInstruction::BinaryOr),
            HlvmHirInstruction::Not => instructions.push(HlvmInstruction::Not),
            HlvmHirInstruction::Negate => instructions.push(HlvmInstruction::Negate),
            HlvmHirInstruction::Typeof => instructions.push(HlvmInstruction::Typeof),
            HlvmHirInstruction::LoadModule(name, module) => {
                instructions.push(HlvmInstruction::Push(HlvmValue::StructInstance(module)));
                instructions.push(HlvmInstruction::SetGlobal(name.to_string()));
                instructions.push(HlvmInstruction::GetGlobal(name));
                instructions.push(HlvmInstruction::GetAttribute("<hlvm:main>".to_string()));
                instructions.push(HlvmInstruction::Call);
            }
            HlvmHirInstruction::IfStatement { ontrue, onelseif, onfalse } => {
                enum JumpType {
                    Next, 
                    End
                }
                
                let mut jump_offsets: Vec<(usize, JumpType)> = vec![];
                let mut block_offsets: Vec<usize> = vec![];

                instructions.push(HlvmInstruction::Not);
                instructions.push(HlvmInstruction::JumpIf(0)); // NEXT
                jump_offsets.push((instructions.len() - 1, JumpType::Next));
                instructions.append(&mut from_hir(ontrue));
                instructions.push(HlvmInstruction::Jump(0)); // END
                jump_offsets.push((instructions.len() - 1, JumpType::End));
                block_offsets.push(instructions.len());

                if let Some(elseifs) = onelseif {
                    for (condition, code) in elseifs {
                        instructions.append(&mut from_hir(condition));
                        instructions.push(HlvmInstruction::Not);
                        instructions.push(HlvmInstruction::JumpIf(0)); // NEXT
                        jump_offsets.push((instructions.len() - 1, JumpType::Next));
                        instructions.append(&mut from_hir(code));
                        instructions.push(HlvmInstruction::Jump(0)); // END 
                        jump_offsets.push((instructions.len() - 1, JumpType::End));
                        block_offsets.push(instructions.len());
                    }
                }

                println!("{}", block_offsets.len());

                instructions.append(&mut from_hir(onfalse));

                let end_offset = instructions.len();
                let mut index = 0;

                for (offset, jump_type) in jump_offsets {                    
                    match jump_type {
                        JumpType::Next => {
                            instructions[offset] = HlvmInstruction::JumpIf(
                                if index + 1 == block_offsets.len() {
                                    end_offset
                                } else {
                                    println!("{} {:?}", index, block_offsets);
                                    index = index + 1;
                                    block_offsets[index]
                                }
                            );
                        },
                        JumpType::End => {
                            instructions[offset] = HlvmInstruction::Jump(end_offset);
                        }
                    }
                }
            }
            HlvmHirInstruction::WhileStatement(condition, body) => {
                let start_offset = instructions.len();
                instructions.append(&mut from_hir(condition));
                instructions.push(HlvmInstruction::Not);
                let jmpif_offset = instructions.len();
                instructions.push(HlvmInstruction::JumpIf(0)); // END

                instructions.append(&mut from_hir(body));
                instructions.push(HlvmInstruction::Jump(start_offset)); // START
                let end_offset = instructions.len();

                instructions[jmpif_offset] = HlvmInstruction::JumpIf(end_offset);
            }
        }
    }

    return instructions;
}
