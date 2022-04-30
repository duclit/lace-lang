use crate::{
    lir::{HlvmCallFrame, HlvmInstruction, HlvmValue},
    traits::*, dev::{hlvm_print, hlvm_exit},
};
use hashbrown::HashMap;

pub struct HighLevelVirtualMachine {
    pub stack: Vec<HlvmValue>,
    pub call_stack: Vec<HlvmCallFrame>,
}

impl HighLevelVirtualMachine {
    /// Instantiate a new HighLevelVirtualMachine.
    pub fn new(local_prealloc: Option<usize>) -> HighLevelVirtualMachine {
        let mut call_stack = Vec::with_capacity(8);

        /* Push the main frame to the call stack */
        call_stack.push(HlvmCallFrame {
            locals: HashMap::with_capacity(local_prealloc.unwrap_or(8)),
        });

        HighLevelVirtualMachine {
            stack: Vec::with_capacity(8),
            call_stack,
        }
    }

    #[inline(always)]
    fn get_global_scope(&self) -> &HashMap<String, HlvmValue> {
        &self
            .call_stack
            .first()
            .expect("Unable to get global scope; Call stack is empty.")
            .locals
    }

    #[inline(always)]
    fn get_mut_global_scope(&mut self) -> &mut HashMap<String, HlvmValue> {
        &mut self
            .call_stack
            .first_mut()
            .expect("Unable to get global scope; Call stack is empty.")
            .locals
    }

    #[inline(always)]
    fn get_local_scope(&self) -> &HashMap<String, HlvmValue> {
        &self
            .call_stack
            .last()
            .expect("Unable to get local scope; Call stack is empty.")
            .locals
    }

    #[inline(always)]
    fn get_mut_local_scope(&mut self) -> &mut HashMap<String, HlvmValue> {
        &mut self
            .call_stack
            .last_mut()
            .expect("Unable to get local scope; Call stack is empty.")
            .locals
    }

    /// Main entry point of the VM.
    /// Returns a `Result::Err` if an error occurs, with an appropriate error message.
    pub fn execute(&mut self, instructions: &[HlvmInstruction]) -> Result<HlvmValue, String> {
        use HlvmInstruction::*;
        let mut ip: usize = 0;

        loop {
            if ip >= instructions.len() {
                break;
            }
            
            let instruction = &instructions[ip];

            match instruction {
                /* Setting and getting globals and locals */
                SetGlobal(name) => {
                    let top = self.stack.pop().unwrap();
                    self.get_mut_global_scope().insert(name.to_string(), top);
                }
                SetLocal(name) => {
                    let top = self.stack.pop().unwrap();
                    self.get_mut_local_scope().insert(name.to_string(), top);
                }

                GetGlobal(name) => self
                    .stack
                    .push(self.get_global_scope().get(name).unwrap().clone()),
                GetLocal(name) => self
                    .stack
                    .push(self.get_local_scope().get(name).unwrap().clone()),

                /* Returning values */
                ReturnValue => return Ok(self.stack.pop().unwrap()),
                Return => return Result::Ok(HlvmValue::Number(0.0)),

                Push(val) => self.stack.push(val.clone()),
                Call => {
                    let function = self.stack.pop().unwrap();

                    match function.call(self) {
                        Ok(val) => self.stack.push(val),
                        Err(err) => return Err(err),
                    }
                }
                CallPrimitive(index, args) => {
                    let mut arguments = Vec::with_capacity(*args);

                    for _ in 0..*args {
                        arguments.push(self.stack.pop().unwrap());
                    }

                    let value = match index {
                        0 => hlvm_print(arguments),
                        1 => hlvm_exit(arguments),
                        _ => panic!("Invalid primitive function")
                    };

                    self.stack.push(value);
                }

                Add | Subtract | Multiply | Divide | Equal | NotEqual | GreaterThan | LessThan
                | GreaterThanOrEqual | LessThanOrEqual | And | Or => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match instruction {
                        HlvmInstruction::Add => self.stack.push(left.add(right)),
                        HlvmInstruction::Subtract => self.stack.push(left.sub(right)),
                        HlvmInstruction::Multiply => self.stack.push(left.mul(right)),
                        HlvmInstruction::Divide => self.stack.push(left.div(right)),
                        HlvmInstruction::Equal => self.stack.push(left._eq(right)),
                        HlvmInstruction::NotEqual => self.stack.push(left._ne(right)),
                        HlvmInstruction::GreaterThan => self.stack.push(left.gt(right)),
                        HlvmInstruction::LessThan => self.stack.push(left.lt(right)),
                        HlvmInstruction::GreaterThanOrEqual => self.stack.push(left.ge(right)),
                        HlvmInstruction::LessThanOrEqual => self.stack.push(left.le(right)),
                        HlvmInstruction::And => self.stack.push(left.and(right)),
                        HlvmInstruction::Or => self.stack.push(left.or(right)),
                        HlvmInstruction::Not => self.stack.push(left.not()),
                        _ => panic!("The universe should've collapsed by now."),
                    }
                }

                Not => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(value.not());
                }

                Jump(addr) => {
                    ip = *addr;
                    continue;
                }

                JumpIf(addr) => {
                    if self.stack.pop().unwrap().is_truthy() {
                        ip = *addr;
                        continue;
                    }
                }

                Instantiate => {
                    let obj = self.stack.pop().unwrap();

                    match obj.initialize(&mut self.stack) {
                        Ok(obj) => self.stack.push(obj),
                        Err(err) => return Result::Err(err),
                    };
                }

                GetAttribute(attr) => {
                    let obj = self.stack.pop().unwrap();

                    self.stack.push(match obj.get(attr.to_string()) {
                        Ok(val) => val,
                        Err(err) => return Result::Err(err),
                    });
                }

                SetAttribute(attr) => {
                    let val = self.stack.pop().unwrap();
                    let obj = self.stack.last_mut().unwrap();

                    match obj.set(attr.to_string(), val) {
                        Ok(val) => val,
                        Err(err) => return Result::Err(err),
                    }
                }

                unimplemented => todo!("{:?}", unimplemented)
            }

            ip += 1;
        }

        Result::Ok(HlvmValue::Number(0.0))
    }
}
