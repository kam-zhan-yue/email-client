use crate::email_error::EmailError;

    pub struct Args {
        pub folder: String,
        pub username: String,
        pub password: String,
        pub message_number: u32,
        pub command: String,
        pub server_name: String,
        pub use_tsl: bool,
    }

pub fn parse_args(args: &[String]) -> Args {
    let mut parsed_args = Args {
        folder: String::from("INBOX"),
        username: String::new(),
        password: String::new(),
        message_number: 0,
        command: String::new(),
        server_name: String::new(),
        use_tsl: false,
    };

    let mut iter = args.iter().peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-f" => {
                if let Some(val) = iter.next() {
                    parsed_args.folder = parse_folder(val);
                } else {
                    eprintln!("Error: -f flag requires a value.");
                    std::process::exit(1);
                }
            }
            "-u" => {
                if let Some(val) = iter.next() {
                    parsed_args.username = val.to_string();
                } else {
                    eprintln!("Error: -u flag requires a value.");
                    std::process::exit(1);
                }
            }
            "-p" => {
                if let Some(val) = iter.next() {
                    parsed_args.password = val.to_string();
                } else {
                    eprintln!("Error: -p flag requires a value.");
                    std::process::exit(1);
                }
            }
            "-n" => {
                if let Some(val) = iter.next() {
                    if let Ok(num) = val.parse::<u32>() {
                        parsed_args.message_number = num;
                    } else {
                        eprintln!("Error: Message number must be an unsigned integer.");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: -n flag requires a value.");
                    std::process::exit(1);
                }
            }
            "retrieve" | "parse" | "mime" | "list" => {
                parsed_args.command = arg.to_string();
            }
            "-t" => {
                parsed_args.use_tsl = true;
            }
            _ => {
                parsed_args.server_name = arg.to_string();
            }
        }
    }

    parsed_args
}

pub fn validate_args(args: &Args) -> Result<(), EmailError> {
    if validate_string(&args.folder) {
        return Err(EmailError::InvalidArguments);
    }
    if validate_string(&args.username) {
        return Err(EmailError::InvalidArguments);
    }
    if validate_string(&args.password) {
        return Err(EmailError::InvalidArguments);
    }
    if validate_string(&args.command) {
        return Err(EmailError::InvalidArguments);
    }
    if validate_string(&args.server_name) {
        return Err(EmailError::InvalidArguments);
    }
    Ok(())
}

fn validate_string(s: &str) -> bool {
    s.is_empty() || s.contains('\r') || s.contains('\n')
}

pub fn print_args(args: &Args) -> () {
    println!("Parsed helper:");
    println!("Folder: {:?}", args.folder);
    println!("Username: {:?}", args.username);
    println!("Password: {:?}", args.password);
    println!("Message Number: {:?}", args.message_number);
    println!("Command: {:?}", args.command);
    println!("Server Name: {:?}", args.server_name);
    println!("Using TSL: {:?}", args.use_tsl);
}

fn parse_folder(folder: &str) -> String {
    if folder.contains(' ') {
        format!("\"{}\"", folder)
    } else {
        folder.to_string()
    }
}
