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

fn main() {
    println!("Hello, world!");
use std::{
    fmt::format,
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use eyre::{Context, Result, eyre};
use hello::ThreadPool;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.wrap_err("couldnt receive incoming connection")?;

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    Ok(())
}
fn handle_connection(mut stream: TcpStream) -> eyre::Result<()> {
    let buf_reader = BufReader::new(&stream);
    let mut http_request = String::new();
    match buf_reader.lines().next() {
        Some(e) => http_request = e?,
        None => (),
    };
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
    stream.write_all(response.as_bytes())?;

    //let response = "HTTP/1.1 200 OK\r\n\r\n";
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn idk() {
        assert!(true);
    }
}
