use lazy_static::lazy_static;
use syntect::dumps::from_binary;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{
    ClassedHTMLGenerator,
    ClassStyle
};
use syntect::parsing::SyntaxSet;

use crate::config::Config;

lazy_static! {
    pub static ref SYNTAX_SET: SyntaxSet = {
        let ss: SyntaxSet =
            from_binary(include_bytes!("../../../sublime/syntaxes/newlines.packdump"));
        ss
    };
    pub static ref THEME_SET: ThemeSet =
        from_binary(include_bytes!("../../../sublime/themes/all.themedump"));
}

pub enum Highlighter<'config> {
    Inline(HighlightLines<'static>, &'config SyntaxSet),
    Class(ClassedHTMLGenerator<'config>)
}

/// Returns the highlighter and whether it was found in the extra or not
pub fn get_highlighter<'config>(
    language: Option<&str>,
    config: &'config Config,
) -> Highlighter<'config> {
    let (syntax, syntax_set) = language
        .map(|lang| {
            SYNTAX_SET
                .find_syntax_by_token(lang)
                .map(|syntax| (syntax, &SYNTAX_SET as &SyntaxSet))
                .or_else(|| {
                    config
                        .extra_syntax_set
                        .as_ref()
                        .map(|extra| extra.find_syntax_by_token(lang).map(|s| (s, extra)))
                        .flatten()
                })
        })
        .flatten()
        .unwrap_or_else(|| (SYNTAX_SET.find_syntax_plain_text(), &SYNTAX_SET));

    if config.highlight_theme != "css" {
        let theme = &THEME_SET.themes[&config.highlight_theme];
        Highlighter::Inline(HighlightLines::new(syntax, theme), syntax_set)
    } else {
        Highlighter::Class(ClassedHTMLGenerator::new_with_class_style(
            syntax,
            syntax_set,
            ClassStyle::Spaced,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classed_json_trouble() {
        let ss_new = SyntaxSet::load_defaults_newlines();
        let ss_no = SyntaxSet::load_defaults_nonewlines();

        let sets = [
            &ss_new,
            &ss_no,
            &SYNTAX_SET
        ];
        for set in sets.iter() {
            let syntax = set.find_syntax_by_name("JSON").unwrap();

            let mut h = ClassedHTMLGenerator::new_with_class_style(
                &syntax,
                &set,
                ClassStyle::Spaced
            );
    
            let source = r#"
    {
        "a": {
        }
    }
            "#;
    
            for line in source.lines() {
                h.parse_html_for_line(line);
            }
            let _output = h.finalize();
            println!("Set didn't panic");
            // This test just shouldn't panic.
        }
    }
}