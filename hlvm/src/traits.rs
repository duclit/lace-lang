use crate::{lir::HlvmValue, vm::HighLevelVirtualMachine};

/// Used to define how operations behave in the VM.
pub trait Operation<T> {
    fn add(&self, b: T) -> T;
    fn sub(&self, b: T) -> T;
    fn mul(&self, b: T) -> T;
    fn div(&self, b: T) -> T;
    /// Equality
    fn _eq(&self, b: T) -> T;
    /// Inequality
    fn _ne(&self, b: T) -> T;
    /// Greater than
    fn gt(&self, b: T) -> T;
    /// Less than
    fn lt(&self, b: T) -> T;
    /// Greater than or equal to
    fn ge(&self, b: T) -> T;
    /// Less than or equal to
    fn le(&self, b: T) -> T;
    /// Logical or
    fn or(&self, b: T) -> T;
    /// Logical and
    fn and(&self, b: T) -> T;
    /// Logical not
    fn not(&self) -> T;
}

/// Should be implemented on all types that can be initialized.
pub trait Initializable {
    /// Called when HlvmInstruction::Initialized is executed.
    /// The stack can be used to get any attributes of the value that is to be initialized.
    fn initialize(&self, stack: &mut Vec<HlvmValue>) -> Result<HlvmValue, String>;
}

/// Should be implemented on all types that can have attributes.
pub trait Instance {
    fn get(&self, name: String) -> Result<HlvmValue, String>;
    fn set(&mut self, name: String, value: HlvmValue) -> Result<(), String>;
}

/// Should be implemented on all types that can be called.
pub trait Callable {
    fn call(&self, vm: &mut HighLevelVirtualMachine) -> Result<HlvmValue, String>;
}
