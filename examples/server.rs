use std::convert::Infallible;
use std::net::SocketAddr;

use http::Request;
use http::Response;
use hyper::service::make_service_fn;
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

async fn run(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let router = tackt::routes![home, about, entity, resource,];

    Server::bind(&addr)
        .serve(make_service_fn(move |_| async move {
            Ok::<_, Infallible>(router)
        }))
        .await?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run(addr))
}
