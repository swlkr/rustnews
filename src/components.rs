use std::io::Write;

use crate::{Route, StaticFile};
use axum::response::Html;
pub use hyped::*;

pub fn render(element: impl Render + 'static) -> Html<String> {
    let files = StaticFile::once();
    Html(hyped::render((
        doctype(),
        html((
            head((
                title(""),
                script(()).src(&files.htmx),
                link(()).href(&files.tailwind).rel("stylesheet"),
            )),
            body((nav(), element))
                .class("dark:bg-slate-950 dark:text-white bg-gray-50 text-slate-950"),
        )),
    )))
}

fn nav() -> Element {
    const NAV_CLASS: &'static str = r#"
        lg:px-0 lg:justify-center lg:relative lg:bottom-auto
        flex gap-4 py-5 items-center justify-between px-5
        fixed bottom-0 text-center w-full dark:bg-slate-800
    "#;

    hyped::nav((
        nav_link(Icon::Home).href(Route::Index),
        nav_link(Icon::Search).href(Route::Index),
        nav_link(Icon::Add).href(Route::Index),
        nav_link(Icon::Past).href(Route::Index),
        nav_link(Icon::Profile).href(Route::Index),
    ))
    .class(NAV_CLASS)
}

enum Icon {
    Home,
    Search,
    Add,
    Past,
    Profile,
}

impl Render for Icon {
    fn render(&self, buffer: &mut Vec<u8>) -> std::io::Result<()> {
        match self {
            Icon::Home => {
                buffer.write(br##"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 12 8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
</svg>
"##)?;
            }
            Icon::Search => {
                buffer.write(br##"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" />
</svg>
"##)?;
            }
            Icon::Add => {
                buffer.write(br##"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v6m3-3H9m12 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
</svg>
"##)?;
            }
            Icon::Past => {
                buffer.write(br##"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="M21 16.811c0 .864-.933 1.406-1.683.977l-7.108-4.061a1.125 1.125 0 0 1 0-1.954l7.108-4.061A1.125 1.125 0 0 1 21 8.689v8.122ZM11.25 16.811c0 .864-.933 1.406-1.683.977l-7.108-4.061a1.125 1.125 0 0 1 0-1.954l7.108-4.061a1.125 1.125 0 0 1 1.683.977v8.122Z" />
</svg>
"##)?;
            }
            Icon::Profile => {
                buffer.write(br##"<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="M17.982 18.725A7.488 7.488 0 0 0 12 15.75a7.488 7.488 0 0 0-5.982 2.975m11.963 0a9 9 0 1 0-11.963 0m11.963 0A8.966 8.966 0 0 1 12 21a8.966 8.966 0 0 1-5.982-2.275M15 9.75a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
</svg>
"##)?;
            }
        }

        Ok(())
    }
}

fn nav_link(r: impl Render + 'static) -> Element {
    a(r)
}
