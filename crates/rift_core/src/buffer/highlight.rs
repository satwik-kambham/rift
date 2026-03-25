use std::collections::HashMap;

use tracing::warn;
use tree_sitter::{Language as TSLanguage, Parser, Query};

use super::instance::{HighlightType, Language};

/// Tree sitter syntax highlight params
pub struct TreeSitterParams {
    pub parser: Parser,
    pub highlight_query: Query,
    pub capture_map: Vec<HighlightType>,
}

pub fn detect_language(file_path: &Option<String>) -> Language {
    match file_path {
        Some(path) => match std::path::Path::new(&path)
            .extension()
            .and_then(|ext| ext.to_str())
        {
            Some(extension) => match extension {
                "rsl" => Language::RSL,
                "rs" => Language::Rust,
                "py" => Language::Python,
                "md" => Language::Markdown,
                "toml" => Language::TOML,
                "nix" => Language::Nix,
                "dart" => Language::Dart,
                "html" => Language::HTML,
                "css" => Language::CSS,
                "scss" => Language::CSS,
                "js" => Language::Javascript,
                "ts" => Language::Typescript,
                "tsx" => Language::Tsx,
                "json" => Language::JSON,
                "c" => Language::C,
                "h" => Language::C,
                "cpp" => Language::CPP,
                "hpp" => Language::CPP,
                "vue" => Language::Vue,
                "zig" => Language::Zig,
                _ => Language::PlainText,
            },
            None => Language::PlainText,
        },
        None => Language::PlainText,
    }
}

pub fn build_highlight_params(language: Language) -> Option<TreeSitterParams> {
    let highlight_map: HashMap<String, HighlightType> = HashMap::from([
        ("attribute".into(), HighlightType::Red),
        ("constant".into(), HighlightType::Red),
        ("constant.builtin".into(), HighlightType::Turquoise),
        ("function.builtin".into(), HighlightType::Purple),
        ("function".into(), HighlightType::Blue),
        ("function.method".into(), HighlightType::Blue),
        ("function.macro".into(), HighlightType::Turquoise),
        ("function.special".into(), HighlightType::Turquoise),
        ("keyword".into(), HighlightType::Purple),
        ("label".into(), HighlightType::Red),
        ("operator".into(), HighlightType::Purple),
        ("property".into(), HighlightType::Yellow),
        ("punctuation".into(), HighlightType::Purple),
        ("punctuation.bracket".into(), HighlightType::Orange),
        ("punctuation.delimiter".into(), HighlightType::Orange),
        ("punctuation.special".into(), HighlightType::Purple),
        ("string".into(), HighlightType::Green),
        ("string.special".into(), HighlightType::Orange),
        ("string.escape".into(), HighlightType::Turquoise),
        ("escape".into(), HighlightType::Turquoise),
        ("comment".into(), HighlightType::Gray),
        ("comment.documentation".into(), HighlightType::Gray),
        ("tag".into(), HighlightType::Blue),
        ("tag.error".into(), HighlightType::Red),
        ("type".into(), HighlightType::Yellow),
        ("type.builtin".into(), HighlightType::Yellow),
        ("variable".into(), HighlightType::Red),
        ("variable.builtin".into(), HighlightType::Orange),
        ("variable.parameter".into(), HighlightType::Red),
        ("text.title".into(), HighlightType::Orange),
        ("text.uri".into(), HighlightType::Blue),
        ("text.reference".into(), HighlightType::Turquoise),
        ("text.literal".into(), HighlightType::Gray),
        ("constructor".into(), HighlightType::Turquoise),
        ("number".into(), HighlightType::Blue),
        ("embedded".into(), HighlightType::Purple),
        ("constructor".into(), HighlightType::Turquoise),
        ("local.definition".into(), HighlightType::Blue),
        ("module".into(), HighlightType::Blue),
    ]);

    fn new_highlight_params(
        ts_language: impl Into<TSLanguage>,
        name: &str,
        highlights: &str,
        highlight_map: &HashMap<String, HighlightType>,
    ) -> Option<TreeSitterParams> {
        let ts_language: TSLanguage = ts_language.into();

        let mut parser = Parser::new();
        if let Err(err) = parser.set_language(&ts_language) {
            warn!(%err, language = name, "Failed to set parser language");
            return None;
        }

        let highlight_query = match Query::new(&ts_language, highlights) {
            Ok(q) => q,
            Err(err) => {
                warn!(%err, language = name, "Failed to create highlight query");
                return None;
            }
        };

        let capture_map: Vec<HighlightType> = highlight_query
            .capture_names()
            .iter()
            .map(|name| {
                highlight_map
                    .get(*name)
                    .copied()
                    .unwrap_or(HighlightType::None)
            })
            .collect();

        Some(TreeSitterParams {
            parser,
            highlight_query,
            capture_map,
        })
    }

    match language {
        Language::Rust => new_highlight_params(
            tree_sitter_rust::LANGUAGE,
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::RSL => new_highlight_params(
            tree_sitter_rust::LANGUAGE,
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::Python => new_highlight_params(
            tree_sitter_python::LANGUAGE,
            "python",
            tree_sitter_python::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::Markdown => new_highlight_params(
            tree_sitter_md::LANGUAGE,
            "md",
            tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
            &highlight_map,
        ),
        Language::Nix => new_highlight_params(
            tree_sitter_nix::LANGUAGE,
            "nix",
            tree_sitter_nix::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::Dart => new_highlight_params(
            tree_sitter_dart::language(),
            "dart",
            tree_sitter_dart::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::Zig => new_highlight_params(
            tree_sitter_zig::LANGUAGE,
            "zig",
            tree_sitter_zig::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::HTML => new_highlight_params(
            tree_sitter_html::LANGUAGE,
            "html",
            tree_sitter_html::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::CSS => new_highlight_params(
            tree_sitter_css::LANGUAGE,
            "css",
            tree_sitter_css::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::Javascript => new_highlight_params(
            tree_sitter_javascript::LANGUAGE,
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            &highlight_map,
        ),
        Language::Typescript => new_highlight_params(
            tree_sitter_javascript::LANGUAGE,
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            &highlight_map,
        ),
        Language::Tsx => new_highlight_params(
            tree_sitter_javascript::LANGUAGE,
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            &highlight_map,
        ),
        Language::Vue => new_highlight_params(
            tree_sitter_javascript::LANGUAGE,
            "javascript",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            &highlight_map,
        ),
        Language::JSON => new_highlight_params(
            tree_sitter_json::LANGUAGE,
            "json",
            tree_sitter_json::HIGHLIGHTS_QUERY,
            &highlight_map,
        ),
        Language::C => new_highlight_params(
            tree_sitter_c::LANGUAGE,
            "c",
            tree_sitter_c::HIGHLIGHT_QUERY,
            &highlight_map,
        ),
        Language::CPP => {
            let highlight_query =
                tree_sitter_c::HIGHLIGHT_QUERY.to_string() + tree_sitter_cpp::HIGHLIGHT_QUERY;
            new_highlight_params(
                tree_sitter_cpp::LANGUAGE,
                "cpp",
                &highlight_query,
                &highlight_map,
            )
        }
        _ => None,
    }
}
