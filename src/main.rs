// Copyright 2026 SECO Mind Srl
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use tracing::{error, info};

use tokio::net::TcpListener;

use axum::{Router, extract::Path, response::Html, routing::get};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn get_hello(Path(page): Path<String>) -> Html<String> {
    info!("page: {page}");
    let path = if page == "hello" {
        format!("html_pages/{page}.html")
    } else {
        "html_pages/404.html".to_string()
    };
    info!("path: {path}");
    match tokio::fs::read_to_string(path).await {
        Ok(html) => Html(html),
        Err(_) => match tokio::fs::read_to_string("html_pages/500.html").await {
            Ok(five_hundered_file) => Html(five_hundered_file),
            Err(error) => {
                error!(%error ,"Unexpected error");
                Html("<p>Unexpected error</p>".to_string())
            }
        },
    }
}

#[tracing::instrument]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::DEBUG.into())
                .from_env_lossy(),
        )
        .try_init()?;

    //let token = CancellationToken::new();

    let app = Router::new().route("/{key}", get(get_hello));
    let listener = TcpListener::bind("127.0.0.1:7877").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn req_version() {
        use super::*;
        let response = get_hello(Path("hello".to_string())).await;

        assert_ne!(response.0, "<p>Unexpected error</p>");
    }
}
