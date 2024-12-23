#[derive(Debug)]
pub enum EmailError {
    LoginFailure,
    MessageNotFound,
    FolderNotFound,
    InvalidArguments,
    InvalidHeader,
    SafeConnection,
    SafeDisconnection,
    InvalidMimeVersion,
    MimeMissing,
    InvalidContentType,
    BoundaryParameterMissing,
    StreamNotConnected,
    SslError,
    HandshakeError,
    AddressNotFound,
}

impl From<std::io::Error> for EmailError {
    fn from(err: std::io::Error) -> EmailError {
        match err.kind() {
            // Return safe connection if it is an uncategorised error
            std::io::ErrorKind::Other => EmailError::SafeConnection,
            // Return safe disconnection otherwise
            _ => EmailError::SafeDisconnection,
        }
    }
}

impl From<openssl::error::ErrorStack> for EmailError {
    fn from(_: openssl::error::ErrorStack) -> EmailError {
        EmailError::SslError
    }
}

impl From<openssl::ssl::HandshakeError<std::net::TcpStream>> for EmailError {
    fn from(_: openssl::ssl::HandshakeError<std::net::TcpStream>) -> EmailError {
        EmailError::HandshakeError
    }
}

fn print_and_exit(message: &str, exit_code: i32) {
    println!("{}", message);
    std::process::exit(exit_code);
}

pub fn handle_error(err: EmailError) {
    use EmailError::*;

    match err {
        SafeConnection => print_and_exit("Connection terminated safely", 1),
        StreamNotConnected => print_and_exit("Stream not connected", 1),
        SslError => print_and_exit("SSL error", 1),
        HandshakeError => print_and_exit("Handshake error", 1),
        AddressNotFound => print_and_exit("Could not make connection. Invalid address", 1),
        InvalidArguments => print_and_exit("Invalid CLI Arguments", 1),
        SafeDisconnection => print_and_exit("Server disconnected unexpectedly", 2),
        LoginFailure => print_and_exit("Login failure", 3),
        MessageNotFound => print_and_exit("Message not found", 3),
        FolderNotFound => print_and_exit("Folder not found", 3),
        InvalidHeader => print_and_exit("Invalid Header", 4),
        InvalidMimeVersion => print_and_exit("Invalid MIME version. Expecting version 1.0", 4),
        MimeMissing => print_and_exit("Message does not contain MIME content", 4),
        InvalidContentType => print_and_exit(
            "Invalid Content-type header. Expecting multipart/alternative",
            4,
        ),
        BoundaryParameterMissing => print_and_exit("Boundary parameter value not present", 4),
    };
}
