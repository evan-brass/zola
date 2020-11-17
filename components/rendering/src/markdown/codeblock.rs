use config::highlighting::{get_highlighter, Highlighter};
use config::Config;
use syntect::html::{
    append_highlighted_html_for_styled_line,
    IncludeBackground
};

use super::fence::FenceSettings;

pub struct CodeBlock<'config> {
    highlighter: Highlighter<'config>,
    background: IncludeBackground,
    contents: String,
}

impl<'config> CodeBlock<'config> {
    pub fn new(fence_info: &str, config: &'config Config, background: IncludeBackground) -> Self {
        let fence_info = FenceSettings::new(fence_info);
        
        let highlighter = get_highlighter(fence_info.language, config);
            
        Self {
            highlighter,
            background,
            contents: String::new(),
        }
    }

    pub fn add_text(&mut self, text: &str) {
        self.contents.push_str(text);
    }

    pub fn highlight(self) -> String {
        // TODO: Learn about highlighting ranges and line numbers and then bring those back for both.
        match self.highlighter {
            Highlighter::Inline(mut highlighter, syntax_set) => {
                let mut html = String::new();
                for line in self.contents.lines() {
                    append_highlighted_html_for_styled_line(
                        &highlighter.highlight(line, syntax_set),
                        self.background,
                        &mut html
                    );
                    html.push('\n');
                }
        
                html
            },
            Highlighter::Class(mut chg) => {
                for line in self.contents.lines() {
                    chg.parse_html_for_line(line);
                }
                chg.finalize()
            }
        }
    }
}

/// This is an index of a character in a `&[(Style, &'b str)]`. The `vec_idx` is the
/// index in the slice, and `str_idx` is the byte index of the character in the
/// corresponding string slice.
///
/// The `Ord` impl on this type sorts lexiographically on `vec_idx`, and then `str_idx`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StyledIdx {
    vec_idx: usize,
    str_idx: usize,
}
