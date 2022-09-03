use tokio_stream::wrappers::UnixListenerStream;
use warp::Filter;

#[derive(serde::Deserialize)]
struct AccountApiResponse {
	//id: u16,
	name: String,
	//alias: String,
	//email: String,
	//rank: String,
}


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
		.map(|| "for documentation, visit: https://github.com/L3P3/minicraft-server");

	let route_greet = warp::path("greet").and(
		warp::path::end()
			.map(|| "greet who? => /name")
			.or(warp::path::param().map(|name: String| format!("hello, {}!", name))),
	);

	let route_account = warp::path("account")
		.and(warp::path::end())
		.and(warp::filters::cookie::optional("token"))
		.and_then(|token_opt: Option<String>| async move {
			Ok::<String, warp::Rejection>(match token_opt {
				Some(token) => {
					// TODO move this out
					let request_template = reqwest::Client::new()
						.get("http://127.0.0.1:8000/svr/account/account.json")
						.header(reqwest::header::USER_AGENT, "minicraft-server");
					let resp = request_template.try_clone().unwrap()
						.query(&[("token", token)])
						.send().await.expect("cannot reach account api");
					let json: AccountApiResponse = resp.json().await.expect("cannot parse api response");
					format!("current account: {}", json.name)
				},
				None => "token cookie missing!".to_string(),
			})
		});

	let route_main = warp::get()
		.and(
			route_index
				.or(route_greet)
				.or(route_account)
		)
		.with(warp::log::custom(|info| {
			println!("{} {} {}", info.method(), info.path(), info.status());
		}));

	warp::serve(route_main)
		.run_incoming(stream_incoming_get())
		.await;
}
