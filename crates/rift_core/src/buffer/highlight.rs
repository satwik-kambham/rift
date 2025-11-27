use std::collections::HashMap;

use tree_sitter_highlight::HighlightConfiguration;

use super::instance::{HighlightType, Language};

/// Tree sitter syntax highlight params
pub struct TreeSitterParams {
    pub language_config: HighlightConfiguration,
    pub highlight_map: HashMap<String, HighlightType>,
    pub highlight_names: Vec<String>,
}

pub fn detect_language(file_path: &Option<String>) -> Language {
    match file_path {
        Some(path) => match std::path::Path::new(&path).extension() {
            Some(extension) => match extension.to_str().unwrap() {
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
    let highlight_names: Vec<String> = highlight_map.keys().map(|key| key.to_string()).collect();

    let language_config = match language {
        Language::Rust => Some(
            HighlightConfiguration::new(
                tree_sitter_rust::LANGUAGE.into(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            )
            .unwrap(),
        ),
        Language::RSL => Some(
            HighlightConfiguration::new(
                tree_sitter_rust::LANGUAGE.into(),
                "rust",
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                tree_sitter_rust::INJECTIONS_QUERY,
                "",
            )
            .unwrap(),
        ),
        Language::Python => Some(
            HighlightConfiguration::new(
                tree_sitter_python::LANGUAGE.into(),
                "python",
                tree_sitter_python::HIGHLIGHTS_QUERY,
                "",
                "",
            )
            .unwrap(),
        ),
        Language::Markdown => Some(
            HighlightConfiguration::new(
                tree_sitter_md::LANGUAGE.into(),
                "md",
                tree_sitter_md::HIGHLIGHT_QUERY_BLOCK,
                tree_sitter_md::INJECTION_QUERY_BLOCK,
                "",
            )
            .unwrap(),
        ),
        Language::Nix => Some(
            HighlightConfiguration::new(
                tree_sitter_nix::LANGUAGE.into(),
                "nix",
                tree_sitter_nix::HIGHLIGHTS_QUERY,
                "",
                "",
            )
            .unwrap(),
        ),
        Language::Dart => Some(
            HighlightConfiguration::new(
                tree_sitter_dart::language(),
                "dart",
                tree_sitter_dart::HIGHLIGHTS_QUERY,
                "",
                "",
            )
            .unwrap(),
        ),
        Language::HTML => Some(
            HighlightConfiguration::new(
                tree_sitter_html::LANGUAGE.into(),
                "html",
                tree_sitter_html::HIGHLIGHTS_QUERY,
                tree_sitter_html::INJECTIONS_QUERY,
                "",
            )
            .unwrap(),
        ),
        Language::CSS => Some(
            HighlightConfiguration::new(
                tree_sitter_css::LANGUAGE.into(),
                "css",
                tree_sitter_css::HIGHLIGHTS_QUERY,
                "",
                "",
            )
            .unwrap(),
        ),
        Language::Javascript => Some(
            HighlightConfiguration::new(
                tree_sitter_javascript::LANGUAGE.into(),
                "javascript",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTIONS_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
            .unwrap(),
        ),
        Language::Typescript => Some(
            HighlightConfiguration::new(
                tree_sitter_javascript::LANGUAGE.into(),
                "javascript",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTIONS_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
            .unwrap(),
        ),
        Language::Tsx => Some(
            HighlightConfiguration::new(
                tree_sitter_javascript::LANGUAGE.into(),
                "javascript",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTIONS_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
            .unwrap(),
        ),
        Language::Vue => Some(
            HighlightConfiguration::new(
                tree_sitter_javascript::LANGUAGE.into(),
                "javascript",
                tree_sitter_javascript::HIGHLIGHT_QUERY,
                tree_sitter_javascript::INJECTIONS_QUERY,
                tree_sitter_javascript::LOCALS_QUERY,
            )
            .unwrap(),
        ),
        Language::JSON => Some(
            HighlightConfiguration::new(
                tree_sitter_json::LANGUAGE.into(),
                "json",
                tree_sitter_json::HIGHLIGHTS_QUERY,
                "",
                "",
            )
            .unwrap(),
        ),
        Language::C => Some(
            HighlightConfiguration::new(
                tree_sitter_c::LANGUAGE.into(),
                "c",
                tree_sitter_c::HIGHLIGHT_QUERY,
                "",
                "",
            )
            .unwrap(),
        ),
        Language::CPP => Some(
            HighlightConfiguration::new(
                tree_sitter_cpp::LANGUAGE.into(),
                "cpp",
                &(tree_sitter_c::HIGHLIGHT_QUERY.to_string() + tree_sitter_cpp::HIGHLIGHT_QUERY),
                "",
                "",
            )
            .unwrap(),
        ),
        _ => None,
    };

    language_config.map(|mut language_config| {
        language_config.configure(&highlight_names);

        TreeSitterParams {
            language_config,
            highlight_map,
            highlight_names,
        }
    })
}
