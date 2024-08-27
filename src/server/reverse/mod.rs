// use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer, Responder};
// use reqwest::Client;

// async fn handle_request(req: HttpRequest) -> impl Responder {
//     let url = req.url().clone();
//     let client = Client::new();
//     let res = client.get("http://example.com") // replace with your target server
//         .header("Host", url.host().unwrap())
//         .header("X-Real-IP", req.peer_addr().unwrap().ip().to_string())
//         .send()
//         .await
//         .unwrap();
//     HttpResponse::build(res.status())
//         .headers(res.headers().clone())
//         .body(res.text().await.unwrap())
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new().route("/", web::get().to(handle_request))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }
