use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command};

use anyhow::{Error, Result};
use fs_extra::dir::{self, CopyOptions};
use rayon::prelude::*;
use serde::Deserialize;

fn main() -> Result<()> {
    let languages = fs::read_to_string("config/languages.toml")?;
    let languages = toml::from_str::<Languages>(&languages)?.languages;

    if std::env::var("INKJET_REDOWNLOAD_LANGS").is_ok() {
        download_langs(&languages)?;
    }

    if std::env::var("INKJET_REBUILD_LANGS_MODULE").is_ok() {
        generate_langs_module(&languages)?;
    }

    languages.par_iter().for_each(Language::compile);

    Ok(())
}

fn download_langs(languages: &[Language]) -> Result<()> {
    fs::remove_dir_all("languages")?;
    fs::create_dir_all("languages/temp")?;

    Command::new("git")
        .arg("clone")
        .arg("https://github.com/nvim-treesitter/nvim-treesitter")
        .arg("languages/temp/nvim")
        .spawn()?
        .wait()?;

    languages
        .par_iter()
        .map(|lang| (lang.download(), lang))
        .try_for_each(|(child, lang)| -> Result<()> {
            child?.wait()?;

            println!("Finished downloading {}.", lang.name);

            fs::create_dir_all(format!("languages/{}/queries", lang.name))?;

            dir::copy(
                format!("languages/temp/{}/src", lang.name),
                format!("languages/{}", lang.name),
                &CopyOptions::new(),
            )?;

            let nvim_query_path = format!("languages/temp/nvim/queries/{}", lang.name);

            let query_path = match Path::new(&nvim_query_path).try_exists()? {
                false => format!("languages/temp/{}/queries", lang.name),
                true => nvim_query_path,
            };

            dir::copy(
                query_path,
                format!("languages/{}/queries", lang.name),
                &CopyOptions::new().content_only(true),
            )?;

            let _ = fs::remove_file(format!("languages/{}/src/grammar.json", lang.name));
            let _ = fs::remove_file(format!("languages/{}/src/node-types.json", lang.name));

            println!("Finished extracting {}.", lang.name);

            Ok(())
        })?;

    fs::remove_dir_all("languages/temp")?;

    Ok(())
}

fn generate_langs_module(languages: &[Language]) -> Result<()> {
    let mut module_buffer = indoc::indoc!(
        "
        #![allow(dead_code)]
        #![allow(clippy::items_after_test_module)]
        // This module is automatically generated by Inkjet.

        use tree_sitter_highlight::HighlightConfiguration;\n
    "
    )
    .to_owned();

    for lang in languages {
        lang.generate_module(&mut module_buffer);
    }

    let mut member_buffer = String::new();
    let mut from_str_buffer = String::new();
    let mut into_cfg_buffer = String::new();

    for lang in languages {
        use std::fmt::Write;

        let pretty_name = lang
            .pretty_name
            .as_deref()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| uppercase_first_char(&lang.name));

        // Add language to member list
        writeln!(&mut member_buffer, "{},", pretty_name)?;

        // Add language to config match
        writeln!(
            &mut into_cfg_buffer,
            "Self::{pretty_name} => {}::config(),",
            lang.name
        )?;

        // Add canonical language name to from_str match
        writeln!(
            &mut from_str_buffer,
            "\"{}\" => Some(Self::{}),",
            lang.name, pretty_name
        )?;

        // Add all language aliases to from_str match
        for alias in &lang.aliases {
            writeln!(
                &mut from_str_buffer,
                "\"{}\" => Some(Self::{}),",
                alias, pretty_name
            )?;
        }
    }

    let enum_definition = indoc::formatdoc! {"
        /// The set of all languages supported by Inkjet.
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum Language {{
            {member_buffer}
        }}

        impl Language {{
            /// Attempts to convert a string token (such as `rust` or `rs`) into the corresponding language.
            /// 
            /// Returns [`None`] if the language was not found.
            /// 
            /// The tokens for each language are sourced from its `name` and `aliases` keys in
            /// `config/languages.toml`.
            pub fn from_token(token: &str) -> Option<Self> {{
                match token {{
                    {from_str_buffer}
                    _ => None,
                }}
            }}

            pub(crate) fn config(&self) -> HighlightConfiguration {{
                match *self {{
                    {into_cfg_buffer}
                }}
            }}
        }}
    "};

    let mut file = File::create("src/languages.rs")?;

    write!(&mut file, "{}", &module_buffer)?;

    write!(&mut file, "{}", &enum_definition)?;

    Ok(())
}

