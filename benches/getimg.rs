// Benchmarking of benefit of readonly pool

use criterion::{Criterion, criterion_main};
use bytes::Bytes;
use actix_rt::SystemRunner;
use reqwest;
use std::future::Future;
use actix_web::{client::{Client, Connector}};


const URL: &'static str = "https://upload.wikimedia.org/wikipedia/commons/f/ff/Pizigani_1367_Chart_10MB.jpg";

fn bench_fn<R>(criterion: &mut Criterion, rt: &mut SystemRunner, f: fn()->R, name: &str)
where
	R: Future<Output=Bytes> + 'static,
{
	// start benchmark loops
	criterion.bench_function(name, move |b| {
		b.iter_custom(|iters| {
			rt.block_on(async move {
				let mut total = 0;

				let start = std::time::Instant::now();

				for _ in 0..iters {
					total += (f)().await.len();
				}
		
				let elapsed = start.elapsed();
				println!("Throughput {}B/s", total * 1_000 / elapsed.as_millis() as usize);
				println!("Iters {} total {} elapsed {}ms", iters, total, elapsed.as_millis());

				elapsed
			})
		})
	});
}

use openssl::ssl::{SslConnector, SslMethod};

async fn awc_test_openssl() -> Bytes {
	let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
//	builder.set_alpn_protos(b"\x02h2\x08http/1.1").unwrap(); 

	let client = Client::build()
		.connector(Connector::new().ssl(builder.build()).finish())
		.finish();

	client
		.get(URL)
		.send()
		.await
		.expect("awc get send")
		.body()
		.limit(20_000_000)
		.await
		.expect("awc body")
}

use rustls::ClientConfig;
use std::sync::Arc;

async fn awc_test_rustls() -> Bytes {
	let protos = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
	let mut config = ClientConfig::new();
	//config.set_protocols(&protos);
	config
		.root_store
		.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

	let client = Client::build()
		.connector(Connector::new().rustls(Arc::new(config)).finish())
		.finish();

	client
		.get(URL)
		.send()
		.await
		.expect("awc get send")
		.body()
		.limit(20_000_000)
		.await
		.expect("awc body")
}

async fn awc_test_rustls_protocols() -> Bytes {
	let protos = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
	let mut config = ClientConfig::new();
	config.set_protocols(&protos);
	config
		.root_store
		.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

	let client = Client::build()
		.connector(Connector::new().rustls(Arc::new(config)).finish())
		.finish();

	client
		.get(URL)
		.send()
		.await
		.expect("awc get send")
		.body()
		.limit(20_000_000)
		.await
		.expect("awc body")
}

async fn awc_test_default() -> Bytes {
	Client::default()
		.get(URL)
		.send()
		.await
		.expect("awc get send")
		.body()
		.limit(20_000_000)
		.await
		.expect("awc body")
}

async fn reqwest_test() -> Bytes {
	reqwest::get(URL)
		.await
		.expect("reqwest get")
		.bytes()
		.await
		.expect("reqwest bytes")
}

pub fn web_clients_benches() {
    let mut criterion: ::criterion::Criterion<_> =
		::criterion::Criterion::default().configure_from_args()
		.sample_size(10);

	let mut rt = actix_rt::System::new("test");

	bench_fn(&mut criterion, &mut rt, awc_test_rustls, "awc_test_rustls");
	bench_fn(&mut criterion, &mut rt, awc_test_rustls, "awc_test_rustls_protocols");
	bench_fn(&mut criterion, &mut rt, awc_test_openssl, "awc_test_openssl");
	bench_fn(&mut criterion, &mut rt, awc_test_default, "awc_test_default");
	bench_fn(&mut criterion, &mut rt, reqwest_test, "reqwest");
}

criterion_main!(web_clients_benches);

