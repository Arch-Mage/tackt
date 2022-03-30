use tackt::route;
use tackt::Method;
use tackt::MethodReq;
use tackt::Param;
use tackt::PathReq;

#[test]
fn test() {
    let router = tackt::routes![home, login, user, content];

    assert_eq!(exec(router, Method::GET, "/").unwrap(), "home");
    assert_eq!(exec(router, Method::GET, "/login").unwrap(), "login");
    assert_eq!(exec(router, Method::POST, "/login").unwrap(), "login");
    assert_eq!(exec(router, Method::GET, "/user/1").unwrap(), "user 1");
    assert_eq!(
        exec(router, Method::GET, "/content/1/name/path/to/file").unwrap(),
        "content 1 name path/to/file"
    );
}

impl Response {
    fn new<S: Into<String>>(body: S) -> Response {
        Response { body: body.into() }
    }
}

#[route]
async fn home(_: Request) -> Result<Response, Error> {
    Ok(Response::new("home"))
}

#[route(GET, POST: "login")]
async fn login(_: Request) -> Result<Response, Error> {
    Ok(Response::new("login"))
}

#[route(GET: "user" / id)]
async fn user(_: Request, id: i32) -> Result<Response, Error> {
    Ok(Response::new(format!("user {}", id)))
}

#[derive(Param)]
#[route(GET: "content" / id / name / path*)]
struct Content {
    id: i32,
    name: String,
    path: String,
}

async fn content(_: Request, param: Content) -> Result<Response, Error> {
    Ok(Response::new(format!(
        "content {} {} {}",
        param.id, param.name, param.path
    )))
}

#[derive(PartialEq, Eq)]
struct Request {
    method: Method,
    path: &'static str,
}

impl Request {
    fn new(method: Method, path: &'static str) -> Request {
        Request { method, path }
    }
}

impl PathReq for Request {
    fn path(&self) -> &str {
        self.path
    }
}

impl MethodReq for Request {
    fn method(&self) -> &http::Method {
        &self.method
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Response {
    body: String,
}

impl PartialEq<String> for Response {
    fn eq(&self, other: &String) -> bool {
        self.body.as_str() == other.as_str()
    }
}

impl<'a> PartialEq<&'a str> for Response {
    fn eq(&self, other: &&'a str) -> bool {
        self.body.as_str() == *other
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Error(tackt::Error);

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<tackt::Error> for Error {
    fn from(err: tackt::Error) -> Self {
        Error(err)
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

fn exec(
    mut svc: impl tackt::Service<Request, Response = Response, Error = Error>,
    method: http::Method,
    path: &'static str,
) -> Result<Response, Error> {
    let req = Request::new(method, path);
    let mut fut = Box::pin(svc.call(req));

    let waker = Waker::new().into();
    let mut ctx = std::task::Context::from_waker(&waker);

    match std::future::Future::poll(fut.as_mut(), &mut ctx) {
        std::task::Poll::Ready(out) => out,
        std::task::Poll::Pending => panic!("still pending!!!"),
    }
}
