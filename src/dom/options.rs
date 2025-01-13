#[derive(Debug, Clone)]
pub struct FormattingOptions {
    /// Double quotation marks or single
    pub double_quot: bool,

    /// Break tags in new line and split attributes when needed
    pub new_lines: bool,

    /// Max length used for split attributes to new lines
    pub max_len: usize,

    /// The amount of white spaces a tab is sized
    /// This will be needed to calculate the max length
    pub tab_size: u8,
}

impl FormattingOptions {
    /// Returns a congif which tries to print the out nicely readable
    pub fn pretty() -> Self {
        Self::default()
    }

    /// Returns a configurations which prints the output in a compact way
    pub fn compact() -> Self {
        Self {
            double_quot: false,
            new_lines: false,
            max_len: 0,
            tab_size: 0,
        }
    }

    /// Return the defined quotes
    pub fn quotes(&self) -> char {
        match self.double_quot {
            true => '"',
            false => '\'',
        }
    }

    /// write the depth as tab to the buffer
    pub fn fmt_depth<W>(&self, f: &mut W, depth: usize) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        // Only when depth and tab are correct we continue
        if self.tab_size == 0 || depth < self.tab_size as usize {
            return Ok(());
        }

        let tabs = depth / self.tab_size as usize;
        for _ in 0..tabs {
            write!(f, "\t")?;
        }

        Ok(())
    }
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            double_quot: false,
            new_lines: true,
            max_len: 60,
            tab_size: 4,
        }
    }
}
