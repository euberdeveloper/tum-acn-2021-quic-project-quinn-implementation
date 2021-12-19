use futures::executor::block_on;

fn main() {
    let future = quic_implementation::client::run_client();
    match block_on(future) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    };
}
