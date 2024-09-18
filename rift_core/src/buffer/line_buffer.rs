/// Text buffer implementation as a list of lines
#[derive(Debug)]
pub struct LineBuffer {
    pub file_path: Option<String>,
    pub lines: Vec<String>,
}

impl LineBuffer {
    /// Create a line buffer
    pub fn new(initial_text: String, file_path: Option<String>) -> Self {
        // Split string at line endings and collect
        // into a vector of strings
        let mut lines: Vec<String> = initial_text.lines().map(String::from).collect();

        // We always want an extra empty line at
        // the end of the buffer / file
        if let Some(last) = lines.last() {
            if !last.is_empty() {
                // Last line is not empty
                lines.push("".into())
            }
        } else {
            // The buffer is empty
            lines.push("".into());
        }

        Self { file_path, lines }
    }

    /// Get text buffer content as a string
    /// with the desired EOL sequence
    pub fn get_content(&self, eol_sequence: String) -> String {
        self.lines.join(&eol_sequence)
    }

    /// Get visible lines based on scroll position, cursor position and number of visible lines
    pub fn get_visible_lines(&self, visible_lines: usize) -> &[String] {
        &self.lines.get(0..visible_lines).unwrap_or(&self.lines[0..])
    }

    /// Get visible lines with line wrap
    pub fn get_visible_lines_with_wrap(
        &self,
        visible_lines: usize,
        max_characters: usize,
        soft_wrap: bool,
    ) -> Vec<String> {
        let mut lines = vec![];
        let mut start = 0;

        for line in self.lines.get(0..visible_lines).unwrap_or(&self.lines[0..]) {
            while start < line.len() {
                let end = std::cmp::min(start + max_characters, line.len());
                lines.push(line[start..end].to_string());
                start = end;
            }
            start = 0;
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::LineBuffer;

    #[test]
    fn line_buffer_empty() {
        let buf = LineBuffer::new("".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec![""])
    }

    #[test]
    fn line_buffer_with_line_ending() {
        let buf = LineBuffer::new("\n".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec![""])
    }

    #[test]
    fn line_buffer_with_no_extra_line() {
        let buf = LineBuffer::new("Hello\nWorld".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec!["Hello", "World", "",])
    }

    #[test]
    fn line_buffer_with_extra_line() {
        let buf = LineBuffer::new("Hello\nWorld\n".into(), None);
        assert_eq!(buf.file_path, None);
        assert_eq!(buf.lines, vec!["Hello", "World", "",])
    }

    #[test]
    fn line_buffer_hard_wrap() {
        let buf = LineBuffer::new(
            "HelloWorld\nSu      ch a wo     nderful    world".into(),
            None,
        );
        assert_eq!(
            buf.get_visible_lines_with_wrap(10, 4, false),
            vec!["Hello", "World", "",]
        )
    }
}
