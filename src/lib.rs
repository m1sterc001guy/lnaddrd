use anyhow::Result;
use api::{
    get_lnaddr_handler, get_lnaddr_manifest_handler, list_domains_handler, register_lnaddr_handler,
    remove_lnaddr_handler,
};
use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
};
use config::Config;
use repository::pg::PgPaymentAddressRepository;
use service::LnaddrService;
use service::direct::DirectLnaddrService;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{debug, info};

pub mod api;
pub mod config;
pub mod repository;
pub mod service;

#[derive(Clone)]
pub struct AppState {
    pub service: LnaddrService,
    pub config: Arc<Config>,
}

pub async fn serve(config: &Config) -> Result<()> {
    debug!(db=%config.database, "Opening database connection");
    let lnaddr_repo = PgPaymentAddressRepository::new(&config.database)?.into_dyn();

    debug!(domains=?config.domains, "Starting LN address service");
    let lnaddr_service = DirectLnaddrService::new(lnaddr_repo, config.domains.clone()).into_dyn();

    let app_state = AppState {
        service: lnaddr_service.clone(),
        config: Arc::new(config.clone()),
    };

    let app = Router::new()
        .route("/domains", get(list_domains_handler))
        .route("/lnaddress/:domain/:username", get(get_lnaddr_handler))
        .route("/lnaddress/register", post(register_lnaddr_handler))
        .route("/lnaddress/remove", delete(remove_lnaddr_handler))
        .route(
            "/.well-known/lnurlp/:username",
            get(get_lnaddr_manifest_handler),
        )
        .route("/", get(landing_page))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(app_state)
        .fallback(|_req: axum::http::Request<axum::body::Body>| async move {
            axum::http::StatusCode::NOT_FOUND
        });

    info!(bind=%config.bind, "Starting HTTP server");
    let listener = TcpListener::bind(&config.bind).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn landing_page() -> impl IntoResponse {
    Html(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>The E-Cash App</title>
        <style>
            body {
                margin: 0;
                font-family: 'Segoe UI', Roboto, sans-serif;
                background: linear-gradient(135deg, #4dd0ff, #2196f3);
                color: white;
                text-align: center;
            }
            header {
                padding: 60px 20px 40px;
            }
            header img {
                width: 120px;
                height: 120px;
                border-radius: 24px;
                box-shadow: 0 8px 20px rgba(0,0,0,0.3);
            }
            header h1 {
                margin-top: 20px;
                font-size: 2.5rem;
                font-weight: bold;
            }
            .download-btn {
                display: inline-block;
                margin-top: 20px;
                padding: 14px 28px;
                background: white;
                color: #2196f3;
                font-size: 1.2rem;
                border-radius: 30px;
                text-decoration: none;
                font-weight: bold;
                transition: background 0.2s, color 0.2s;
            }
            .download-btn:hover {
                background: #e3f2fd;
                color: #1976d2;
            }
            section {
                padding: 60px 20px;
            }
            .features {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
                gap: 30px;
                margin-top: 40px;
            }
            .feature {
                background: rgba(255,255,255,0.1);
                padding: 20px;
                border-radius: 16px;
                backdrop-filter: blur(8px);
                box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                text-align: center;
            }

            .icon-badge {
                display: inline-flex;
                align-items: center;
                justify-content: center;
                width: 50px;
                height: 50px;
                margin-bottom: 12px;
                border-radius: 50%;
                background: white;
                color: #2196f3;
                font-size: 1.5rem;
                font-weight: bold;
                box-shadow: 0 4px 10px rgba(0,0,0,0.2);
            }
            .feature h3 {
                margin-bottom: 10px;
            }
            .gallery {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                gap: 15px;
                margin-top: 40px;
            }
            .gallery img {
                width: 100%;
                border-radius: 16px;
                box-shadow: 0 4px 12px rgba(0,0,0,0.3);
            }
            footer {
                margin-top: 60px;
                padding: 20px;
                font-size: 0.9rem;
                color: rgba(255,255,255,0.7);
            }
        </style>
    </head>
    <body>
        <header>
            <img src="/assets/e-cash-app.png" alt="App Icon">
            <h1>The E-Cash App</h1>
            <a href="https://github.com/fedimint/e-cash-app/releases/download/latest/e-cash-app-0.1.0+10086-2dcd5d63.apk" class="download-btn">
                Download Latest APK
            </a>
        </header>

        <div class="features">
            <div class="feature">
                <div class="icon-badge">⚡</div>
                <h3>First-class Payments</h3>
                <p>Lightning, Onchain, and E-Cash support in one app.</p>
            </div>
            <div class="feature">
                <div class="icon-badge">📧</div>
                <h3>Lightning Address</h3>
                <p>Receive payments easily with your own Lightning Address.</p>
            </div>
            <div class="feature">
                <div class="icon-badge">🔗</div>
                <h3>Nostr Wallet Connect</h3>
                <p>Connect seamlessly with apps and services via NWC.</p>
            </div>
            <div class="feature">
                <div class="icon-badge">🌐</div>
                <h3>Discover Federations</h3>
                <p>Find and join new federations using Nostr.</p>
            </div>
            <div class="feature">
                <div class="icon-badge">🛡</div>
                <h3>Automated Backup & Recovery</h3>
                <p>Your funds are safe with automatic backups and recovery options.</p>
            </div>
        </div>

        <section>
            <h2>App Showcase</h2>
            <div class="gallery">
                <img src="/assets/1.png" alt="Screenshot 1">
                <img src="/assets/2.png" alt="Screenshot 2">
                <img src="/assets/3.png" alt="Screenshot 3">
                <img src="/assets/4.png" alt="Screenshot 4">
                <img src="/assets/5.png" alt="Screenshot 5">
                <img src="/assets/6.png" alt="Screenshot 6">
                <img src="/assets/7.png" alt="Screenshot 7">
                <img src="/assets/8.png" alt="Screenshot 8">
                <img src="/assets/9.png" alt="Screenshot 9">
                <img src="/assets/10.png" alt="Screenshot 10">
                <img src="/assets/11.png" alt="Screenshot 11">
                <img src="/assets/12.png" alt="Screenshot 12">
                <img src="/assets/13.png" alt="Screenshot 13">
                <img src="/assets/14.png" alt="Screenshot 14">
                <img src="/assets/15.png" alt="Screenshot 15">
                <img src="/assets/16.png" alt="Screenshot 16">
                <img src="/assets/17.png" alt="Screenshot 17">
            </div>
        </section>
    </body>
    </html>
    "#,
    )
}
