use std::{fs, path::{Path, PathBuf}, io, collections::{HashMap, HashSet}, sync::Arc, ffi::OsString};
use acetewm::selector;
use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use log::{debug, warn, error};
use scraper::{Html, Node as HtmlNode, Selector, ElementRef, node::Text};
use ego_tree::{Tree, NodeId};

use crate::{consts::{ATTRIBUTE_ACE_NAME, VALID_CUSTOM_ELEMENT_NAME, INVALID_CUSTOM_ELEMENT_NAME, ATTRIBUTE_ID, ATTRIBUTE_INLINE}, workarounds::{html_node_editable::EditableHtmlNode, ego_tree_addons::NodeMutAddons}};

use super::recursive_template_search;

#[derive(Debug)]
pub(crate) struct TemplateBundle {
	frontend_templates: String, 
	backend_templates: HashMap<Arc<str>, Arc<str>>
}
impl TemplateBundle {
	pub fn new() -> Result<Self> {
		Ok(
			TemplateBundle {
				frontend_templates: String::new(),
				backend_templates: HashMap::new()
			}
		)
	}
	pub fn add_to_bundle(&mut self, file_path: &Path) -> Result<()> {

		debug!("add_to_bundle: process file: {}", file_path.to_string_lossy());
		let template_markup = Html::parse_fragment(
			&String::from_utf8_lossy(&fs::read(&file_path)?)
		);
		if template_markup.errors.len() != 0 {
			warn!("Parse error(s) were detected while reading the following file: {}", file_path.to_string_lossy());
			warn!("Parse error(s) are as follows:");
		}
		for error_msg in template_markup.errors.iter() {
			warn!("    {}", error_msg);
		}
		let template_markup_root_elem = template_markup.root_element();
		
		
		Ok(())
	}
}

pub(crate) fn traverse_node_modules<P: AsRef<Path>>(path_dir: P) -> Result<()> {
	let mut path_dir = fs::canonicalize(path_dir)?;
	while path_dir.parent().is_some() {
		path_dir.push("node_modules");
		let modules_folder_meta = match fs::metadata(&path_dir) {
			Ok(meta) => meta,
			Err(e) if e.kind() == io::ErrorKind::NotFound => {
				path_dir.pop();
				path_dir.pop();
				continue;
			}
			Err(e) => return Err(e.into()),
		};
		if modules_folder_meta.is_dir() {
			// recursive_bundle_search(path_dir.clone())?;
		}
		path_dir.pop();
		path_dir.pop();
	}
	Ok(())
}

pub(crate) struct IncludeElementChecker {
	include: HashSet<Arc<str>>,
	exclude: HashSet<Arc<str>>
}
impl IncludeElementChecker {
	pub fn from_string_vecs(include: Vec<String>, exclude: Vec<String>) -> Self {
		IncludeElementChecker {
			include: include.into_iter().map(|s| {s.into()}).collect(),
			exclude: exclude.into_iter().map(|s| {s.into()}).collect()
		}
	}
	pub fn should_include(&self, elem_tag: &str) -> bool {
		if self.exclude.contains(elem_tag) {
			return false;
		}
		return self.include.len() == 0 || self.include.contains(elem_tag);
	}
}

lazy_static! {
	pub static ref NO_MAIN_TEMPLATE: HashSet<OsString> = {
		let mut m = HashSet::new();
		m.insert("main_template.html".into());
		m
	};
}
/* 
fn create_editable_html_tree(tree_source: Html) -> Tree<HtmlNode> {
	tree_source.tree.
}
*/

