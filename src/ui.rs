use crate::api::RegisterRequest;
use crate::service::LnaddrService;
use axum::{
    Form,
    extract::Path,
    extract::State,
    response::{Html, IntoResponse, Redirect},
};
use maud::{DOCTYPE, Markup, html};
use qrcode::QrCode;
use qrcode::render::svg;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterForm {
    domain: String,
    username: String,
    lnurl: String,
}

pub async fn register_form(State(service): State<LnaddrService>) -> impl IntoResponse {
    let domains = service.list_domains().await.unwrap_or_default();
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Register LN Address" }
            }
            body {
                h1 { "Register LN Address" }
                form id="register-form" method="post" action="/ui/register"  {
                    label { "Domain: "
                        select name="domain" required {
                            @for domain in &domains {
                                option value=(domain) { (domain) }
                            }
                        }
                    }
                    br;
                    label { "Username: " input name="username" required; }
                    br;
                    label { "LNURL: " input name="lnurl" required; }
                    br;
                    button type="submit" { "Register" }
                }
            }
        }
    };
    Html(markup.into_string())
}

pub async fn register_form_submit(
    State(service): State<LnaddrService>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    let req = RegisterRequest {
        domain: form.domain.clone(),
        username: form.username.clone(),
        lnurl: form.lnurl,
    };
    match service
        .register_lnaddr(&req.domain, &req.username, &req.lnurl)
        .await
    {
        Ok(_resp) => {
            // Redirect to the details page
            Redirect::to(&format!("/ui/lnaddress/{}/{}", req.domain, req.username)).into_response()
        }
        Err(e) => {
            let markup: Markup = html! {
                div style="color:red" { "Error: " (e.to_string()) }
            };
            Html(markup.into_string()).into_response()
        }
    }
}

pub async fn lnaddress_details(
    State(service): State<LnaddrService>,
    Path((domain, username)): Path<(String, String)>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let lnaddr = format!("{username}@{domain}");
    let lnurl = service
        .get_lnaddr(&domain, &username)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    let manifest = service
        .get_lnaddr_manifest(&domain, &username)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .expect("If LNURL is registered, manifest should be present");
    let manifest_str = serde_json::to_string_pretty(&manifest).unwrap();

    let lnaddr_svg = {
        let lnaddr_code = QrCode::new(&lnaddr).unwrap();
        lnaddr_code
            .render::<svg::Color>()
            .min_dimensions(256, 256)
            .build()
    };

    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "LN Address Details" }
            }
            body {
                h1 { "LN Address Details" }
                p { b { "Lightning Address:" } " " (lnaddr) }
                div { (maud::PreEscaped(lnaddr_svg)) }
                p { b { "LNURL:" } " " (lnurl) }
                p { b { "LNURL Decoded:" } " " (lnurl.url) }
                p { b { "LNURL Manifest:" } " "  pre { (manifest_str) } }
                p { a href="/" { "Back to Register" } }
            }
        }
    };
    Ok(Html(markup.into_string()).into_response())
}
