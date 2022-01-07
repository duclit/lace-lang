use crate::opcode::CodeObject;

use bincode;

pub fn serialize(code: CodeObject) -> Vec<u8> {
    bincode::serialize(&code).unwrap()
}

pub fn deserialize(bytes: Vec<u8>) -> CodeObject {
    bincode::deserialize(&bytes[..]).unwrap()
}