pub(crate) fn do_bundle_spa<P: AsRef<Path>>(
	output_file: P,
	include: IncludeElementChecker,
	input_dir: P
) -> Result<()> {
	
	let mut file_path = input_dir.as_ref().to_path_buf();
	file_path.push("main_template.html");
	debug!("do_bundle_spa: process file: {}", file_path.to_string_lossy());
	let mut main_template_markup = Html::parse_document(
		&String::from_utf8_lossy(&fs::read(&file_path)?)
	);
	file_path.pop();


	//let custom_elements = 

	// TODO: Read closest found package.json and read dependencies for templates
	recursive_template_search(file_path, &NO_MAIN_TEMPLATE, &mut |file_path, _| {
		debug!("do_bundle_spa: process file: {}", file_path.to_string_lossy());
		let mut template_markup = Html::parse_fragment(
			&String::from_utf8_lossy(&fs::read(&file_path)?)
		);
		if template_markup.errors.len() != 0 {
			warn!("Parse error(s) were detected while reading the following file: {}", file_path.to_string_lossy());
			warn!("Parse error(s) are as follows:")
		}
		for error_msg in template_markup.errors.iter() {
			warn!("    {}", error_msg);
		}

		let template_markup_root_elem = template_markup.root_element();

		// We gotta collect node references to all the elements we want to edit first due to rust's mutability rules.
		let template_tree_nodes: Vec<NodeId> = template_markup_root_elem
			.children()
			.filter_map(|node_ref| {
				let HtmlNode::Element(elem) = node_ref.value() else {
					return None;
				};
				if elem.name() != "template" {
					return None;
				}
				let Some(template_elem_tag) = elem.attrs.get(&ATTRIBUTE_ACE_NAME) else {
					warn!("Skipping template without \"ace-name\" attribute");
					return None;
				};
				if
					!VALID_CUSTOM_ELEMENT_NAME.is_match(&template_elem_tag) ||
					INVALID_CUSTOM_ELEMENT_NAME.is_match(&template_elem_tag)
				{
					error!("\"{}\" is not a valid custom element name", template_elem_tag);
					return None;
				}
				if !include.should_include(&template_elem_tag) {
					return None;
				}
				Some(node_ref.id())
			})
			.collect();
		
		for node_id in template_tree_nodes.iter() {
			let Some(mut node_ref) = template_markup.tree.get_mut(*node_id) else {
				continue;
			};
			let elem = node_ref.value().as_element_mut().unwrap();
			let template_elem_tag = elem.attrs.get(&ATTRIBUTE_ACE_NAME).unwrap().clone();
			let template_template_id = format!("ace-template-{}", template_elem_tag);
			elem.attrs.remove(&ATTRIBUTE_ACE_NAME);
			elem.attrs.insert(ATTRIBUTE_ID.clone(), template_template_id.clone().into());
			let is_inline = elem.attrs.contains_key(&ATTRIBUTE_INLINE);
			let node_ref = template_markup.tree.get(*node_id).unwrap(); // invalidates elem
			
			if is_inline {
				// Working around Rust's mutability rules actually saves us here from infinite recursion!
				let nodes_to_replace: Vec<NodeId> = main_template_markup
					.select(selector!(&template_elem_tag))
					.map(|elem_ref| {elem_ref.id()})
					.collect();

				for main_t_node_id in nodes_to_replace.iter() {
					// This replaces main_t_node with the _children_ of node_ref, i.e. the template.
					// the template elements itself doesn't get copied, only its contents.
					let mut main_t_node_ref = main_template_markup.tree.get_mut(*main_t_node_id).unwrap();
					main_t_node_ref.insert_cloned_descendants_before(&node_ref);
					main_t_node_ref.detach();
				}
			} else {
				
				// Append the template element and all its children to the end of the <body>
				let mut main_template_body_node = main_template_markup.tree.get_mut(
					main_template_markup.select(selector!("body"))
					.next()
					.expect("main_template.html should have a <body>")
					.id()
				).unwrap();
				main_template_body_node.append_cloned_tree(&node_ref);
				main_template_body_node.append(HtmlNode::Text(Text { text: "\n".into()}));
				// TODO: apply the correct whitespace so the output looks pretty
			}
		}
		Ok(())
	})?;
	fs::write(output_file, main_template_markup.html())?;
	Ok(())
}
