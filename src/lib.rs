// Export the main modules
pub mod error;
pub mod evaluator;
pub mod lexer;
pub mod parser;
pub mod value;
pub mod ffi;
pub mod embed;

use std::cell::RefCell;
use std::rc::Rc;

// Global environment setup
thread_local! {
    // Initialize with an environment directly
    pub static GLOBAL_ENV: RefCell<Rc<RefCell<crate::value::Environment>>> = {
        let env = crate::evaluator::environment::setup_initial_env();
        RefCell::new(env)
    };
}

pub fn execute(code: &str) -> Result<String, String> {
    // Get the global environment
    let env = GLOBAL_ENV.with(|global_env| global_env.borrow().clone());

    let tokens = match crate::lexer::lex(code) {
        Ok(tokens) => tokens,
        Err(err) => return Err(err.to_string()),
    };

    let parsed = match crate::parser::parse(&tokens) {
        Ok(expr) => expr,
        Err(err) => return Err(err.to_string()),
    };

    match crate::evaluator::eval_with_env(parsed, env) {
        Ok(result) => {
            // Special case: Nil (empty list) should display as an empty string
            if let crate::value::Value::Nil = result {
                Ok("".to_string())
            } else {
                Ok(result.to_string())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}
