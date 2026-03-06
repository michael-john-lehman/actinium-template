pub mod db;
pub mod tls;
pub mod scopes;
pub mod middleware;
pub mod cryptography;

#[bon::builder(finish_fn = serve)]
pub async fn server(
    port: u16,
    host: &str,
    certificate_file: Option<&str>,
    key_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use actix_web::*;
    let repository = db::from_env().await?;
    let tls: bool = certificate_file.is_some() && key_file.is_some();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(repository.clone()))
            .wrap(actix_cors::Cors::default())
            .wrap(crate::middleware::https::RedirectHTTPS::enabled(tls))
            .service(actix_files::Files::new("/", "./static")
                .index_file("index.html")
                .prefer_utf8(true)
            )
    });
    if tls {
        let tls = tls::config(certificate_file.unwrap(), key_file.unwrap())?;
        server.bind_rustls_0_23((host, port), tls)?.run().await?;
    } else {
        server.bind((host, port))?.run().await?;
    }
    Ok(())
}
