// Benchmarking of benefit of readonly pool

use criterion::{Criterion, criterion_main};
use awc::Client;
use bytes::Bytes;
use actix_rt::SystemRunner;
use reqwest;
use std::future::Future;

const URL: &'static str = "https://upload.wikimedia.org/wikipedia/commons/f/ff/Pizigani_1367_Chart_10MB.jpg";

fn bench_fn<R>(criterion: &mut Criterion, rt: &mut SystemRunner, f: fn()->R, name: &str)
where
	R: Future<Output=Bytes> + 'static,
{
	// start benchmark loops
	criterion.bench_function(name, move |b| {
		b.iter_custom(|iters| {
			let elapsed = rt.block_on(async move {
				let mut total = 0;

				let start = std::time::Instant::now();

				for _ in 0..iters {
					total += (f)().await.len();
				}
		
				let elapsed = start.elapsed();
				println!("Throughput {}B/s", total * 1_000_000 / elapsed.as_micros() as usize);

				elapsed
			});
			// check that at least first request succeeded
			elapsed
		})
	});
}

async fn awc_test() -> Bytes {
	Client::default()
		.get(URL)
		.send()
		.await
		.expect("awc get send")
		.body()
		.limit(1024*1024*1024)
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
		.sample_size(10).nresamples(2).confidence_level(0.5).significance_level(0.5)
		.measurement_time(std::time::Duration::from_secs(10)).warm_up_time(std::time::Duration::from_secs(10));

	let mut rt = actix_rt::System::new("test");

	bench_fn(&mut criterion, &mut rt, awc_test, "awc");
	bench_fn(&mut criterion, &mut rt, reqwest_test, "reqwest");
	bench_fn(&mut criterion, &mut rt, awc_test, "awc");
	bench_fn(&mut criterion, &mut rt, reqwest_test, "reqwest");
}

criterion_main!(web_clients_benches);

