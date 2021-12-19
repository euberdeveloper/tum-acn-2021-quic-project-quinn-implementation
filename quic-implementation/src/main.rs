use quic_implementation::env_parser::Config;

fn main() {
    let config = Config::new();
    println!("{:#?}", config.logs);
}