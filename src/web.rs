use crate::Source;
use actix_web::{App, HttpServer};
use routes::health_check;

fn run() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        let app = App::new().service(health_check);
        app
    });
    Ok(())
}

pub mod routes {
    use crate::{Article, Source};
    use actix_web::{get, web::Query, HttpResponse};

	

    #[get("/health.check")]
    async fn health_check() -> std::io::Result<HttpResponse> {
        Ok(HttpResponse::Ok().finish())
    }
}
