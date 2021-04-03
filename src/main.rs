use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to port 8080");

    println!("listening");
    loop {
        let (socket, socket_addr) = listener.accept().await.expect("Failed to accept a connect");
        tokio::spawn(async move {
            if let Err(err) = handle_socket(socket, socket_addr).await {
                eprintln!("an error occured in the handler: {:#?}", err);
            }
        });
    }
}

async fn handle_socket(
    mut socket: TcpStream,
    socket_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let (read, mut write) = socket.split();

    let mut buffered_reader = BufReader::new(read);

    // I don't have to give a capcity for this. read_line will resize the string if the
    // data we read is more. But it's good to give it an initial capcity so that it doesn't
    // have to resize on the first run
    let mut line = String::with_capacity(8 * 1024);

    write
        .write(
            format!(
                "Hello! You are connected from this socket address: {}\n",
                socket_addr
            )
            .as_bytes(),
        )
        .await?;

    loop {
        match tokio::time::timeout(Duration::from_secs(5), buffered_reader.read_line(&mut line))
            .await
        {
            // This is a timeout error
            Err(_) => {
                println!(
                    "Dropping socket connect from {}, reason: timed out after 5 seconds",
                    socket_addr.port()
                );
                return Ok(());
            }
            Ok(val) => val,
        }?;

        let trimmed = line.trim();

        println!("[{}]: '{}'", socket_addr.port(), trimmed);

        let response = match trimmed {
            "Hi" => "Hello",
            "Hang up" => "No u hang up",
            "Bye" => {
                println!("Closing socket at: {}", socket_addr.port());
                return Ok(());
            }
            _ => "wtf u on about bro",
        };

        write.write(format!("{}\n", response).as_bytes()).await?;
        // Clear the buffer
        line.clear();
    }
}
