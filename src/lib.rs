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
        r###"
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
                scroll-behavior: smooth;
            }
            header { padding: 60px 20px 40px; }
            header img {
                width: 120px; height: 120px; border-radius: 24px;
                box-shadow: 0 8px 20px rgba(0,0,0,0.3);
            }
            header h1 { margin-top: 20px; font-size: 2.5rem; font-weight: bold; }
            .download-btn {
                display: inline-block; margin-top: 20px; padding: 14px 28px;
                background: white; color: #2196f3; font-size: 1.2rem; border-radius: 30px;
                text-decoration: none; font-weight: bold; transition: background .2s, color .2s;
            }
            .download-btn:hover { background: #e3f2fd; color: #1976d2; }
            section { padding: 60px 20px; }
            h2 { font-size: 2rem; margin-bottom: 20px; }

            .features {
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
                gap: 30px; margin-top: 40px;
            }
            .feature {
                background: rgba(255,255,255,0.1);
                padding: 20px; border-radius: 16px; backdrop-filter: blur(8px);
                box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                text-align: center; cursor: pointer; transition: transform .2s;
                text-decoration: none; color: inherit;
            }
            .feature:hover { transform: translateY(-5px); }
            .icon-badge {
                display: inline-flex; align-items: center; justify-content: center;
                width: 50px; height: 50px; margin-bottom: 12px; border-radius: 50%;
                background: white; color: #2196f3; font-size: 1.5rem; font-weight: bold;
                box-shadow: 0 4px 10px rgba(0,0,0,0.2);
            }

            .details {
                text-align: left; max-width: 900px;
                margin: 20px auto 0 auto;
            }
            .details h3 {
                margin-bottom: 10px;
                font-size: 1.6rem;
                display: flex;
                align-items: center;
                gap: 12px;
            }

            .details h3 .icon-badge {
                flex-shrink: 0;
                width: 40px;
                height: 40px;
                font-size: 1.2rem;
            }

            .details p { margin-bottom: 20px; }
            .detail-images {
                display: flex; flex-wrap: wrap; gap: 16px;
            }
            .detail-images img {
                width: 100%;
                max-width: 280px;
                flex: 1 1 220px;

                border-radius: 24px; /* smoother curves */
                box-shadow: 0 8px 20px rgba(0,0,0,0.4);

                background: linear-gradient(145deg, #ffffff 0%, #f0f0f0 100%);
                padding: 8px; /* like a frame */
                
                transition: transform 0.3s ease, box-shadow 0.3s ease;
                cursor: pointer;
            }

            .detail-images img:hover {
                transform: scale(1.05);
                box-shadow: 0 12px 30px rgba(0,0,0,0.6);
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

        <section>
            <h2>Features</h2>
            <div class="features">
                <a href="#payments" class="feature">
                    <div class="icon-badge">⚡</div>
                    <h3>Choose your payment method</h3>
                    <p>Lightning, Onchain, and E-Cash support.</p>
                </a>
                <a href="#lnaddress" class="feature">
                    <div class="icon-badge">📧</div>
                    <h3>Lightning Address</h3>
                    <p>Claim your per-federation Lightning Address.</p>
                </a>
                <a href="#nwc" class="feature">
                    <div class="icon-badge">🔗</div>
                    <h3>Nostr Wallet Connect</h3>
                    <p>Connect The E-Cash to NWC compatible apps for easy zaps.</p>
                </a>
                <a href="#discover" class="feature">
                    <div class="icon-badge">🌐</div>
                    <h3>Discover Federations</h3>
                    <p>Find new federations using Nostr.</p>
                </a>
                <a href="#recover" class="feature">
                    <div class="icon-badge">🛡</div>
                    <h3>Backup & Recovery</h3>
                    <p>Keep your e-cash safe using a familiar seed phrase.</p>
                </a>
            </div>
        </section>

        <!-- Detailed sections (click cards to scroll here) -->
        <section id="payments" class="details">
            <h3><span class="icon-badge">⚡</span>Choose your payment methods</h3>
            <p>The E-Cash app has full support for Lightning, Onchain, and E-Cash payments in a single unified wallet.</p>
            <div class="detail-images">
                <img src="/assets/withdraw_ecash.png" alt="Payments 1">
                <img src="/assets/redeem_ecash.png" alt="Payments 2">
                <img src="/assets/receive.png" alt="Payments 3">
                <img src="/assets/notes.png" alt="Payments 4">
                <img src="/assets/lightning_request.png" alt="Payments 5">
                <img src="/assets/lightning_receive.png" alt="Payments 6">
                <img src="/assets/ecash_receive.png" alt="Payments 7">
                <img src="/assets/addresses.png" alt="Payments 8">
                <img src="/assets/deposit.png" alt="Payments 9">
            </div>
        </section>

        <section id="lnaddress" class="details">
            <h3><span class="icon-badge">📧</span>Lightning Address</h3>
            <p>Receive payments with your personal per-federation Lightning Address.</p>
            <div class="detail-images">
                <img src="/assets/3.png" alt="Lightning Address 1">
                <img src="/assets/4.png" alt="Lightning Address 2">
            </div>
        </section>

        <section id="nwc" class="details">
            <h3><span class="icon-badge">🔗</span>Nostr Wallet Connect</h3>
            <p>Connect The E-Cash App to Nostr apps instantly using Nostr Wallet Connect for easy zaps.</p>
            <div class="detail-images">
                <img src="/assets/nostr_relays.png" alt="NWC 1">
            </div>
        </section>

        <section id="discover" class="details">
            <h3><span class="icon-badge">🌐</span>Discover Federations</h3>
            <p>The E-Cash app using NIP87 to find new joinable federations.</p>
            <div class="detail-images">
                <img src="/assets/discover.png" alt="Federations 1">
                <img src="/assets/preview.png" alt="Federations 2">
            </div>
        </section>

        <section id="recover" class="details">
            <h3><span class="icon-badge">🛡</span>Automated Backup & Recovery</h3>
            <p>The E-Cash App supports backup recovery using a familiar seed phrase. The user's federations are also encrypted and backed up to Nostr, enabling automated recovery.</p>
            <div class="detail-images">
                <img src="/assets/9.png" alt="Backup 1">
                <img src="/assets/10.png" alt="Backup 2">
            </div>
        </section>
    </body>
    </html>
    "###,
    )
}
