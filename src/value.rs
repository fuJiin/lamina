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