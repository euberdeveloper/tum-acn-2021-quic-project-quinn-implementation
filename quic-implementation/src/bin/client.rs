use futures::executor::block_on;

#[tokio::main]
async fn main() {
    let future = quic_implementation::client::run_client();
    match block_on(future) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    };
}
