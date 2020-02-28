use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use awc::Client;
use bytes::BytesMut;
use futures::stream::StreamExt;
use reqwest;
use std::env;

async fn index(req: HttpRequest) -> HttpResponse {
    // ----------------------------------------------------------------------
    // V1:  Run using awc
    // ----------------------------------------------------------------------
    let client = Client::default();

	let now = std::time::Instant::now();
    let mut res =
        client
        .get("https://upload.wikimedia.org/wikipedia/commons/f/ff/Pizigani_1367_Chart_10MB.jpg")
        .send()
        .await
        .unwrap();

    let mut payload = BytesMut::new();
    while let Some(item) = res.next().await {
        payload.extend_from_slice(&item.unwrap());
    }
    println!("awc time elapsed while reading bytes into memory: {} secs", now.elapsed().as_secs());

    // ----------------------------------------------------------------------
    // V2:  Run using reqwest
    // ----------------------------------------------------------------------
	let now = std::time::Instant::now();
	let payload = reqwest::get("https://upload.wikimedia.org/wikipedia/commons/f/ff/Pizigani_1367_Chart_10MB.jpg")
		.await.unwrap()
		.bytes()
		.await.unwrap();
    println!("reqwest time elapsed while reading bytes into memory: {} secs", now.elapsed().as_secs());
    
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
