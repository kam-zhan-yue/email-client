use crate::email_error::EmailError;
use crate::helper::parse_string;
use openssl::ssl::{SslConnector, SslMethod, SslStream};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};

pub trait Shutdownable {
    fn shutdown_stream(&mut self, how: Shutdown) -> Result<(), std::io::Error>;
}

pub trait Streamable: Shutdownable + Read + Write {}

impl Shutdownable for TcpStream {
    fn shutdown_stream(&mut self, how: Shutdown) -> Result<(), std::io::Error> {
        self.shutdown(how)?;
        Ok(())
    }
}

impl Shutdownable for SslStream<TcpStream> {
    fn shutdown_stream(&mut self, _: Shutdown) -> Result<(), std::io::Error> {
        self.shutdown().unwrap();
        Ok(())
    }
}

impl Streamable for TcpStream {}
impl Streamable for SslStream<TcpStream> {}

pub struct Server<'a> {
    username: &'a str,
    password: &'a str,
    command: u8,
    stream: Option<Box<dyn Streamable>>,
    pub debug: bool,
}

impl<'a> Server<'a> {
    // Constructor method
    pub fn new(username: &'a str, password: &'a str, debug: bool) -> Server<'a> {
        Server {
            username,
            password,
            command: 1,
            stream: None,
            debug,
        }
    }

    // Connect method
    pub fn connect(&mut self, mut stream: Box<dyn Streamable>) -> Result<(), EmailError> {
        // Read the welcome message from the server
        let mut buffer = [0; 512];
        stream
            .read(&mut buffer)
            .expect("Failed to read from server");
        if self.debug {
            println!("Welcome Message: {}", String::from_utf8_lossy(&buffer));
        }

        self.stream = Some(stream);

        Ok(())
    }

    // Login method
    pub fn login(&mut self) -> Result<String, EmailError> {
        let command = format!("LOGIN {} {}", self.username, self.password);
        let response = self.run_command(&command)?;

        if self.debug {
            println!("Login Response: {}", response);
        }

        if self.valid_response(&response) {
            return Ok(response);
        } else {
            return Err(EmailError::LoginFailure);
        }
    }

    pub fn select(&mut self, folder: &str) -> Result<String, EmailError> {
        let command = format!("SELECT {}", folder);
        let response = self.run_command(&command)?;
        if self.debug {
            println!("Select Response: {}", response);
        }
        if self.valid_response(&response) {
            return Ok(response);
        } else {
            return Err(EmailError::FolderNotFound);
        }
    }

    pub fn valid_response(&self, response: &str) -> bool {
        // Split the response by lines and take the first line
        if let Some(last_line) = response.lines().last() {
            if last_line.contains("OK") {
                // If OK is found in the first line, return Ok(response)
                return true;
            } else if last_line.contains("NO") {
                return false;
            }
        }
        false
    }

    pub fn run_command(&mut self, command: &str) -> Result<String, EmailError> {
        // The stream must exist first
        if let Some(ref mut stream) = self.stream {
            // Create a relevant tag and increment the command index
            let tag = format!("A{:02}", self.command);
            self.command += 1;
            // Format the command appropriately
            let full_command = format!("{} {}\r\n", tag, command);
            // let full_command = command;
            if self.debug {
                print!("Sending Command: {}", full_command);
            }
            stream.write_all(full_command.as_bytes())?;

            // Read the response from the server
            let mut response = String::new();
            let mut buffer = [0; 1]; // Read one byte at a time
            let mut prev_byte = None; // To keep track of previous byte

            let mut line = Vec::<u8>::new();
            loop {
                stream.read_exact(&mut buffer)?; // Read one byte
                let byte = buffer[0];

                // Append byte to response if not null
                line.push(byte);

                // Check for \r\n termination
                if prev_byte == Some(b'\r') && byte == b'\n' {
                    unsafe {
                        let line_str = String::from_utf8_unchecked(line);

                        // Append the line to the response
                        response.push_str(&line_str);
                        // Check if the line starts with the tag, then break
                        if line_str.starts_with(&tag) {
                            break;
                        }
                    }
                    line = Vec::<u8>::new();
                }

                // Update prev_byte
                prev_byte = Some(byte);
            }
            Ok(response)
        } else {
            Err(EmailError::StreamNotConnected)
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(ref mut stream) = self.stream {
            stream
                .shutdown_stream(Shutdown::Both)
                .expect("shutdown call failed");
        }
    }

    // Removes all of the empty lines within the header
    fn unwrap_header(&self, header: &str) -> String {
        header
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn fetch_header(&mut self, message_num: u32, field: &str) -> Result<String, EmailError> {
        let command = format!("FETCH {} BODY.PEEK[HEADER.FIELDS ({})]", message_num, field);
        let response = self.run_command(&command)?;
        let header = self.process_header(&response)?;
        Ok(header)
    }

    // Function to parse and unwrap the response
    fn process_header(&self, header: &str) -> Result<String, EmailError> {
        if self.valid_response(header) {
            let parsed_response = parse_string(header, 1, 4);
            let unwrapped = self.unwrap_header(&parsed_response);
            return Ok(unwrapped);
        } else {
            println!("Message not found");
            std::process::exit(3);
        }
    }
}

pub fn create_tcp_stream(host: &str) -> Result<TcpStream, EmailError> {
    let addr = (host, 143)
        .to_socket_addrs()?
        .filter(|addr| addr.is_ipv4() || addr.is_ipv6())
        .next()
        .ok_or_else(|| EmailError::AddressNotFound)?;

    Ok(TcpStream::connect(addr)?)
}

pub fn create_ssl_stream(host: &str) -> Result<SslStream<TcpStream>, EmailError> {
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let addr = (host, 993)
        .to_socket_addrs()?
        .filter(|addr| addr.is_ipv4() || addr.is_ipv6())
        .next()
        .ok_or_else(|| EmailError::AddressNotFound)?;

    let stream = TcpStream::connect(addr)?;
    Ok(connector.connect(host, stream)?)
}
