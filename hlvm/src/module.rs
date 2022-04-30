use crate::lir::HlvmValue;
use hashbrown::HashMap;

/// Get the default System module
pub fn system() -> HlvmValue {
    HlvmValue::StructInstance(HashMap::from([
        (
            String::from("print"),
            HlvmValue::BuiltInFunction(0, 1),
        ),
        (
            String::from("exit"),
            HlvmValue::BuiltInFunction(1, 0),
        ),
    ]))
}
