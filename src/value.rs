
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

#[derive(Clone)]
#[derive(Clone)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub bindings: std::collections::HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            parent: None,
            bindings: std::collections::HashMap::new(),
        }
    }
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
    Vector(Rc<Vec<Value>>),
    Procedure(Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>),
    Environment(Rc<RefCell<Environment>>),
}

#[derive(Clone)]
pub enum NumberKind {
    Integer(i64),
    Real(f64),
    Rational(i64, i64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "()"),
            Value::Boolean(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Value::Number(n) => match n {
                NumberKind::Integer(i) => write!(f, "{}", i),
                NumberKind::Real(r) => write!(f, "{}", r),
                NumberKind::Rational(n, d) => write!(f, "{}/{}", n, d),
            },
            Value::Character(c) => write!(f, "#\\{}", c),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Pair(p) => write!(f, "({} . {})", p.0, p.1),
            Value::Vector(v) => {
                write!(f, "#(")?;
                for (i, val) in v.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", val)?;
                }
                write!(f, ")")
            },
            Value::Procedure(_) => write!(f, "#<procedure>"),
            Value::Environment(_) => write!(f, "#<environment>"),
        }
    }
}
