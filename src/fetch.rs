use crate::email_error::EmailError;
use crate::file;
use crate::helper::{carriage_return, parse_string};
use crate::Server;

impl<'a> Server<'a> {
    pub fn fetch(&mut self, message_num: u32) -> Result<String, EmailError> {
        let command = format!("FETCH {} BODY.PEEK[]", message_num);
        let response = self.run_command(&command)?;
        if self.valid_response(&response) {
            let parsed_response = parse_string(&response, 1, 4);
            let carriage_response = carriage_return(&parsed_response);
            if self.debug {
                file::write(&carriage_response)?;
            }
            return Ok(carriage_response);
        } else {
            if self.debug {
                file::write(&response)?;
            }

            return Err(EmailError::MessageNotFound);
        }
    }
}
