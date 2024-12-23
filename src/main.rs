use std::env;

mod arguments;
mod email_error;
mod fetch;
mod file;
mod helper;
mod list;
mod mime;
mod parse;
mod server;
use crate::arguments::print_args;
use crate::email_error::handle_error;
use crate::server::{Server, Streamable};
use openssl::ssl::SslStream;
use std::net::TcpStream;

fn main() {
    let debug: bool = false;
    let args: Vec<String> = env::args().skip(1).collect();
    let parsed_args = arguments::parse_args(&args);
    if debug {
        print_args(&parsed_args);
        println!();
    }
    if let Err(err) = arguments::validate_args(&parsed_args) {
        handle_error(err)
    }

    let mut stream: Option<Box<dyn Streamable>> = None;

    if parsed_args.use_tsl {
        let ssl_stream = server::create_ssl_stream(&parsed_args.server_name);

        match ssl_stream {
            Ok(stream_result) => {
                stream = Some(Box::<SslStream<TcpStream>>::new(stream_result));
            }
            Err(e) => handle_error(e),
        }
    } else {
        let tcp_stream = server::create_tcp_stream(&parsed_args.server_name);

        match tcp_stream {
            Ok(stream_result) => {
                stream = Some(Box::<TcpStream>::new(stream_result));
            }
            Err(e) => handle_error(e),
        }
    }

    let mut server = Server::new(&parsed_args.username, &parsed_args.password, debug);

    let res = server
        .connect(stream.unwrap())
        .and_then(|_| server.login())
        .and_then(|_| server.select(&parsed_args.folder));

    match res {
        Ok(_) => (),
        Err(e) => handle_error(e),
    }

    let response = match parsed_args.command.as_str() {
        "retrieve" => server.fetch(parsed_args.message_number),
        "parse" => server.parse(parsed_args.message_number),
        "mime" => server.mime(parsed_args.message_number),
        "list" => server.list(),
        _ => Ok("".to_string()),
    };

    let newline = match parsed_args.command.as_str() {
        "retrieve" => "\r\n",
        "parse" | "mime" | "list" => "",
        _ => "",
    };

    match response {
        Ok(result) => print!("{}{}", result, newline),
        Err(e) => handle_error(e),
    };

    server.shutdown();
}
