use std::pin::Pin;
use std::time::Duration;

use async_bincode::AsyncBincodeStream;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_tower::multiplex;
use tokio_tower::multiplex::Client;
use tower::{Service, ServiceExt};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

fn on_service_error(e: anyhow::Error) {
    eprintln!("error handling: {:?}", e);
}

pub async fn ready<S: Service<RequestFrame>, RequestFrame>(svc: &mut S) -> Result<(), S::Error> {
    use futures_util::future::poll_fn;

    poll_fn(|cx| svc.poll_ready(cx)).await
}

#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let socket = TcpStream::connect("127.0.0.1:17653").await.unwrap();

    socket.set_nodelay(true).unwrap();

    let stream = AsyncBincodeStream::from(socket).for_async();
    let t = multiplex::MultiplexTransport::new(stream, Tagger::default());

    let mut client = Client::with_error_handler(t, on_service_error);

    let delay_array = vec![5, 4, 1, 2, 1];

    for i in 0..5 {
        let ready_client = client.ready().await.unwrap();

        let fut1 = ready_client.call(Tagged::<Frame>::from(Frame {
            content: "Bob !".to_string(),
            delay: delay_array[i],
        }));

        tokio::task::spawn(async {
            let response: Tagged<Frame> = fut1.await.unwrap();
            println!("response = {:?}", response);
        });
    }

    loop {
        tokio::time::sleep(Duration::from_secs(1000)).await;
    }
}
