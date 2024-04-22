use std::{
    collections::HashMap, fmt::Debug, fmt::Display, hash::DefaultHasher, hash::Hash, hash::Hasher,
};

use once_cell::sync::Lazy;

// this map is used for Identifier visualization
static mut BACKWARDS_MAP: Lazy<HashMap<u64, String>> = Lazy::new(HashMap::new);

#[derive(Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Identifier {
    // holds hash for fast comparison and copy
    hashed_ident: u64,
}

#[derive(Hash, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct OperatorIdentifier {
    pub op: Identifier,
    pub left: Identifier,
    pub right: Identifier,
}

impl Identifier {
    pub fn new(ident: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        ident.hash(&mut hasher);
        let hash = hasher.finish();

        unsafe {
            // oh no, I am such a bad boy))))
            BACKWARDS_MAP
                .entry(hash)
                .or_insert_with(|| ident.to_string());
        }

        Self { hashed_ident: hash }
    }
}

impl OperatorIdentifier {
    pub fn new(op: Identifier, left: Identifier, right: Identifier) -> Self {
        Self { op, left, right }
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe {
            BACKWARDS_MAP.get(&self.hashed_ident).unwrap()
        })
    }
}

impl Debug for OperatorIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operator({} {} {})", self.left, self.op, self.right)
    }
}

impl Identifier {
    // builtin types
    pub fn for_nah() -> Self {
        Self::new("Nah")
    }

    pub fn for_number() -> Self {
        Self::new("Number")
    }

    pub fn for_bool() -> Self {
        Self::new("Bool")
    }

    pub fn for_string() -> Self {
        Self::new("String")
    }

    pub fn for_function() -> Self {
        Self::new("Function")
    }

    pub fn for_struct_type() -> Self {
        Self::new("StructType")
    }

    pub fn for_native_object() -> Self {
        Self::new("NativeObject")
    }

    // builtin operators
    pub fn for_plus() -> Self {
        Self::new("+")
    }

    pub fn for_minus() -> Self {
        Self::new("-")
    }

    pub fn for_multiply() -> Self {
        Self::new("*")
    }

    pub fn for_divide() -> Self {
        Self::new("/")
    }

    pub fn for_mod() -> Self {
        Self::new("%")
    }

    pub fn for_pow() -> Self {
        Self::new("**")
    }

    pub fn for_and() -> Self {
        Self::new("&&")
    }

    pub fn for_or() -> Self {
        Self::new("||")
    }

    pub fn for_combine() -> Self {
        Self::new("<>")
    }

    // builtin operators (comparison)
    pub fn for_less() -> Self {
        Self::new("<")
    }

    pub fn for_less_eq() -> Self {
        Self::new("<=")
    }

    pub fn for_greater() -> Self {
        Self::new(">")
    }

    pub fn for_greater_eq() -> Self {
        Self::new(">=")
    }

    pub fn for_eq() -> Self {
        Self::new("==")
    }

    pub fn for_not_eq() -> Self {
        Self::new("!=")
    }
}