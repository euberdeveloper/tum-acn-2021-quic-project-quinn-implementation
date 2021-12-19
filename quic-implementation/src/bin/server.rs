use futures::executor::block_on;

fn main() {
    let future = quic_implementation::server::run_server();
    match block_on(future) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    };
}
