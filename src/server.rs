use tokio::net::TcpListener;
use tokio_serde::formats::Bincode;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use serde::{Serialize, Deserialize};
use futures::TryStreamExt;

#[derive(Serialize, Deserialize)]
pub struct Frame {
    content: String,
}

type SendFrame = Frame;
type RecvFrame = Frame;

#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let listener = TcpListener::bind("127.0.0.1:17653").await.unwrap();

    println!("listening on {:?}", listener.local_addr());

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        // Delimit frames using a length header
        let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());

        // Deserialize frames
        let mut deserialized = tokio_serde::SymmetricallyFramed::new(
            length_delimited,
            Bincode::<Frame, RecvFrame>::default(),
        );

        // Spawn a task that prints all received messages to STDOUT
        tokio::spawn(async move {
            while let Some(msg) = deserialized.try_next().await.unwrap() {
                println!("GOT: {}", msg.content);
            }
        });
    }
}
