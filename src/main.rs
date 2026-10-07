use std::{fmt, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};

use hyper::{Request, Response, body::Incoming, server::conn::http1, service::service_fn};

use hyper_util::rt::TokioIo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(proxy))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[derive(Debug)]
enum ProxyError {
    Io(std::io::Error),
    Hyper(hyper::Error),
    // Http(hyper::http::Error),
    InvalidUri(hyper::http::uri::InvalidUri),
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyError::Io(err) => write!(f, "IO error: {}", err),
            ProxyError::Hyper(err) => write!(f, "Hyper error: {}", err),
            // ProxyError::Http(err) => write!(f, "HTTP error: {}", err),
            ProxyError::InvalidUri(err) => write!(f, "Invalid URI: {}", err),
        }
    }
}

impl std::error::Error for ProxyError {}

impl From<std::io::Error> for ProxyError {
    fn from(err: std::io::Error) -> Self {
        ProxyError::Io(err)
    }
}

impl From<hyper::Error> for ProxyError {
    fn from(err: hyper::Error) -> Self {
        ProxyError::Hyper(err)
    }
}

impl From<hyper::http::uri::InvalidUri> for ProxyError {
    fn from(err: hyper::http::uri::InvalidUri) -> Self {
        ProxyError::InvalidUri(err)
    }
}

async fn proxy(mut request: Request<Incoming>) -> Result<Response<Incoming>, ProxyError> {
    let url = "http://127.0.0.1:3000".parse::<hyper::Uri>()?;

    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap_or(80);

    let address = format!("{}:{}", host, port);

    let stream = TcpStream::connect(address).await?;

    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    request.headers_mut().insert(
        hyper::http::header::HOST,
        hyper::http::HeaderValue::from_static("http://127.0.0.1:3000"),
    );

    let response = sender.send_request(request).await?;
    Ok(response)
}