// See https://stackoverflow.com/questions/59794375
#[derive(Debug, Deserialize)]
struct Languages {
    languages: Vec<Language>,
}

#[derive(Debug, Deserialize)]
struct Language {
    name: String,
    repo: String,
    #[serde(default)]
    aliases: Vec<String>,
    command: Option<String>,
    pretty_name: Option<String>,
}

impl Language {
    pub fn download(&self) -> Result<Child> {
        if let Some(override_command) = &self.command {
            Command::new("sh").arg("-c").arg(override_command).spawn()
        } else {
            Command::new("git")
                .arg("clone")
                .arg(&self.repo)
                .arg(&format!("languages/temp/{}", self.name))
                .spawn()
        }
        .map_err(Error::from)
    }

    pub fn compile(&self) {
        let path = Path::new("languages").join(&self.name).join("src");

        let has_scanner = path.join("scanner.c").exists() || path.join("scanner.cc").exists();
        let scanner_is_cpp = path.join("scanner.cc").exists();

        let mut build = cc::Build::new();

        let parser_path = path.join("parser.c");

        let build = build
            .include(&path)
            .flag_if_supported("-w")
            .flag_if_supported("-s")
            .flag_if_supported("-O2")
            .file(&parser_path);

        rerun_if_changed(&parser_path);

        if has_scanner && !scanner_is_cpp {
            let scanner_path = path.join("scanner.c");
            rerun_if_changed(&scanner_path);
            build.file(&scanner_path);
        } else if scanner_is_cpp {
            let mut build = cc::Build::new();

            let scanner_path = path.join("scanner.cc");
            rerun_if_changed(&scanner_path);

            build
                .cpp(true)
                .include(&path)
                .flag_if_supported("-w")
                .flag_if_supported("-s")
                .flag_if_supported("-O2")
                .file(&scanner_path)
                .compile(&format!("{}-scanner", self.name));
        }

        build.compile(&format!("{}-parser", self.name));
    }

    pub fn generate_module(&self, module_buffer: &mut String) {
        let name = &self.name;

        let highlight_path = format!("languages/{name}/queries/highlights.scm");
        let injections_path = format!("languages/{name}/queries/injections.scm");
        let locals_path = format!("languages/{name}/queries/locals.scm");

        let highlight_query = match Path::new(&highlight_path).exists() {
            false => "\"\"".to_string(),
            true => format!("include_str!(\"../{}\")", &highlight_path),
        };

        let injections_query = match Path::new(&injections_path).exists() {
            false => "\"\"".to_string(),
            true => format!("include_str!(\"../{}\")", &injections_path),
        };

        let locals_query = match Path::new(&locals_path).exists() {
            false => "\"\"".to_string(),
            true => format!("include_str!(\"../{}\")", &locals_path),
        };

        let generated_module = indoc::formatdoc!(
            "
            pub mod {name} {{
                use tree_sitter::Language;
                use tree_sitter_highlight::HighlightConfiguration;
            
                extern \"C\" {{
                    pub fn tree_sitter_{name}() -> Language;
                }}

                pub fn config() -> HighlightConfiguration {{
                    HighlightConfiguration::new(
                        unsafe {{ tree_sitter_{name}() }},
                        HIGHLIGHT_QUERY,
                        INJECTIONS_QUERY,
                        LOCALS_QUERY,
                    ).expect(\"Failed to load highlight configuration for language '{name}'!\")
                }}
            
                pub const HIGHLIGHT_QUERY: &str = {highlight_query};
                pub const INJECTIONS_QUERY: &str = {injections_query};
                pub const LOCALS_QUERY: &str = {locals_query};
            
                #[cfg(test)]
                mod tests {{
                    #[test]
                    fn grammar_loading() {{
                        let mut parser = tree_sitter::Parser::new();
                        parser
                            .set_language(unsafe {{ super::tree_sitter_{name}() }})
                            .expect(\"Grammar should load successfully.\");
                    }}
                }}
            }}
            
        "
        );

        module_buffer.push_str(&generated_module);
    }
}

fn rerun_if_changed(path: impl AsRef<Path>) {
    println!("cargo:rerun-if-changed={}", path.as_ref().to_str().unwrap());
}

fn uppercase_first_char(str: &str) -> String {
    let mut chars = str.chars();
    match chars.next() {
        None => String::new(),
        Some(char) => char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
