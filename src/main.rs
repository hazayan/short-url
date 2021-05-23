use crate::routes::Globals;
use dotenv::dotenv;
use tera::Tera;
use tide::utils::After;

mod db;
mod routes;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    tide::log::start();
    dotenv().ok();

    let pool = db::create_pool().await?;
    let mut tera = Tera::new("views/**/*").unwrap();
    tera.autoescape_on(vec!["html"]);

    let mut app = tide::with_state(Globals { tera, pool });
    app.with(After(routes::handle_error));

    app.at("/static").serve_dir("static/")?;
    app.at("/").get(routes::render_home);
    app.at("/").post(routes::render_hashed_url);
    app.at("/r/:hash").get(routes::redirect);

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
