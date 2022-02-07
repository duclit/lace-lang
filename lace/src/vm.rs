use crate::*;
use crate::macros::*;

use lacec::scanner::ExtractValue;
use rustc_hash::FxHashMap;

fn runtime_error(error: String) -> ! {
    println!("{}: {}", "runtime_error".red(), error);
    exit(0);
}

#[derive(Debug)]
pub struct CallFrame {
    locals: FxHashMap<String, Value>,
}

pub struct VirtualMachine {
    stack: Vec<Value>,
    constants: Vec<Value>,
    call_stack: Vec<CallFrame>,
}

impl VirtualMachine {
    pub fn new(constants: Vec<Value>) -> VirtualMachine {
        VirtualMachine {
            stack: Vec::with_capacity(256),
            constants,
            call_stack: vec![CallFrame {
                locals: FxHashMap::default(),
            }],
        }
    }

    #[inline(always)]
    fn get_locals(&mut self) -> &mut FxHashMap<String, Value> {
        &mut self.call_stack.last_mut().unwrap().locals
    }

    #[inline(always)]
    fn get_globals(&mut self) -> &FxHashMap<String, Value> {
        &mut self.call_stack.first_mut().unwrap().locals
    }

    pub fn run(&mut self, code: Vec<Instruction>) -> Value {
        let mut ip = 0usize;

        while ip < code.len() {
            let instruction = &code[ip];

            match instruction {
                Instruction::LoadConstant(idx) => self.stack.push(self.constants[*idx].clone()),
                Instruction::LoadVariable(idx) => {
                    let name: &String = &self.constants[*idx].extract();

                    match self.get_locals().get(name) {
                        Some(val) => {
                            let val = val.clone();
                            self.stack.push(val);
                        }
                        None => match self.get_globals().get(name) {
                            Some(val) => {
                                let val = val.clone();
                                self.stack.push(val);
                            }
                            None => runtime_error(format!("variable '{}' not found.", name)),
                        },
                    }
                }
                Instruction::AssignVariable(idx) => {
                    let name = &self.constants[*idx].extract();
                    let val = self.stack.pop().unwrap();

                    self.get_locals().insert(name.to_string(), val);
                }
                Instruction::CallFunction(len) => {
                    let function = self.stack.pop().unwrap();

                    if let Value::Function {
                        code,
                        mut parameters,
                        ..
                    } = function
                    {
                        parameters.reverse();
                        let mut locals: FxHashMap<String, Value> = FxHashMap::default();

                        for parameter in parameters.iter() {
                            locals.insert(parameter.to_string(), self.stack.pop().unwrap());
                        }

                        self.call_stack.push(CallFrame { locals });
                        let ret = self.run(code);

                        self.stack.push(ret);
                    } else {
                        runtime_error(format!(
                            "variable '{}' is not callable.",
                            "<anonymous>".magenta()
                        ))
                    }
                }
                Instruction::CallPrimitiveFunction(len, name) => {
                    let mut arguments: Vec<Value> = vec![];

                    for _ in 0..*len {
                        arguments.push(self.stack.pop().unwrap());
                    }

                    arguments.reverse();

                    let name = self.constants[*name].clone().extract();
                    let name = name.as_str();

                    match name {
                        "writeln!" => lace_writeln(arguments),
                        "exit!" => lace_exit(arguments),
                        _ => runtime_error(format!(
                            "unknown primitive function '{}'.",
                            name.magenta()
                        )),
                    };
                }
                Instruction::Return => {
                    self.call_stack.pop();
                    return self.stack.pop().unwrap();
                }
                Instruction::ReturnNone => {
                    self.call_stack.pop();
                    return Value::None;
                }
                Instruction::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.add(b);
                    self.stack.push(res);
                }
                Instruction::Sub => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.sub(b);
                    self.stack.push(res);
                }
                Instruction::Mul => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.mul(b);
                    self.stack.push(res);
                }
                Instruction::Div => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.div(b);
                    self.stack.push(res);
                }
                Instruction::Mod => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.rem(b);
                    self.stack.push(res);
                }
                Instruction::Pow => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.pow(b);
                    self.stack.push(res);
                }
                Instruction::LeftShift => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.lsh(b);
                    self.stack.push(res);
                }
                Instruction::RightShift => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.rsh(b);
                    self.stack.push(res);
                }
                Instruction::UnEq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.ne(b);
                    self.stack.push(res);
                }
                Instruction::Eq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.eq(b);
                    self.stack.push(res);
                }
                Instruction::Less => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.st(b);
                    self.stack.push(res);
                }
                Instruction::More => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.gt(b);
                    self.stack.push(res);
                }
                Instruction::LessEq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.se(b);
                    self.stack.push(res);
                }
                Instruction::MoreEq => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.ge(b);
                    self.stack.push(res);
                }
                Instruction::BinaryOr => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.bo(b);
                    self.stack.push(res);
                }
                Instruction::BinaryXor => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.bx(b);
                    self.stack.push(res);
                }
                Instruction::BinaryAnd => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let res = a.ba(b);
                    self.stack.push(res);
                }
                Instruction::LogicalNot => {
                    let a = self.stack.pop().unwrap();
                    let res = a.not();
                    self.stack.push(res);
                }
                Instruction::Negate => {
                    let a = self.stack.pop().unwrap();
                    let res = a.negate();
                    self.stack.push(res);
                }
                Instruction::Typeof => {
                    let a = self.stack.pop().unwrap();
                    let res = a.tpyeof();
                    self.stack.push(res);
                }
                Instruction::Jump(idx) => {
                    ip = *idx;
                    continue;
                }
                Instruction::JumpT(idx) => {
                    if self.stack.pop().unwrap().istruthy() {
                        ip = *idx;
                        continue;
                    }
                }
                Instruction::JumpF(idx) => {
                    if !self.stack.pop().unwrap().istruthy() {
                        ip = *idx;
                        continue;
                    }
                }
                Instruction::LogicalAnd => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if a.istruthy() && b.istruthy() {
                        self.stack.push(Value::True)
                    } else {
                        self.stack.push(Value::False)
                    }
                }
                Instruction::LogicalOr => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if a.istruthy() || b.istruthy() {
                        self.stack.push(Value::True)
                    } else {
                        self.stack.push(Value::False)
                    }
                }
            }

            ip += 1;
        }

        self.call_stack.pop();
        Value::None
    }
}
