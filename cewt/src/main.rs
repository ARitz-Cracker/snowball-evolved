use std::{path::PathBuf, ffi::OsString};
use bpaf::Bpaf;
use cli::{recursive_template_search, bundle::{do_bundle_spa, IncludeElementChecker}};
use color_eyre::eyre::Result;

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
		#[bpaf(argument("NAME"), short, long)]
		exclude: Vec<OsString>,
		/// Include HTML snippet in TypeScript output instead of assuming the template exists in the DOM
		#[bpaf(short('I'), long)]
		inline_html: bool,
		/// Have generated code contain helpers and "known" properties for HTMLFormElements
		#[bpaf(short('F'), long)]
		extended_form_controls: bool,
		/// Folder to scan for HTML template fragments and generate accompanying code.
		#[bpaf(positional("PATH"))]
		path: PathBuf
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

fn main() -> Result<()> {
	color_eyre::install()?;
	env_logger::init();
	let options = cli_action().run();
	match options {
		CliAction::Codegen { exclude, path, inline_html, extended_form_controls } => {
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
					do_code_gen(file_path, base_name_hint, inline_html, extended_form_controls)
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
