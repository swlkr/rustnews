use axum::{
    http::{header::CONTENT_TYPE, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use enum_router::Routes;
use hyped::*;
use rustnews::*;
use static_stash::{Css, Js, StaticFiles};

#[tokio::main]
async fn main() {
    let _x = import().await;
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

async fn index() -> Result<impl IntoResponse> {
    let Database { db, posts } = db().await;
    let posts: Vec<Post> = db
        .select()
        .from(posts)
        .order(vec![desc(posts.created_at)])
        .all()
        .await?;

    render((
        h1("rust news").class("text-2xl text-center"),
        posts
            .into_iter()
            .map(|post| div(post.title))
            .collect::<Vec<_>>(),
    ))
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

pub fn render(element: impl Render + 'static) -> Result<Html<String>> {
    let files = StaticFile::once();
    Ok(Html(hyped::render((
        doctype(),
        html((
            head((
                title(""),
                script(()).src(&files.htmx),
                link(()).href(&files.tailwind).rel("stylesheet"),
            )),
            body(element).class("dark:bg-slate-950 dark:text-white bg-gray-50 text-slate-950"),
        )),
    ))))
}
