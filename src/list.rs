use crate::email_error::EmailError;
use crate::Server;

impl<'a> Server<'a> {
    pub fn list(&mut self) -> Result<String, EmailError> {
        let command = "FETCH 1:* BODY.PEEK[HEADER.FIELDS (SUBJECT)]";
        let response = self.run_command(&command)?;
        let lines = response.split("\r\n").collect::<Vec<&str>>();
        let lines_len = lines.len();
        let mut response = String::new();
        let mut i = 0;

        while i < lines_len - 2 {
            let line = lines[i];
            i += 1;

            let number = line.as_bytes()[2] as char;

            let subject_line = get_subject_line(&lines, &mut i);

            let subject: &str;

            if subject_line.is_empty() {
                subject = "<No subject>";
            } else {
                subject = &subject_line[9..subject_line.len()];
            }

            response.push(number);
            response.push_str(": ");
            response.push_str(subject);
            response.push('\n');
        }

        Ok(response)
    }
}

fn get_subject_line(lines: &Vec<&str>, i: &mut usize) -> String {
    let mut subject = String::new();
    let mut terminating = false;

    while *i < lines.len() {
        let line = lines[*i];
        *i += 1;

        if line == "" {
            terminating = true;
            continue;
        }

        if terminating && line == ")" {
            break;
        } else {
            terminating = false;
        }

        subject.push_str(line);
    }

    subject
}
