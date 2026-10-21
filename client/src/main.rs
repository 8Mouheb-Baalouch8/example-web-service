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

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    email: String,
    password: String,
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

    let client = Client::new();

    let _response = client.get("http://127.0.0.1:7877/hello").send().await?;

    let msg = Message {
        email: "mouhebbbaalouch@gmail.com".to_string(),
        password: "123456789".to_string(),
    };

    let credentials_response = client
        .post("http://127.0.0.1:7877/credentials")
        .json(&msg)
        .send()
        .await?;

    info!("Status: {}", credentials_response.status());

    info!("Headers: {:#?}", credentials_response.headers());

    let body = credentials_response.text().await?;

    info!("credentials_response body:\n{}", body);

    Ok(())
}
