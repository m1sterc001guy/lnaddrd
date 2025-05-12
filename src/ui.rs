use crate::AppState;
use crate::api::RegisterRequest;
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

// Add a helper function for the common <head> markup
fn common_head(title: &str) -> Markup {
    html! {
        head {
            meta charset="UTF-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title { (title) }
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/flowbite@1.7.0/dist/flowbite.min.css";
            script src="https://cdn.tailwindcss.com" {}
            script src="https://cdn.jsdelivr.net/npm/flowbite@1.7.0/dist/flowbite.min.js" {}
        }
    }
}

pub async fn register_form(State(state): State<AppState>) -> impl IntoResponse {
    let domains = state.service.list_domains().await.unwrap_or_default();
    let warning = state.config.warning.clone();
    let markup = html! {
        (DOCTYPE)
        html lang="en" {
            (common_head("Register LN Address"))
            body class="bg-gray-50 min-h-screen flex items-center justify-center" {
                div class="w-full max-w-lg mx-auto p-6 bg-white rounded-lg shadow-lg" {
                    h1 class="text-3xl font-bold mb-6 text-center text-gray-900" { "Register LN Address" }
                    @if let Some(warning) = warning {
                        div class="p-4 mb-4 text-sm text-yellow-800 rounded-lg bg-yellow-50 dark:bg-gray-800 dark:text-yellow-300" role="alert" {
                            span class="font-bold" { "Warning:" }
                            " " (maud::PreEscaped(warning))
                        }
                    }
                    form id="register-form" method="post" action="/ui/register" class="space-y-6"  {
                        div {
                            label for="domain" class="block mb-2 text-sm font-medium text-gray-900" { "Domain" }
                            select name="domain" id="domain" required class="block w-full p-2.5 border border-gray-300 rounded-lg bg-gray-50 text-gray-900 focus:ring-blue-500 focus:border-blue-500" {
                                @for domain in &domains {
                                    option value=(domain) { (domain) }
                                }
                            }
                        }
                        div {
                            label for="username" class="block mb-2 text-sm font-medium text-gray-900" { "Username" }
                            input name="username" id="username" required class="block w-full p-2.5 border border-gray-300 rounded-lg bg-gray-50 text-gray-900 focus:ring-blue-500 focus:border-blue-500" {}
                        }
                        div {
                            label for="lnurl" class="block mb-2 text-sm font-medium text-gray-900" { "LNURL" }
                            textarea name="lnurl" id="lnurl" required rows="3" class="block w-full p-2.5 border border-gray-300 rounded-lg bg-gray-50 text-gray-900 focus:ring-blue-500 focus:border-blue-500 resize-y" style="word-break: break-all;" {}
                        }
                        button type="submit" class="w-full text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" { "Register" }
                    }
                }
            }
        }
    };
    Html(markup.into_string())
}

pub async fn register_form_submit(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> impl IntoResponse {
    let req = RegisterRequest {
        domain: form.domain.clone(),
        username: form.username.clone(),
        lnurl: form.lnurl,
    };
    match state
        .service
        .register_lnaddr(&req.domain, &req.username, &req.lnurl)
        .await
    {
        Ok(_resp) => {
            // Redirect to the details page
            Redirect::to(&format!("/ui/lnaddress/{}/{}", req.domain, req.username)).into_response()
        }
        Err(e) => {
            let markup: Markup = html! {
                (DOCTYPE)
                html lang="en" {
                    (common_head("Error"))
                    body class="bg-gray-50 min-h-screen flex items-center justify-center" {
                        div class="w-full max-w-lg mx-auto p-6 bg-white rounded-lg shadow-lg" {
                            h1 class="text-2xl font-bold mb-4 text-center text-red-700" { "Error" }
                            div class="mb-6 text-center text-red-600 font-mono break-all" { (e.to_string()) }
                            div class="text-center" {
                                a href="/" class="inline-block text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" { "Back to Register" }
                            }
                        }
                    }
                }
            };
            Html(markup.into_string()).into_response()
        }
    }
}

pub async fn lnaddress_details(
    State(state): State<AppState>,
    Path((domain, username)): Path<(String, String)>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let lnaddr = format!("{username}@{domain}");
    let lnurl = state
        .service
        .get_lnaddr(&domain, &username)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    let manifest = state
        .service
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
            (common_head("LN Address Details"))
            body class="bg-gray-50 min-h-screen flex items-center justify-center" {
                div class="w-full max-w-lg mx-auto p-6 bg-white rounded-lg shadow-lg" {
                    h1 class="text-3xl font-bold mb-6 text-center text-gray-900" { "LN Address Details" }
                    div class="mb-4" {
                        p class="mb-2" { b { "Lightning Address:" } " " (lnaddr) }
                        div class="flex justify-center mb-2" { (maud::PreEscaped(lnaddr_svg)) }
                        p class="mb-2" { b { "LNURL:" } " " span class="break-all font-mono" { (lnurl) } }
                        p class="mb-2" { b { "LNURL Decoded:" } " " span class="break-all font-mono" { (lnurl.url) } }
                        p class="mb-2" { b { "LNURL Manifest:" } }
                        pre class="bg-gray-100 rounded p-2 text-xs overflow-x-auto" { (manifest_str) }
                    }
                    div class="text-center" {
                        a href="/" class="inline-block text-blue-600 hover:underline font-medium text-lg" { "Back to Register" }
                    }
                }
            }
        }
    };
    Ok(Html(markup.into_string()).into_response())
}
