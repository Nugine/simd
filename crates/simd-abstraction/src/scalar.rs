#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct Bytes16(pub [u8; 16]);

#[derive(Debug, Clone)]
#[repr(C, align(32))]
pub struct Bytes32(pub [u8; 32]);
