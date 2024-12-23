pub fn parse_string(input: &str, remove_start: usize, remove_end: usize) -> String {
    // Split the input string into lines
    let mut lines: Vec<&str> = input.split("\r\n").collect();
    let total_lines = lines.len() as usize;

    // Calculate the number of lines to remove from the start and end
    let start_to_remove = remove_start.min(total_lines);
    let end_to_remove = remove_end.min(total_lines);

    // Remove lines from the start
    lines.drain(0..start_to_remove);

    // Remove lines from the end
    lines.drain(total_lines - end_to_remove..);

    // Join the remaining lines into a single string
    lines.join("\n")
}

pub fn carriage_return(input: &str) -> String {
    input.replace("\n", "\r\n")
}