use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use futures::SinkExt;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Frame {
    content: String,
}

type SendFrame = Frame;
type RecvFrame = Frame;


#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let socket = TcpStream::connect("127.0.0.1:17653").await.unwrap();

    // Delimit frames using a length header
    let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());

    // Serialize frames with JSON
    let mut serialized =
        tokio_serde::SymmetricallyFramed::new(length_delimited, Bincode::<RecvFrame, SendFrame>::default());

    // Send the value
    serialized
        .send(Frame {
            content: String::from("Hello world")
        })
        .await
        .unwrap()
}
