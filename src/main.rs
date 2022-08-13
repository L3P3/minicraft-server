use tokio_stream::wrappers::UnixListenerStream;
use warp::Filter;

fn stream_incoming_get() -> UnixListenerStream {
	const PATH_SOCKET: &str = "/tmp/minicraft.socket";

	match std::fs::remove_file(PATH_SOCKET) {
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
		result => result.expect("failed to remove existing socket file"),
	}

	UnixListenerStream::new(
		tokio::net::UnixListener::bind(PATH_SOCKET).expect("failed to create socket file"),
	)
}

#[tokio::main]
async fn main() {
	let route_index = warp::path::end()
		.map(|| "For documentation, visit: https://github.com/L3P3/minicraft-server");

	let route_greet = warp::path("greet").and(
		warp::path::end()
			.map(|| "Greet who? => /name")
			.or(warp::path::param().map(|name: String| format!("Hello, {}!", name))),
	);

	let route_main = warp::get()
		.and(route_index.or(route_greet))
		.with(warp::log::custom(|info| {
			println!("{} {} {}", info.method(), info.path(), info.status());
		}));

	warp::serve(route_main)
		.run_incoming(stream_incoming_get())
		.await;
}
