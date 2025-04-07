use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub bindings: std::collections::HashMap<String, Value>,
}

#[allow(dead_code)]
impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            parent: None,
            bindings: std::collections::HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<Value> {
        self.bindings
            .get(key)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().get(key)))
    }

    #[allow(dead_code)]
    pub fn set(&mut self, key: String, value: Value) {
        self.bindings.insert(key, value);
    }
}

// Define a record type structure
#[derive(Clone)]
pub struct RecordType {
    pub name: String,
    pub fields: Vec<(String, bool)>, // (field_name, mutable)
}

// Define a record instance structure
#[derive(Clone)]
pub struct Record {
    pub type_info: Rc<RecordType>,
    pub values: RefCell<std::collections::HashMap<String, Value>>,
}

// Define a library structure
#[derive(Clone)]
pub struct Library {
    pub name: Vec<String>,    // Library name (e.g., (scheme base))
    pub exports: Vec<String>, // List of exported symbols
    #[allow(dead_code)]
    pub imports: Vec<Vec<String>>, // List of imported libraries
    pub environment: Rc<RefCell<Environment>>, // Library's environment
}

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(NumberKind),
    Character(char),
    String(String),
    Symbol(String),
    Pair(Rc<(Value, Value)>),
    #[allow(dead_code)]
    Vector(Rc<Vec<Value>>),
    Procedure(Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>),
    #[allow(dead_code)]
    Environment(Rc<RefCell<Environment>>),
    // Add Record types
    RecordType(Rc<RecordType>),
    Record(Rc<Record>),
    // Add Bytevector
    Bytevector(Rc<RefCell<Vec<u8>>>),
    // Add Library
    Library(Rc<RefCell<Library>>),
    // Add RustFn to represent foreign Rust functions
    RustFn(Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>, String),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "Nil"),
            Value::Boolean(b) => write!(f, "Boolean({})", b),
            Value::Number(n) => write!(f, "Number({:?})", n),
            Value::Character(c) => write!(f, "Character({})", c),
            Value::String(s) => write!(f, "String({})", s),
            Value::Symbol(s) => write!(f, "Symbol({})", s),
            Value::Pair(p) => write!(f, "Pair({:?}, {:?})", p.0, p.1),
            Value::Vector(v) => write!(f, "Vector({:?})", v),
            Value::Procedure(_) => write!(f, "Procedure"),
            Value::Environment(_) => write!(f, "Environment"),
            Value::RecordType(rt) => write!(f, "RecordType({})", rt.name),
            Value::Record(r) => write!(f, "Record({})", r.type_info.name),
            Value::Bytevector(bytes) => write!(f, "Bytevector({:?})", bytes.borrow()),
            Value::Library(lib) => write!(f, "Library({:?})", lib.borrow().name),
            Value::RustFn(_, name) => write!(f, "RustFn({})", name),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NumberKind {
    Integer(i64),
    Real(f64),
    #[allow(dead_code)]
    Rational(i64, i64),
}

impl NumberKind {
    pub fn as_f64(&self) -> f64 {
        match self {
            NumberKind::Integer(i) => *i as f64,
            NumberKind::Real(r) => *r,
            NumberKind::Rational(n, d) => *n as f64 / *d as f64,
        }
    }
}

// Implement From trait for Value
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(NumberKind::Real(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Number(NumberKind::Integer(value))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => match n {
                NumberKind::Integer(i) => write!(f, "{}", i),
                NumberKind::Real(r) => {
                    if r.fract() == 0.0 {
                        write!(f, "{}.0", r)
                    } else {
                        write!(f, "{}", r)
                    }
                }
                NumberKind::Rational(num, den) => write!(f, "{}/{}", num, den),
            },
            Value::Symbol(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => {
                if *b {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Value::Character(c) => write!(f, "#\\{}", c),
            Value::Nil => write!(f, "()"),
            Value::Pair(_p) => {
                let mut current = self;
                let mut is_first = true;
                write!(f, "(")?;
                loop {
                    match current {
                        Value::Pair(pair) => {
                            if !is_first {
                                write!(f, " ")?;
                            }
                            write!(f, "{}", pair.0)?;
                            current = &pair.1;
                            is_first = false;
                        }
                        Value::Nil => break,
                        _ => {
                            write!(f, " . {}", current)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
            Value::Procedure(_) => write!(f, "#<procedure>"),
            Value::Library(lib) => {
                let name = &lib.borrow().name;
                write!(f, "#<library:{}>", name.join(" "))
            }
            Value::RecordType(rt) => {
                write!(f, "#<record-type:{}>", rt.name)
            }
            Value::Record(r) => {
                write!(f, "#<{}>", r.type_info.name)
            }
            Value::Bytevector(bv) => {
                let bytes = bv.borrow();
                write!(f, "#u8(")?;
                for (i, byte) in bytes.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", byte)?;
                }
                write!(f, ")")
            }
            Value::Vector(v) => {
                write!(f, "#(")?;
                for (i, val) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, ")")
            }
            Value::Environment(_) => write!(f, "#<environment>"),
            Value::RustFn(_, name) => write!(f, "#<rust-function:{}>", name),
        }
    }
}

// Manual implementation of PartialEq for Value
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Pair(a), Value::Pair(b)) => {
                // Compare car and cdr
                a.0 == b.0 && a.1 == b.1
            }
            (Value::Vector(a), Value::Vector(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                a.iter().zip(b.iter()).all(|(x, y)| x == y)
            }
            // Procedures are never equal
            (Value::Procedure(_), Value::Procedure(_)) => false,
            // For environments, record types, records, bytevectors, and libraries,
            // compare by reference identity
            (Value::Environment(a), Value::Environment(b)) => Rc::ptr_eq(a, b),
            (Value::RecordType(a), Value::RecordType(b)) => Rc::ptr_eq(a, b),
            (Value::Record(a), Value::Record(b)) => Rc::ptr_eq(a, b),
            (Value::Bytevector(a), Value::Bytevector(b)) => Rc::ptr_eq(a, b),
            (Value::Library(a), Value::Library(b)) => Rc::ptr_eq(a, b),
            // Different variants are never equal
            _ => false,
        }
    }
}
