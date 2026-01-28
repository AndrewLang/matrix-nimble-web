use clap::Parser;
use nimble_web::cli::{Cli, Generator};

fn main() {
    let cli = Cli::parse();
    let generator = Generator::new(cli);
    
    if let Err(e) = generator.generate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
