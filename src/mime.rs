use crate::email_error::EmailError;
use crate::Server;

impl<'a> Server<'a> {
    pub fn mime(&mut self, message_num: u32) -> Result<String, EmailError> {
        // check mime version
        let response = self.fetch_header(message_num, "MIME-VERSION")?;

        if response.is_empty() {
            return Err(EmailError::MimeMissing);
        } else if response.to_lowercase() != "mime-version: 1.0" {
            return Err(EmailError::InvalidMimeVersion);
        }

        let response = self.fetch_header(message_num, "CONTENT-TYPE")?;
        let boundary_parameter = parse_content_type(response)?;

        let body = self.fetch(message_num)?;
        let response = parse_mime_from_body(body, boundary_parameter);

        // do the slice to remove the trailing newline
        Ok(String::from(&response[..response.len() - 2]))
    }
}

fn parse_content_type(content_header: String) -> Result<String, EmailError> {
    if &content_header[..=35].to_lowercase() != "content-type: multipart/alternative;" {
        return Err(EmailError::InvalidContentType);
    }

    if &content_header[37..=45].to_lowercase() != "boundary=" {
        return Err(EmailError::BoundaryParameterMissing);
    }

    let mut quoted = false;
    let mut escaped = false;
    let mut parsed_content_header = String::new();
    let mut index = 46;

    loop {
        if index == content_header.len() {
            break;
        }

        let this_char = content_header.as_bytes()[index] as char;

        let end_of_boundary =
            (!quoted && !escaped && this_char == ';') || (quoted && !escaped && this_char == '"');

        if end_of_boundary {
            break;
        }

        if index == 46 && this_char == '"' {
            quoted = true;
            index += 1;
            continue;
        }

        if !escaped && this_char == '\\' {
            escaped = true;
            index += 1;
            continue;
        }

        parsed_content_header.push(this_char);
        index += 1;
    }

    Ok(parsed_content_header)
}

fn parse_mime_from_body(body: String, boundary_parameter: String) -> String {
    let mut response = String::new();
    let mut in_mime = false;
    let mut check_content_type = false;
    let mut check_charset = false;
    let mut valid_content_type = false;
    let mut valid_transfer_encoding = false;

    for line in body.split("\r\n") {
        // check start of mime section
        if !in_mime && line == format!("--{}", boundary_parameter) {
            check_content_type = true;
            continue;
        }

        // check end of mime section
        if line == format!("--{}--", boundary_parameter)
            || (in_mime && line == format!("--{}", boundary_parameter))
        {
            break;
        }

        if check_content_type {
            let lowered_line = line.to_lowercase();

            if check_charset {
                if lowered_line.trim() == "charset=utf-8"
                    || lowered_line.trim() == "charset=\"utf-8\""
                {
                    valid_content_type = true;
                }

                check_charset = false;
                continue;
            }

            if lowered_line.starts_with("content-type:") {
                if lowered_line == "content-type: text/plain; charset=utf-8" {
                    valid_content_type = true;
                } else if lowered_line.contains("content-type: text/plain;")
                    && !lowered_line.contains("charset")
                {
                    check_charset = true;
                } else {
                    valid_content_type = false;
                    valid_transfer_encoding = false;
                    check_content_type = false;
                }

                continue;
            }

            if lowered_line.starts_with("content-transfer-encoding:") {
                let encoding = &lowered_line[27..lowered_line.len()];

                valid_transfer_encoding = match encoding {
                    "quoted-printable" | "7bit" | "8bit" => true,
                    _ => false,
                };

                if !valid_transfer_encoding {
                    valid_content_type = false;
                    check_content_type = false;
                }

                continue;
            }

            if lowered_line.is_empty() {
                if valid_content_type && valid_transfer_encoding {
                    in_mime = true;
                }

                valid_content_type = false;
                valid_transfer_encoding = false;
                check_content_type = false;
                continue;
            }
        }

        if in_mime {
            response.push_str(line);
            response.push_str("\r\n");
        }
    }

    response
}
