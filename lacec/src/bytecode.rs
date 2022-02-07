use crate::common::*;

/* Serializes a lace program to pure bytes, compresses it and returns. */
fn serialize(code: &(Vec<Value>, Vec<Instruction>)) -> Vec<u8> {
    let bytes = bincode::serialize(code).unwrap();
    bytes
}
