use std::convert::Infallible;
use std::future::Future;
use std::net::TcpListener;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures_util::stream::Stream;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Bytes, Frame, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as ConnBuilder;

use crate::app::application::{AppError, Application};
use crate::http::request::HttpRequest;
use crate::http::request_body::RequestBody;
use crate::http::response_body::{ResponseBody, ResponseBodyStream};
use crate::runtime::runtime::{Runtime, RuntimeFuture};

pub(crate) struct HyperRuntime;

impl HyperRuntime {
    pub fn new() -> Self {
        Self
    }

    async fn handle_request(app: Arc<Application>, req: Request<Incoming>) -> Response<BoxedBody> {
        let method = req.method().as_str().to_string();
        let path = req.uri().path().to_string();
        log::debug!("request {} {}", method, path);

        let mut request = HttpRequest::new(&method, &path);
        for (name, value) in req.headers().iter() {
            if let Ok(value) = value.to_str() {
                request.headers_mut().insert(name.as_str(), value);
            }
        }

        let bytes = match req.into_body().collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => Bytes::new(),
        };

        if !bytes.is_empty() {
            request.set_body(RequestBody::Bytes(bytes.to_vec()));
        }

        let response = app.handle_http(request);
        log::debug!("response {} {} -> {}", method, path, response.status());
        Self::to_hyper_response(response)
    }

    fn to_hyper_response(response: crate::http::response::HttpResponse) -> Response<BoxedBody> {
        let mut builder = Response::builder().status(
            StatusCode::from_u16(response.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );

        if let Some(headers) = builder.headers_mut() {
            for (key, value) in response.headers().iter() {
                if let (Ok(name), Ok(value)) = (
                    hyper::header::HeaderName::from_bytes(key.as_bytes()),
                    hyper::header::HeaderValue::from_str(value),
                ) {
                    headers.insert(name, value);
                }
            }
        }

        let body = match response.into_body() {
            ResponseBody::Empty => empty_body(),
            ResponseBody::Bytes(bytes) => full_body(bytes.into()),
            ResponseBody::Text(text) => full_body(Bytes::from(text)),
            ResponseBody::Stream(stream) => {
                let stream = ResponseStream::new(stream);
                BoxBody::new(StreamBody::new(stream))
            }
        };

        builder
            .body(body)
            .unwrap_or_else(|_| Response::new(empty_body()))
    }
}

impl Runtime for HyperRuntime {
    fn run<'a>(
        &'a self,
        listener: TcpListener,
        app: Arc<Application>,
        shutdown: Pin<Box<dyn Future<Output = ()> + Send + 'a>>,
    ) -> RuntimeFuture<'a> {
        Box::pin(async move {
            log::info!("runtime accept loop started");
            let listener = tokio::net::TcpListener::from_std(listener)
                .map_err(|err| AppError::runtime("bind", err))?;

            tokio::pin!(shutdown);
            loop {
                tokio::select! {
                    _ = &mut shutdown => {
                        log::info!("runtime shutdown signal received");
                        break;
                    }
                    accept = listener.accept() => {
                        let (stream, _) = accept.map_err(|err| AppError::runtime("accept", err))?;
                        log::debug!("accepted connection");
                        let io = TokioIo::new(stream);
                        let app = Arc::clone(&app);
                        let service = service_fn(move |req| {
                            let app = Arc::clone(&app);
                            async move {
                                Ok::<_, Infallible>(HyperRuntime::handle_request(app, req).await)
                            }
                        });

                        tokio::spawn(async move {
                            let _ = ConnBuilder::new(TokioExecutor::new())
                                .serve_connection_with_upgrades(io, service)
                                .await;
                        });
                    }
                }
            }
            log::info!("runtime accept loop stopped");
            Ok(())
        })
    }
}

struct ResponseStream {
    inner: Mutex<Box<dyn ResponseBodyStream>>,
}

impl ResponseStream {
    fn new(inner: Box<dyn ResponseBodyStream>) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }
}

impl Stream for ResponseStream {
    type Item = Result<Frame<Bytes>, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut guard = self.inner.lock().expect("response stream lock");
        match guard.read_chunk() {
            Ok(Some(chunk)) => Poll::Ready(Some(Ok(Frame::data(Bytes::from(chunk))))),
            Ok(None) => Poll::Ready(None),
            Err(err) => Poll::Ready(Some(Err(err))),
        }
    }
}

type BoxedBody = BoxBody<Bytes, std::io::Error>;

fn full_body(bytes: Bytes) -> BoxedBody {
    Full::new(bytes)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "body error"))
        .boxed()
}

fn empty_body() -> BoxedBody {
    full_body(Bytes::new())
}
