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

use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tracing::{debug, error, info};

use eyre::{OptionExt, eyre};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct Buffer {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl Buffer {
    fn new(socket: TcpStream) -> Buffer {
        let (read, write) = socket.into_split();
        let reader = BufReader::new(read);
        let writer = BufWriter::new(write);
        Buffer { reader, writer }
    }

    async fn read_request(&mut self) -> Result<String, eyre::ErrReport> {
        let mut request_line = String::new();
        self.reader.read_line(&mut request_line).await?;

        Ok(request_line)
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
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            match handle_connection(socket).await {
                Ok(()) => {
                    info!("connection established");
                }
                Err(error) => {
                    error!(%error ,"connection failed");
                }
            }
        });
    }
}

fn parse_http_request(http_request: &str) -> eyre::Result<(&str, &str, &str)> {
    let mut request_fields = http_request.split_whitespace();
    let method = request_fields.next().ok_or_eyre("missing HTTP method")?;
    let path = request_fields
        .next()
        .ok_or_eyre("missing path")?
        .trim_start_matches('/');
    let version = request_fields.next().ok_or_eyre("missing version")?;
    Ok((method, path, version))
}

async fn handle_connection(socket: TcpStream) -> eyre::Result<()> {
    let mut buf = Buffer::new(socket);

    let http_request = buf.read_request().await?;

    let (_method, path, version) = parse_http_request(&http_request)?;
    let path = Path::new(path);

    if !path.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(eyre!("not a normal path"));
    }

    let mut file_path = PathBuf::from("html_pages");
    file_path.push(path);
    file_path.as_mut_os_string().push(".html");

    let (status_line, mut contents) = match tokio::fs::File::open(&file_path).await {
        Ok(cte) => ("200 Ok", cte),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            debug!(path = %file_path.display(), "file doesnt exist");

            (
                "400 NOT FOUND",
                tokio::fs::File::open("html_pages/404.html").await?,
            )
        }
        Err(error) => {
            error!(%error, path = %&file_path.display(), "couldnt read the file");
            (
                "500 INTERNAL SERVER ERROR",
                tokio::fs::File::open("html_pages/500.html").await?,
            )
        }
    };

    let length = contents.metadata().await?.len();
    buf.writer
        .write_all(
            format!("{version} {status_line}\r\nContent-Length: {length}\r\n\r\n").as_bytes(),
        )
        .await?;
    tokio::io::copy(&mut contents, &mut buf.writer).await?;
    buf.writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn req() {
        use super::*;
        let http_request = "GET /hello HTTP/1.1";
        let (_method, _path, version) = parse_http_request(http_request).unwrap();
        assert_eq!("HTTP/1.1", version);
    }
}
