use std::future::Future;

use http::Method;
use http::Request;
use http::Response;
use http::Uri;
use tackt::route;
use tackt::Param;
use tower_service::Service;

fn create_router() -> impl Service<Request<()>, Error = Error, Response = Response<String>> {
    tackt::routes![home, login, user, content]
        .mount("/protected", tackt::routes![protected].with(protection))
}

#[test]
fn test() {
    let mut router = create_router();

    let res = oneshot(router.call(request(Method::GET, "/")));
    assert_eq!(res.map(Response::into_body), Ok("home".to_string()));

    let res = oneshot(router.call(request(Method::GET, "/login")));
    assert_eq!(res.map(Response::into_body), Ok("login".to_string()));

    let res = oneshot(router.call(request(Method::POST, "/login")));
    assert_eq!(res.map(Response::into_body), Ok("login".to_string()));

    let res = oneshot(router.call(request(Method::GET, "/user/1")));
    assert_eq!(res.map(Response::into_body), Ok("user 1".to_string()));

    let res = oneshot(router.call(request(Method::GET, "/content/1/name/path/to/file")));
    assert_eq!(
        res.map(Response::into_body),
        Ok("content 1 name path/to/file".to_string())
    );

    let res = oneshot(router.call(request(Method::GET, "/protected")));
    assert_eq!(res.map(Response::into_body), Err(Error::Unauthorized));

    let mut req = request(Method::GET, "/protected/");
    req.headers_mut()
        .insert("user", "someone".try_into().unwrap());
    let res = oneshot(router.call(req));
    assert_eq!(res.map(Response::into_body), Ok("someone".to_string()));
}

fn respond<S: Into<String>>(body: S) -> Response<String> {
    Response::new(body.into())
}

fn request(method: Method, path: &'static str) -> Request<()> {
    let mut req = Request::new(());
    *req.method_mut() = method;
    *req.uri_mut() = Uri::from_static(path);
    req
}

#[route]
async fn home(_: Request<()>) -> Result<Response<String>, Error> {
    Ok(respond("home"))
}

#[route(GET, POST: "login")]
async fn login(_: Request<()>) -> Result<Response<String>, Error> {
    Ok(respond("login"))
}

#[route(GET: "user" / id)]
async fn user(_: Request<()>, id: i32) -> Result<Response<String>, Error> {
    Ok(respond(format!("user {}", id)))
}

#[derive(Param)]
#[route(GET: "content" / id / name / path*)]
struct Content {
    id: i32,
    name: String,
    path: String,
}

async fn content(_: Request<()>, param: Content) -> Result<Response<String>, Error> {
    Ok(respond(format!(
        "content {} {} {}",
        param.id, param.name, param.path
    )))
}

#[route]
async fn protected(req: Request<()>) -> Result<Response<String>, Error> {
    let user = req.headers().get("user").unwrap().to_str().unwrap();
    Ok(respond(user))
}

async fn protection(req: Request<()>) -> Result<Request<()>, Error> {
    req.headers().get("user").ok_or(Error::Unauthorized)?;
    Ok(req)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Error {
    Routing(tackt::Error),
    Unauthorized,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Routing(err) => write!(f, "{err}"),
            Error::Unauthorized => write!(f, "unauthorized"),
        }
    }
}

impl From<tackt::Error> for Error {
    fn from(err: tackt::Error) -> Self {
        Error::Routing(err)
    }
}

struct Waker(std::thread::Thread);

impl Waker {
    fn new() -> std::sync::Arc<Waker> {
        std::sync::Arc::new(Waker(std::thread::current()))
    }
}

impl std::task::Wake for Waker {
    fn wake(self: std::sync::Arc<Self>) {
        self.0.unpark();
    }
}

fn oneshot<T>(fut: impl Future<Output = T>) -> T {
    use std::task::Context;
    use std::task::Poll;

    let mut fut = Box::pin(fut);
    let waker = Waker::new().into();
    let mut cx = Context::from_waker(&waker);

    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(out) => out,
        Poll::Pending => panic!("still pending!!!"),
    }
}
