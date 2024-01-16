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
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = StaticFile::once();
    let _ = db().await;
    let importer = tokio::task::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));

        loop {
            interval.tick().await;
            match import().await {
                Ok(_) => {}
                Err(err) => return err,
            };
        }
    });

    let server = tokio::task::spawn(async {
        let ip = "127.0.0.1:9006";
        let listener = tokio::net::TcpListener::bind(ip).await.unwrap();
        println!(
            "Listening on http://localhost:{}",
            listener.local_addr().unwrap().port()
        );
        axum::serve(listener, routes()).await.unwrap();
    });

    tokio::try_join!(importer, server)?;

    Ok(())
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
        h1("rust news").class("text-2xl text-center pt-8"),
        div((posts
            .into_iter()
            .map(|post| render_post(post))
            .collect::<Vec<_>>(),))
        .class("flex flex-col gap-8 pt-8"),
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
                title("rust news"),
                script(()).src(&files.htmx),
                link(()).href(&files.tailwind).rel("stylesheet"),
            )),
            body(hyped::main(element).class("max-w-2xl mx-auto"))
                .class("dark:bg-gray-950 dark:text-white bg-gray-50 text-slate-950 pb-8"),
        )),
    ))))
}

fn render_post(post: Post) -> Element {
    div((
        a(post.title.clone()).href(post.link.clone()),
        div((
            div(time_ago(post.created_at)),
            div("·"),
            link(post.source_display()).href(post.source_link),
        ))
        .class("flex gap-2"),
    ))
    .class("flex flex-col gap-1")
}

fn a(s: impl Render + 'static) -> Element {
    hyped::a(s)
        .rel("noreferrer noopener")
        .target("_blank")
        .class("underline dark:text-orange-400 text-orange-500")
}

fn now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn time_ago(seconds: u64) -> impl Render {
    let now = now();
    let seconds = now - seconds;

    const YEAR: u64 = 31_536_000;
    const MONTH: u64 = 2_592_000;
    const DAY: u64 = 86_400;
    const HOUR: u64 = 3600;
    const MINUTE: u64 = 60;
    let diff = seconds / YEAR;
    if diff > 0 {
        return format!("{}y ago", diff);
    }

    let diff = seconds / MONTH;
    if diff > 0 {
        return format!("{}m ago", diff);
    }

    let diff = seconds / DAY;
    if diff > 0 {
        return format!("{}d ago", diff);
    }

    let diff = seconds / HOUR;
    if diff > 0 {
        return format!("{}h ago", diff);
    }

    let diff = seconds / MINUTE;
    if diff > 0 {
        return format!("{}m ago", diff);
    }

    return format!("{}s ago", seconds);
}
