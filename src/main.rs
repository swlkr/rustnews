mod db;

use axum::{
    http::{header::CONTENT_TYPE, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use db::db;
use enum_router::Routes;
use hyped::*;
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
    render(h1("self hosted hn clone").class("text-2xl text-center"))
}

fn render(element: Element) -> Html<String> {
    let static_files = StaticFile::once();
    Html(hyped::render((
        doctype(),
        html((
            head((
                title("social news"),
                script(()).src(&static_files.htmx),
                link(()).href(&static_files.tailwind).rel("stylesheet"),
            )),
            body(element).class(""),
        )),
    )))
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
