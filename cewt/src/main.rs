use std::{ffi::OsString, path::PathBuf, str::FromStr};
use bpaf::Bpaf;
use cli::{recursive_template_search, bundle::{do_bundle_spa, IncludeElementChecker}};
use color_eyre::eyre::Result;
use lazy_regex::regex_captures;

mod cli;
pub mod workarounds;
pub mod consts;

use crate::cli::codegen::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
/// ARitz's Custom Element Template Engine With Markdown
pub(crate) enum CliAction {
	#[bpaf(command("codegen"))]
	/// Recursively generates boilerplate code for your custom elements.
	Codegen {
		/// Folder names to exclude, defaults to node_modules.
		#[bpaf(argument("FOLDER_NAME"), short, long)]
		exclude: Vec<OsString>,
		/// Include HTML snippet in TypeScript output instead of assuming the template exists in the DOM
		#[bpaf(short('I'), long)]
		inline_html: bool,
		/// Have generated code contain helpers and "known" properties for HTMLFormElements
		#[bpaf(short('F'), long)]
		extended_form_controls: bool,
		/// Custom elements to use in the mapping, the following formats are accepted:
		/// 
		/// <custom-tag-name> CustomClassName from package_name
		/// 
		/// <tag-name is="custom-tag-name"> CustomClassName from package_name
		#[bpaf(argument("CUSTOM_ELEMENT_DEFINITION"), short, long)]
		external_custom_element: Vec<CliCustomElement>,
		/// Folder to scan for HTML template fragments and generate accompanying code.
		#[bpaf(positional("PATH"))]
		path: PathBuf,
	},
	#[bpaf(command("bundle-single"))]
	/// Bundles all template elements for a single-page application.
	BundleSinglePageApp {
		/// File name for the template bundle.
		#[bpaf(argument("PATH"), short, long)]
		output_file: PathBuf,
		/// Elements to include in the bundle. By default, all elements will be included.
		#[bpaf(argument("ELEMENT"), short, long)]
		include: Vec<String>,
		/// Elements to exclude from the bundle. By default, no elements will be excluded. 
		#[bpaf(argument("ELEMENT"), short, long)]
		exclude: Vec<String>,
		/// Folder to scan for HTML template fragments. Must contain a main_template.html.
		#[bpaf(positional("PATH"))]
		input_fragments: PathBuf
	}
}

fn trim_quotes(str: &str) -> &str {
	if str.starts_with('"') && str.ends_with('"') {
		return &str[1..{str.len() - 1}];
	} else if str.starts_with('\'') && str.ends_with('\'') {
		return &str[1..{str.len() - 1}];
	}
	return str;
}

#[derive(Debug, Clone, Bpaf)]
pub(crate) struct CliCustomElement {
	#[bpaf(argument("HTML_TAG"), short, long)]
	tag: String,
	#[bpaf(argument("HTML_TAG"), short, long)]
	extends: Option<String>,
	#[bpaf(argument("CLASS_NAME"), short, long)]
	class_name: String,
	#[bpaf(argument("PKG_NAME"), short, long)]
	package: Option<String>
}
impl FromStr for CliCustomElement {
	type Err = color_eyre::eyre::Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let Some((_, tag, extends, class_name, package)) = regex_captures!(
			r#"^<(\S+?)\s*?(?:is=(\S+?))?>\s*?(\S+?)(?:\s+?from\s*?(\S+?))?\s*?$"#,
			s
		) else {
			return Err(color_eyre::eyre::Error::msg("Argument does not conform to the format: <tag-name> ClassName from package_name"));
		};
		let extends = trim_quotes(extends);
		let package = trim_quotes(package);
		Ok(
			CliCustomElement {
				tag: tag.into(),
				extends: if extends.len() == 0 { None } else { Some(extends.into()) },
				class_name: class_name.into(),
				package: if package.len() == 0 { None } else { Some(package.into()) },
			}
		)
	}
}

fn main() -> Result<()> {
	color_eyre::install()?;
	env_logger::init();
	let options = cli_action().run();
	match options {
		CliAction::Codegen { exclude, path, inline_html, extended_form_controls, external_custom_element } => {
			recursive_template_search(
				path,
				&{
					if exclude.is_empty() {
						vec!["node_modules".into()]
					}else{
						exclude
					}
				}.into_iter().collect(),
				&mut |file_path, base_name_hint| {
					do_code_gen(file_path, base_name_hint, inline_html, extended_form_controls, &external_custom_element)
				}
			)?;
		},
		CliAction::BundleSinglePageApp {
			output_file,
			include,
			exclude,
			input_fragments
		} => {
			do_bundle_spa(
				output_file,
				IncludeElementChecker::from_string_vecs(include, exclude),
				input_fragments
			)?;
		}
	}
	Ok(())
}
