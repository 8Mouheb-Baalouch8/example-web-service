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

use std::path::{Component, Path, PathBuf};
use std::{
    fs::{self},
    io::{self, BufReader, BufWriter, ErrorKind, prelude::*},
    net::{TcpListener, TcpStream},
};

use tracing::{debug, error, info, warn};

use example_web_service::ThreadPool;
use eyre::{Context, OptionExt, eyre};
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

        pool.execute(|| match handle_connection(stream) {
            Ok(()) => {
                info!("connection established");
            }
            Err(error) => {
                error!(%error ,"connection failed");
            }
        });
    }

    Ok(())
}

struct Buffer<'a> {
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
}

impl<'a> Buffer<'a> {
    fn new(r: BufReader<&'a TcpStream>, w: BufWriter<&'a TcpStream>) -> Buffer<'a> {
        let reader = r;
        let writer = w;
        Buffer { reader, writer }
    }

    fn read_request(&mut self) -> Result<String, io::Error> {
        match self.reader.by_ref().lines().next() {
            Some(Ok(request)) => Ok(request),
            Some(Err(error)) => Err(error),
            None => Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "cannot read the request",
            )),
        }
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

fn handle_connection(stream: TcpStream) -> eyre::Result<()> {
    let mut buf = Buffer::new(BufReader::new(&stream), BufWriter::new(&stream));

    let http_request = buf.read_request()?;

    let (_method, path, version) = parse_http_request(&http_request)?;

    let path = Path::new(path);

    if !path.components().all(|c| matches!(c, Component::Normal(_))) {
        return Err(eyre!("not a normal path"));
    }

    let mut file_path = PathBuf::from("html_pages");
    file_path.push(path);
    file_path.as_mut_os_string().push(".html");

    let (status_line, mut contents) = write_in_stream(file_path)?;

    let length = contents.metadata()?.len();
    write!(
        &mut buf.writer,
        "{version} {status_line}\r\nContent-Length: {length}\r\n\r\n"
    )?;
    io::copy(&mut contents, &mut buf.writer)?;
    Ok(())
}

fn write_in_stream(file_path: PathBuf) -> Result<(String, fs::File), eyre::Error> {
    let (status_line, contents) = match fs::File::open(&file_path) {
        Ok(cte) => ("200 Ok", cte),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            debug!(path = %file_path.display(), "file doesnt exist");

            (
                "400 NOT FOUND",
                fs::File::open("html_pages/404.html").wrap_err("couldnt find 404 page")?,
            )
        }
        Err(error) => {
            error!(%error, path = %&file_path.display(), "couldnt read the file");
            (
                "500 INTERNAL SERVER ERROR",
                fs::File::open("html_pages/500.html").wrap_err("couldnt find 500 page")?,
            )
        }
    };
    Ok((status_line.to_string(), contents))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn req_version() {
        use super::*;
        let http_request = "GET /hello HTTP/1.1";
        let (_method, _path, version) = parse_http_request(http_request).unwrap();
        assert_eq!("HTTP/1.1", version);
    }
    #[test]
    fn status_line_test() {
        let (status_line, _) = write_in_stream(PathBuf::from("html_pages/hello.html")).unwrap();
        assert_eq!(status_line, "200 Ok".to_string());
    }
}
