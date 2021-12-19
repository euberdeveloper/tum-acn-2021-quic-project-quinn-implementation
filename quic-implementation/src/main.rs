use quic_implementation::env_parser::QuicImplementationConfig;

fn main() {
    let config = QuicImplementationConfig::new();
    println!("{:#?}", config);
}