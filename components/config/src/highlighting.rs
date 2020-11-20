use lazy_static::lazy_static;
use syntect::{
    dumps::from_binary,
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

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

pub struct SyntaxAndTheme<'config> {
    pub syntax: &'config SyntaxReference,
    pub syntax_set: &'config SyntaxSet,
    pub theme: Option<&'config Theme>,
}

/// Returns the highlighter and whether it was found in the extra or not
pub fn resolve_syntax_and_theme<'config>(
    language: Option<&str>,
    config: &'config Config,
) -> SyntaxAndTheme<'config> {
    let theme = if config.highlight_theme != "css" {
        Some(&THEME_SET.themes[&config.highlight_theme])
    } else {
        None
    };

    language
        .and_then(|lang| {
            SYNTAX_SET
                .find_syntax_by_token(lang)
                .map(|syntax| SyntaxAndTheme {
                    syntax,
                    syntax_set: &SYNTAX_SET as &SyntaxSet,
                    theme,
                })
                .or_else(|| {
                    config.extra_syntax_set.as_ref().and_then(|extra| {
                        extra.find_syntax_by_token(lang).map(|syntax| SyntaxAndTheme {
                            syntax,
                            syntax_set: extra,
                            theme,
                        })
                    })
                })
        })
        .unwrap_or_else(|| SyntaxAndTheme {
            syntax: SYNTAX_SET.find_syntax_plain_text(),
            syntax_set: &SYNTAX_SET,
            theme,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntect::html::{ClassStyle, ClassedHTMLGenerator};
    #[test]
    fn classed_json_trouble() {
        let ss_new = SyntaxSet::load_defaults_newlines();
        let ss_no = SyntaxSet::load_defaults_nonewlines();

        let sets = [&ss_new, &ss_no, &SYNTAX_SET];
        for set in sets.iter() {
            let syntax = set.find_syntax_by_name("JSON").unwrap();

            let mut h =
                ClassedHTMLGenerator::new_with_class_style(&syntax, &set, ClassStyle::Spaced);

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
