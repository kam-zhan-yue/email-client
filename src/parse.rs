use crate::email_error::EmailError;
use crate::file;
use crate::Server;

impl<'a> Server<'a> {
    pub fn parse(&mut self, message_num: u32) -> Result<String, EmailError> {
        let fields = ["FROM", "TO", "DATE", "SUBJECT"];
        let mut header = String::new();

        // Loop through all of the fields, send a command and parse it into a header string
        for field in &fields {
            let line = self.parse_command(message_num, field)?;
            header.push_str(&line);
            header.push('\n');
        }

        // Print out the header to file and/or std out
        if self.debug {
            file::write(&header)?;
        }
        Ok(header)
    }

    fn parse_command(&mut self, message_num: u32, field: &str) -> Result<String, EmailError> {
        let header = self.fetch_header(message_num, field)?;
        if header.is_empty() {
            match field {
                "FROM" => Ok("From:".to_string()),
                "TO" => Ok("To:".to_string()),
                "DATE" => Ok("Date:".to_string()),
                "SUBJECT" => Ok("Subject: <No subject>".to_string()),
                _ => Ok(String::new()),
            }
        } else {
            let parts:  Vec<&str> = header.splitn(2, ": ").collect();
            if parts.len() != 2 {
                return Err(EmailError::InvalidHeader)
            }
            let command = parts[0];
            let message = parts[1].trim_start();
            let formatted_command = format!("{}{}",
                command.chars().next().unwrap_or_default().to_uppercase(),
                command.chars().skip(1).collect::<String>().to_lowercase());

            let formatted_header = format!("{}: {}", formatted_command, message);
            Ok(formatted_header)
        }
    }
}
