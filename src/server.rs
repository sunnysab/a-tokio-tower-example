use std::pin::Pin;
use std::time::Duration;

use async_bincode::AsyncBincodeStream;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_tower::multiplex;
use tokio_tower::multiplex::Server;

#[derive(Debug, Serialize, Deserialize)]
pub struct Frame {
    delay: i32,
    content: String,
}

type Request = Frame;
type Response = Frame;

#[derive(Debug, Default)]
// only pub because we use it to figure out the error type for ViewError
pub struct Tagger(slab::Slab<()>);

impl<Request: core::fmt::Debug, Response: core::fmt::Debug>
multiplex::TagStore<Tagged<Request>, Tagged<Response>> for Tagger
{
    type Tag = u32;

    fn assign_tag(mut self: Pin<&mut Self>, r: &mut Tagged<Request>) -> Self::Tag {
        r.tag = self.0.insert(()) as u32;
        r.tag
    }
    fn finish_tag(mut self: Pin<&mut Self>, r: &Tagged<Response>) -> Self::Tag {
        self.0.remove(r.tag as usize);
        r.tag
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Tagged<T>
    where
        T: core::fmt::Debug,
{
    pub v: T,
    pub tag: u32,
}

impl<T: core::fmt::Debug> From<T> for Tagged<T> {
    fn from(t: T) -> Self {
        Tagged { tag: 0, v: t }
    }
}

async fn handler(req: Tagged<Frame>) -> Result<Tagged<Frame>, anyhow::Error> {
    let tag = req.tag;
    println!("Received frame: {:?}, tag = {}", &req, tag);

    let mut response = Tagged::<Frame>::from(Frame {
        delay: req.v.delay,
        content: format!("Hello, {}", req.v.content),
    });

    tokio::time::sleep(Duration::from_secs(req.v.delay as u64)).await;
    response.tag = tag;
    Ok(response)
}

#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let listener = TcpListener::bind("127.0.0.1:17653").await.unwrap();

    println!("listening on {:?}", listener.local_addr());

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let server = Server::new(
            AsyncBincodeStream::from(socket).for_async(),
            tower::service_fn(handler),
        )
            .await;

        if let Err(e) = server {
            eprintln!("Server error: {:?}", e);
        }
    }
}
