/* --------------------------------------------------------------
Contains all the default implementations for traits that can be used to customize the vm
-------------------------------------------------------------- */

use std::fmt::{Display, Formatter};

use crate::{
    lir::{HlvmCallFrame, HlvmValue},
    traits::*,
    vm::HighLevelVirtualMachine,
};
use hashbrown::HashMap;

/* 0 */ pub fn hlvm_print(var: Vec<HlvmValue>) -> HlvmValue {
    println!("{}", var[0]);
    HlvmValue::Number(0.0)
}

/* 1 */ pub fn hlvm_exit(_: Vec<HlvmValue>) -> HlvmValue {
    std::process::exit(0)
}

impl Initializable for HlvmValue {
    fn initialize(&self, stack: &mut Vec<HlvmValue>) -> Result<HlvmValue, String> {
        match *self {
            HlvmValue::StructBlueprint(ref attributes) => Ok(HlvmValue::StructInstance(attributes.clone())),
            _ => Err(format!("Cannot initialize value {:?}", self)),
        }
    }
}

impl Instance for HlvmValue {
    fn get(&self, name: String) -> Result<HlvmValue, String> {
        match *self {
            HlvmValue::StructInstance(ref values) => Ok(match values.get(&name) {
                Some(val) => val.clone(),
                None => return Err(format!("Undefined parameter {}", name)),
            }),
            _ => Err(format!("Cannot get attribute value {:?}", self)),
        }
    }

    fn set(&mut self, name: String, value: HlvmValue) -> Result<(), String> {
        match *self {
            HlvmValue::StructInstance(ref mut values) => {
                values.insert(name, value);
                Ok(())
            }
            _ => Err(format!("Cannot set value {:?}", self)),
        }
    }
}

impl Callable for HlvmValue {
    fn call(&self, vm: &mut HighLevelVirtualMachine) -> Result<HlvmValue, String> {
        match self {
            HlvmValue::Function(instructions, args, loc_prealloc) => {
                vm.call_stack.push(HlvmCallFrame {
                    locals: HashMap::with_capacity(loc_prealloc.unwrap_or(8)),
                });

                /* Push all of the arguments to the function's local scope */
                for iden in args {
                    let argument = vm.stack.pop().unwrap();
                    vm.call_stack
                        .last_mut()
                        .unwrap()
                        .locals
                        .insert(iden.to_string(), argument);
                }

                let result = vm.execute(&instructions);
                vm.call_stack.pop();

                /* Push the result to the stack, return if an error occured */
                match result {
                    Ok(val) => Ok(val),
                    Err(err) => return Result::Err(err),
                }
            }
            _ => Err(format!("Cannot call value {:?}", self)),
        }
    }
}

impl Operation<HlvmValue> for HlvmValue {
    fn add(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Number(*a + b),
            (HlvmValue::String(a), HlvmValue::String(_b)) => {
                let mut a = a.clone();
                a.push_str(stringify!(_b));
                HlvmValue::String(a.to_string())
            }
            _ => panic!("Unable to add values of different types"),
        }
    }

    fn sub(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Number(*a - b),
            _ => panic!("Unable to subtract values of different types"),
        }
    }

    fn mul(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Number(*a * b),
            (HlvmValue::String(a), HlvmValue::Number(b)) => {
                todo!()
            }
            _ => panic!("Unable to multiply values of different types"),
        }
    }

    fn div(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Number(*a / b),
            _ => panic!("Unable to divide values of different types"),
        }
    }

    fn _eq(&self, b: HlvmValue) -> HlvmValue {
        HlvmValue::Bool(self == &b)
    }

    fn _ne(&self, b: HlvmValue) -> HlvmValue {
        HlvmValue::Bool(self != &b)
    }

    fn gt(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Bool(*a > b),
            _ => panic!("Unable to compare values of different types"),
        }
    }

    fn lt(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Bool(*a < b),
            _ => panic!("Unable to compare values of different types"),
        }
    }

    fn ge(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Bool(*a >= b),
            _ => panic!("Unable to compare values of different types"),
        }
    }

    fn le(&self, b: HlvmValue) -> HlvmValue {
        match (self, b) {
            (HlvmValue::Number(a), HlvmValue::Number(b)) => HlvmValue::Bool(*a <= b),
            _ => panic!("Unable to compare values of different types"),
        }
    }

    fn and(&self, b: HlvmValue) -> HlvmValue {
        HlvmValue::Bool(self.is_truthy() && b.is_truthy())
    }

    fn or(&self, b: HlvmValue) -> HlvmValue {
        HlvmValue::Bool(self.is_truthy() || b.is_truthy())
    }

    fn not(&self) -> HlvmValue {
        HlvmValue::Bool(!self.is_truthy())
    }
}

impl Display for HlvmValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            HlvmValue::Bool(true) => write!(f, "true"),
            HlvmValue::Bool(false) => write!(f, "false"),
            HlvmValue::Number(a) => write!(f, "{}", a),
            HlvmValue::String(a) => write!(f, "{}", a),
            HlvmValue::BuiltInFunction(..) => write!(f, "<rust-function>"),
            HlvmValue::Function(..) => write!(f, "<hlvm-function>"),
            HlvmValue::StructBlueprint(..) => write!(f, "<struct-blueprint>"),
            HlvmValue::StructInstance(..) => write!(f, "<struct-instance>"),
        }
    }
}
