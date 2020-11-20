use config::highlighting::{resolve_syntax_and_theme, SyntaxAndTheme};
use config::Config;
use html_escape::encode_text_to_string;
use syntect::{
    parsing::{
        SyntaxReference,
        SyntaxSet,
        ParseState,
        ScopeStack,
        BasicScopeStackOp,
        SCOPE_REPO
    },
    highlighting::{
        Theme, 
        Color,
        Highlighter,
        HighlightState,
        HighlightIterator
    },
    html::{
        append_highlighted_html_for_styled_line,
        IncludeBackground
    }
};

use super::fence::{FenceSettings, Range};

pub struct CodeBlock<'config> {
    language: Option<String>,
    #[allow(unused)]
    line_numbers: bool,
    #[allow(unused)]
    highlight_lines: Vec<Range>,
    syntax: &'config SyntaxReference,
    syntax_set: &'config SyntaxSet,
    theme: Option<&'config Theme>,
    contents: String,
}

fn get_theme_background(theme: &Theme) -> Color {
    theme.settings.background.unwrap_or(Color::WHITE)
}

impl<'config> CodeBlock<'config> {
    pub fn new(fence_info: &str, config: &'config Config) -> Self {
        let FenceSettings {
            language, line_numbers, highlight_lines
        } = FenceSettings::new(fence_info);
        
        let SyntaxAndTheme {
            syntax, syntax_set, theme
        } = resolve_syntax_and_theme(language, config);

        let language = language.map(String::from);
            
        Self {
            language, line_numbers, highlight_lines,
            syntax, syntax_set, theme,
            contents: String::new(),
        }
    }

    pub fn add_text(&mut self, text: &str) {
        self.contents.push_str(text);
    }

    pub fn highlight(self) -> String {
        let repo = SCOPE_REPO.lock().unwrap();
        let mut highlighter = self.theme.map(|theme| {
            let highlighter = Highlighter::new(theme);
            let highlighter_state = HighlightState::new(&highlighter, ScopeStack::new());
            (highlighter, highlighter_state, theme)
        });

        let pre_styles = self.theme.map(|theme| {
            // If there's a theme (even without a background set), then set a background:
            let color = get_theme_background(theme);
            format!(" style=\"background-color:#{:02x}{:02x}{:02x};\"", color.r, color.g, color.b)
        }).unwrap_or(" class=\"code\"".into());

        let code_class = self.language.map(|lang| format!(" class=\"language-{}\"", lang)).unwrap_or("".into());

        let mut html = format!("<pre{}><code{}>", pre_styles, code_class);

        let mut parser = ParseState::new(self.syntax);
        let mut scope_stack = ScopeStack::new();
        let mut unclosed_spans = 0usize;
        for line in self.contents.split_inclusive('\n') {
            // if self.line_numbers {
            //     html += "<span class=\"code-line\">"
            // }

            let tokens = parser.parse_line(line, self.syntax_set);
            if let Some((ref highlighter, ref mut highlight_state, theme)) = highlighter.as_mut() {
                let tokens: Vec<_> = HighlightIterator::new(highlight_state, &tokens, line, highlighter).collect();
                
                append_highlighted_html_for_styled_line(
                    &tokens,
                    IncludeBackground::IfDifferent(get_theme_background(theme)),
                    &mut html
                );
            } else {
                let mut prev_i = 0usize;
                tokens.iter().for_each(|(i, op)| {
                    encode_text_to_string(&line[prev_i..*i], &mut html);
                    prev_i = *i;
                    // TODO: Handle empty text and empty spans.
                    scope_stack.apply_with_hook(op, |basic_op, _| match basic_op {
                        BasicScopeStackOp::Pop => {
                            html += "</span>";
                            unclosed_spans -= 1;
                        },
                        BasicScopeStackOp::Push(scope) => {
                            html += "<span class=\"";
                            for i in 0..(scope.len()) {
                                let atom = scope.atom_at(i as usize);
                                let atom_s = repo.atom_str(atom);
                                if i != 0 {
                                    html.push_str(" ");
                                }
                                html.push_str(atom_s);
                            }
                            html += "\">";
                            unclosed_spans += 1;
                        }
                    });
                });
                encode_text_to_string(&line[prev_i..], &mut html);
            }
            
            // if self.line_numbers {
            //     html += "</span>"
            // }
        }

        for _ in 0..unclosed_spans {
            html += "</span>";
        }

        html += "</code></pre>";
        html
    }
}
