use std::convert::Infallible;
use std::net::SocketAddr;

use http::Request;
use http::Response;
use hyper::server::conn::AddrIncoming;
use hyper::server::conn::AddrStream;
use hyper::service::make_service_fn;
use hyper::service::Service;
use hyper::Body;
use hyper::Server;

type Error = Box<dyn 'static + Send + Sync + std::error::Error>;

#[tackt::route]
async fn home(_: Request<Body>) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from("home")))
}

#[tackt::route(GET: "about")]
async fn about(_: Request<Body>) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from("about")))
}

#[tackt::route(GET, PUT: "entity" / id)]
async fn entity(_: Request<Body>, id: i32) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from(format!("entity: {id}"))))
}

#[tackt::route(GET, PUT: "entity" / id / "resource" / path*)]
async fn resource(_: Request<Body>, id: i32, path: String) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from(format!("resource: {id} {path}"))))
}

async fn run(addr: SocketAddr) -> Server<AddrIncoming, impl Service<&'static AddrStream>> {
    let router = tackt::routes![home, about, entity, resource,];

    let make_svc = make_service_fn(move |_| async move { Ok::<_, Infallible>(router) });

    Server::bind(&addr).serve(make_svc)
}

fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));

    // NOTE: this requires an async executor
    let _ = run(addr);
}
