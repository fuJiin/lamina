#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// Include all the test modules
mod ffi;
mod ffi_integration;
mod libraries;
mod primitives;
mod procedures;
mod r7rs_core;
mod special_forms;

// Include the Huff compiler tests
#[cfg(test)]
mod backends {
    #[cfg(test)]
    mod huff {
        mod compiler_test;
    }
} 