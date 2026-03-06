pub mod tls;
pub mod scopes;
pub mod outbound;
pub mod middleware;
pub mod cryptography;

async fn hello_world() -> actix_web::HttpResponse {
    actix_web::HttpResponseBuilder::new(actix_web::http::StatusCode::OK)
    .body("hello world!")
}

#[bon::builder(finish_fn = serve)]
pub async fn server(
    port: u16,
    host: &str,
    certificate_file: Option<&str>,
    key_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use actix_web::*;
    log::info!("booting server");
    log::debug!("configuring outbound driver from environment variables");
    let driver = outbound::from_env().await?;
    log::debug!("recieved driver<{}>", driver.name());
    let tls: bool = certificate_file.is_some() && key_file.is_some();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(driver.clone()))
            .wrap(actix_cors::Cors::default())
            .wrap(crate::middleware::https::RedirectHTTPS::enabled(tls))
            .route("/hello", web::get().to(hello_world))
            .service(actix_files::Files::new("/", "./static")
                .index_file("index.html")
                .prefer_utf8(true)
            )
    });
    if tls {
        log::debug!("binding with TLS enabled");
        let tls = tls::config(certificate_file.unwrap(), key_file.unwrap())?;
        server.bind_rustls_0_23((host, port), tls)?.run().await?;
    } else {
        log::debug!("binding with TLS disabled");
        server.bind((host, port))?.run().await?;
    }
    Ok(())
}
