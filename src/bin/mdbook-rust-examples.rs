use std::io;
use std::process;

use embedded_rust_tutorial_validation::preprocessor::RustExamplesPreprocessor;

fn main() {
    let mut preprocessor = RustExamplesPreprocessor::new();
    
    if let Err(e) = preprocessor.run() {
        eprintln!("Error running rust-examples preprocessor: {}", e);
        process::exit(1);
    }
}