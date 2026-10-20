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

use std::{
    fs,
    io::{BufReader, BufWriter, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use tracing::{debug, info};

use example_web_service::ThreadPool;
use eyre::Context;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tracing::instrument]
fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::Level::DEBUG.into())
                .from_env_lossy(),
        )
        .try_init()?;

    let listener = TcpListener::bind("127.0.0.1:7878")?;

    let addr = listener.local_addr()?;
    info!("listening on http://{addr}");

    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.wrap_err("couldnt receive incoming connection")?;
        pool.execute(|| {
            if handle_connection(stream).is_ok() {
                info!("connection established");
            } else {
                debug!("connection failed");
            }
        });
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) -> eyre::Result<()> {
    let buf_reader = BufReader::new(&stream);
    let mut buf_writer = BufWriter::new(&stream);
    let mut http_request = String::new();
    if let Some(e) = buf_reader.lines().next() {
        http_request = e?
    }
    write_in_stream(&mut buf_writer, http_request)?;
    Ok(())
}

fn write_in_stream<T>(writer: &mut T, http_request: String) -> Result<(), eyre::Error>
where
    T: Write,
{
    let (status_line, filename) = match &http_request[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(10));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    let contents = fs::read_to_string(filename)?;
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    writer.write_all(response.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn http_req_test() {
        let mut written_response = Vec::new();
        let http_request = "GET / HTTP/1.1".to_string();
        let http_response = "HTTP/1.1 200 OK".as_bytes();
        write_in_stream(&mut written_response, http_request).unwrap();
        assert_eq!(&written_response[..15], http_response);
    }
}
