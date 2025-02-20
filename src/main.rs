use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Item {
    id: u32,
    name: String,
}

type Items = Arc<Mutex<Vec<Item>>>;

async fn handle_request(req: Request<Body>, items: Items) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/items") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let new_item: Item = serde_json::from_slice(&whole_body).unwrap();
            let mut items = items.lock().unwrap();
            items.push(new_item);
            Ok(Response::new(Body::from("Item added")))
        }
        (&Method::GET, "/items") => {
            let items = items.lock().unwrap();
            Ok(Response::new(Body::from(serde_json::to_string(&*items).unwrap())))
        }
        (&Method::PUT, path) if path.starts_with("/items/") => {
            let id: u32 = path.trim_start_matches("/items/").parse().unwrap();
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let updated_item: Item = serde_json::from_slice(&whole_body).unwrap();
            let mut items = items.lock().unwrap();
            if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                *item = updated_item;
                Ok(Response::new(Body::from("Item updated")))
            } else {
                Ok(Response::new(Body::from("Item not found")))
            }
        }
        (&Method::DELETE, path) if path.starts_with("/items/") => {
            let id: u32 = path.trim_start_matches("/items/").parse().unwrap();
            let mut items = items.lock().unwrap();
            items.retain(|i| i.id != id);
            Ok(Response::new(Body::from("Item deleted")))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let items = Arc::new(Mutex::new(Vec::<Item>::new()));

    let make_svc = make_service_fn(move |_conn| {
        let items = items.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, items.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}