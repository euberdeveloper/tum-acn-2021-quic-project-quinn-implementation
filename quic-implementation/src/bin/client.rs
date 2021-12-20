use futures::executor::block_on;

#[tokio::main]
async fn main() {
    println!("begin");
    let future = quic_implementation::client::run_client();
    println!("tattaratta");
    match block_on(future) {
        Ok(_) => {
            println!("tutto ok");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    };
    println!("finisheeed");
}
