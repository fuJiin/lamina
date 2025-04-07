use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread_local;

use crate::value::Library;

// A global registry to track all defined libraries
thread_local! {
    pub static LIBRARIES: RefCell<HashMap<Vec<String>, Rc<RefCell<Library>>>> = RefCell::new(HashMap::new());
}

// Function to get a library by name
pub fn get_library(name: &[String]) -> Option<Rc<RefCell<Library>>> {
    LIBRARIES.with(|libraries| libraries.borrow().get(&name.to_vec()).cloned())
}

// Function to register a library
pub fn register_library(library: Rc<RefCell<Library>>) {
    let name = library.borrow().name.clone();
    LIBRARIES.with(|libraries| {
        libraries.borrow_mut().insert(name, library);
    });
}
