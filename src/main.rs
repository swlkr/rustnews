mod components;
mod db;

use axum::{
    http::{header::CONTENT_TYPE, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use components::*;
use db::*;
use enum_router::Routes;
use static_stash::{Css, Js, StaticFiles};

#[tokio::main]
async fn main() {
    let _ = StaticFile::once();
    let _ = db().await;

    let ip = "127.0.0.1:9006";
    let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
    println!("Listening on {}", ip);
    axum::serve(listener, routes()).await.unwrap();
}

fn routes() -> Router {
    Route::router().route("/*file", get(serve_file))
}

async fn index() -> impl IntoResponse {
    let divs = (0..1000).map(|i| div(i).class("py-96")).collect::<Vec<_>>();
    render((h1("rust news").class("text-2xl text-center"), divs))
}

async fn serve_file(uri: Uri) -> impl IntoResponse {
    let static_files = StaticFile::once();
    match static_files.get(&uri.path()) {
        Some(file) => (
            StatusCode::OK,
            [(CONTENT_TYPE, file.content_type)],
            file.content,
        ),
        None => (
            StatusCode::NOT_FOUND,
            [(CONTENT_TYPE, "text/html; charset=utf-8")],
            "not found".as_bytes().to_vec(),
        ),
    }
}

#[allow(dead_code)]
#[derive(Routes)]
enum Route {
    #[get("/")]
    Index,
}

#[derive(StaticFiles)]
struct StaticFile {
    #[file("/static/htmx.min.js")]
    htmx: Js,
    #[file("/static/tailwind.css")]
    tailwind: Css,
}
