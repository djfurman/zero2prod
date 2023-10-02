use std::net::TcpListener;
use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener =
        TcpListener::bind("127.0.0.1:8000").expect("Could not bind application run to port 8000");

    run(listener)?.await
}
