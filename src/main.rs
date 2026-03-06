use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Host address to bind the server to
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,
    
    /// Port number to listen on
    #[arg(long, default_value_t = 8000)]
    port: u16,

    /// Path to TLS certificate file (enables HTTPS if provided with key-file)
    #[arg(long)]
    certificate_file: Option<String>,

    /// Path to TLS private key file (enables HTTPS if provided with certificate-file)
    #[arg(long)]
    key_file: Option<String>
}

#[inline]
fn pre() {
    dotenv::dotenv().ok();
    env_logger::init();
}

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    backend::server()
        .host(&args.host)
        .port(args.port)
        .maybe_certificate_file(args.certificate_file.as_deref())
        .maybe_key_file(args.key_file.as_deref())
        .serve()
        .await?;
    Ok(())
}

#[actix_web::main]
async fn main() {
    pre();
    if let Err(error) = run(Args::parse()).await {
        log::error!("{error}");
    }
}
