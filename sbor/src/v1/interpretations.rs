pub struct DefaultInterpretations {
}

/// These are the default interpretations in base-Sbor
/// Specific variants may introduce more
impl DefaultInterpretations {
    // A placeholder interpretation meaning "not fixed"
    pub const NOT_FIXED: u8 = 0x00;

    // Misc raw bytes
    pub const BOOLEAN: u8 = 0x01;
    pub const UTF8_STRING: u8 = 0x01;
    pub const UTF8_STRING_DISCRIMINATOR: u8 = 0x02;
    pub const SBOR_ANY: u8 = 0x03;
    pub const PLAIN_RAW_BYTES: u8 = 0x04;

    // Unsigned Integers
    pub const U8: u8 = 0x10;
    pub const U16: u8 = 0x11;
    pub const U32: u8 = 0x12;
    pub const U64: u8 = 0x13;
    pub const U128: u8 = 0x14;
    pub const U256: u8 = 0x15;
    pub const USIZE: u8 = 0x1a;

    // Signed Integers
    pub const I8: u8 = 0x20;
    pub const I16: u8 = 0x21;
    pub const I32: u8 = 0x22;
    pub const I64: u8 = 0x23;
    pub const I128: u8 = 0x24;
    pub const I256: u8 = 0x25;
    pub const ISIZE: u8 = 0x1b;

    // Product type interpretations
    pub const UNIT: u8 = 0x30;
    pub const TUPLE: u8 = 0x31;
    pub const STRUCT: u8 = 0x32;
    pub const ENUM_VARIANT: u8 = 0x33;

    // Sum types
    pub const ENUM: u8 = 0x40;
    pub const RESULT: u8 = 0x41;
    pub const OPTION: u8 = 0x42;

    // List types
    pub const NORMAL_LIST: u8 = 0x50;
    pub const FIXED_LENGTH_ARRAY: u8 = 0x52;
    /// The map defines no particular ordering of values
    pub const UNORDERED_SET: u8 = 0x5a;
    /// The map defines a particular ordering of keys (eg insertion order), respected by the serialization
    pub const ORDERED_SET: u8 = 0x5b;
    /// The map denotes that the keys are sorted by some ordering on the value space
    pub const SORTED_SET: u8 = 0x5c;

    // Map types
    /// The map defines no particular ordering of keys
    pub const UNORDERED_MAP: u8 = 0x6a;
    /// The map defines a particular ordering of keys (eg insertion order), respected by the serialization
    pub const ORDERED_MAP: u8 = 0x6b;
    /// The map denotes that the keys are sorted by some ordering on the key space
    pub const SORTED_MAP: u8 = 0x6c;
}