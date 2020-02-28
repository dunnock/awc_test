use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use awc::Client;
use bytes::BytesMut;
use futures::stream::StreamExt;
use std::env;

async fn index(req: HttpRequest) -> HttpResponse {
    let client = Client::default();

    let mut resp =
        client
        .get("https://upload.wikimedia.org/wikipedia/commons/f/ff/Pizigani_1367_Chart_10MB.jpg")
        .send()
        .await
        .unwrap();

    let now = std::time::Instant::now();

    let mut payload = BytesMut::new();
    while let Some(item) = resp.next().await { 
       payload.extend_from_slice(&item.unwrap());
    }

    println!("time elapsed while reading bytes into memory: {} secs", now.elapsed().as_secs());


    HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(payload)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let port = 3000;

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/").to(index))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
