use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::rc::Rc;
use crate::consts::{ATTRIBUTE_INLINE, ATTRIBUTE_NAME, ATTRIBUTE_ACE_REF, HTML_TAG_TO_TYPE, ATTRIBUTE_ACE_NAME, VALID_CUSTOM_ELEMENT_NAME, INVALID_CUSTOM_ELEMENT_NAME, ATTRIBUTE_TYPE, ATTRIBUTE_VALUE, ATTRIBUTE_ACE_EXTENDS, ATTRIBUTE_ACE_ATTRIBUTES};
use acetewm::selector;
use color_eyre::eyre::Result;

use convert_case::{Casing, Case};
use html5ever::tendril::fmt::Slice;
use lazy_regex::{lazy_regex, Captures};
use log::{warn, debug, info, error};
use scraper::{Html, Node as HtmlNode, ElementRef};
use std::fs;
use std::io::Write;

// There are symbols which are valid custom-element names but aren't valid in JS variables, emojis for example.
// Though I am not properly handling unicode characters outside the BMP.
// This is cuz I've partially copied this regex from https://mothereff.in/js-variables, and due to JS and
// it's UTF16 shenanigans, they have a bunch of OR's matching with the surrogate pairs, which is a no-no in
// Rust, (since it handles characters outside the BMP properly) so I've removed said matches for now.
static INVALID_JS_VAR_REGEX: lazy_regex::Lazy<lazy_regex::Regex> = lazy_regex!(r#"[^\$0-9A-Z_a-z\xAA\xB5\xB7\xBA\xC0-\xD6\xD8-\xF6\xF8-\u{2C1}\u{2C6}-\u{2D1}\u{2E0}-\u{2E4}\u{2EC}\u{2EE}\u{300}-\u{374}\u{376}\u{377}\u{37A}-\u{37D}\u{37F}\u{386}-\u{38A}\u{38C}\u{38E}-\u{3A1}\u{3A3}-\u{3F5}\u{3F7}-\u{481}\u{483}-\u{487}\u{48A}-\u{52F}\u{531}-\u{556}\u{559}\u{561}-\u{587}\u{591}-\u{5BD}\u{5BF}\u{5C1}\u{5C2}\u{5C4}\u{5C5}\u{5C7}\u{5D0}-\u{5EA}\u{5F0}-\u{5F2}\u{610}-\u{61A}\u{620}-\u{669}\u{66E}-\u{6D3}\u{6D5}-\u{6DC}\u{6DF}-\u{6E8}\u{6EA}-\u{6FC}\u{6FF}\u{710}-\u{74A}\u{74D}-\u{7B1}\u{7C0}-\u{7F5}\u{7FA}\u{800}-\u{82D}\u{840}-\u{85B}\u{8A0}-\u{8B4}\u{8E3}-\u{963}\u{966}-\u{96F}\u{971}-\u{983}\u{985}-\u{98C}\u{98F}\u{990}\u{993}-\u{9A8}\u{9AA}-\u{9B0}\u{9B2}\u{9B6}-\u{9B9}\u{9BC}-\u{9C4}\u{9C7}\u{9C8}\u{9CB}-\u{9CE}\u{9D7}\u{9DC}\u{9DD}\u{9DF}-\u{9E3}\u{9E6}-\u{9F1}\u{A01}-\u{A03}\u{A05}-\u{A0A}\u{A0F}\u{A10}\u{A13}-\u{A28}\u{A2A}-\u{A30}\u{A32}\u{A33}\u{A35}\u{A36}\u{A38}\u{A39}\u{A3C}\u{A3E}-\u{A42}\u{A47}\u{A48}\u{A4B}-\u{A4D}\u{A51}\u{A59}-\u{A5C}\u{A5E}\u{A66}-\u{A75}\u{A81}-\u{A83}\u{A85}-\u{A8D}\u{A8F}-\u{A91}\u{A93}-\u{AA8}\u{AAA}-\u{AB0}\u{AB2}\u{AB3}\u{AB5}-\u{AB9}\u{ABC}-\u{AC5}\u{AC7}-\u{AC9}\u{ACB}-\u{ACD}\u{AD0}\u{AE0}-\u{AE3}\u{AE6}-\u{AEF}\u{AF9}\u{B01}-\u{B03}\u{B05}-\u{B0C}\u{B0F}\u{B10}\u{B13}-\u{B28}\u{B2A}-\u{B30}\u{B32}\u{B33}\u{B35}-\u{B39}\u{B3C}-\u{B44}\u{B47}\u{B48}\u{B4B}-\u{B4D}\u{B56}\u{B57}\u{B5C}\u{B5D}\u{B5F}-\u{B63}\u{B66}-\u{B6F}\u{B71}\u{B82}\u{B83}\u{B85}-\u{B8A}\u{B8E}-\u{B90}\u{B92}-\u{B95}\u{B99}\u{B9A}\u{B9C}\u{B9E}\u{B9F}\u{BA3}\u{BA4}\u{BA8}-\u{BAA}\u{BAE}-\u{BB9}\u{BBE}-\u{BC2}\u{BC6}-\u{BC8}\u{BCA}-\u{BCD}\u{BD0}\u{BD7}\u{BE6}-\u{BEF}\u{C00}-\u{C03}\u{C05}-\u{C0C}\u{C0E}-\u{C10}\u{C12}-\u{C28}\u{C2A}-\u{C39}\u{C3D}-\u{C44}\u{C46}-\u{C48}\u{C4A}-\u{C4D}\u{C55}\u{C56}\u{C58}-\u{C5A}\u{C60}-\u{C63}\u{C66}-\u{C6F}\u{C81}-\u{C83}\u{C85}-\u{C8C}\u{C8E}-\u{C90}\u{C92}-\u{CA8}\u{CAA}-\u{CB3}\u{CB5}-\u{CB9}\u{CBC}-\u{CC4}\u{CC6}-\u{CC8}\u{CCA}-\u{CCD}\u{CD5}\u{CD6}\u{CDE}\u{CE0}-\u{CE3}\u{CE6}-\u{CEF}\u{CF1}\u{CF2}\u{D01}-\u{D03}\u{D05}-\u{D0C}\u{D0E}-\u{D10}\u{D12}-\u{D3A}\u{D3D}-\u{D44}\u{D46}-\u{D48}\u{D4A}-\u{D4E}\u{D57}\u{D5F}-\u{D63}\u{D66}-\u{D6F}\u{D7A}-\u{D7F}\u{D82}\u{D83}\u{D85}-\u{D96}\u{D9A}-\u{DB1}\u{DB3}-\u{DBB}\u{DBD}\u{DC0}-\u{DC6}\u{DCA}\u{DCF}-\u{DD4}\u{DD6}\u{DD8}-\u{DDF}\u{DE6}-\u{DEF}\u{DF2}\u{DF3}\u{E01}-\u{E3A}\u{E40}-\u{E4E}\u{E50}-\u{E59}\u{E81}\u{E82}\u{E84}\u{E87}\u{E88}\u{E8A}\u{E8D}\u{E94}-\u{E97}\u{E99}-\u{E9F}\u{EA1}-\u{EA3}\u{EA5}\u{EA7}\u{EAA}\u{EAB}\u{EAD}-\u{EB9}\u{EBB}-\u{EBD}\u{EC0}-\u{EC4}\u{EC6}\u{EC8}-\u{ECD}\u{ED0}-\u{ED9}\u{EDC}-\u{EDF}\u{F00}\u{F18}\u{F19}\u{F20}-\u{F29}\u{F35}\u{F37}\u{F39}\u{F3E}-\u{F47}\u{F49}-\u{F6C}\u{F71}-\u{F84}\u{F86}-\u{F97}\u{F99}-\u{FBC}\u{FC6}\u{1000}-\u{1049}\u{1050}-\u{109D}\u{10A0}-\u{10C5}\u{10C7}\u{10CD}\u{10D0}-\u{10FA}\u{10FC}-\u{1248}\u{124A}-\u{124D}\u{1250}-\u{1256}\u{1258}\u{125A}-\u{125D}\u{1260}-\u{1288}\u{128A}-\u{128D}\u{1290}-\u{12B0}\u{12B2}-\u{12B5}\u{12B8}-\u{12BE}\u{12C0}\u{12C2}-\u{12C5}\u{12C8}-\u{12D6}\u{12D8}-\u{1310}\u{1312}-\u{1315}\u{1318}-\u{135A}\u{135D}-\u{135F}\u{1369}-\u{1371}\u{1380}-\u{138F}\u{13A0}-\u{13F5}\u{13F8}-\u{13FD}\u{1401}-\u{166C}\u{166F}-\u{167F}\u{1681}-\u{169A}\u{16A0}-\u{16EA}\u{16EE}-\u{16F8}\u{1700}-\u{170C}\u{170E}-\u{1714}\u{1720}-\u{1734}\u{1740}-\u{1753}\u{1760}-\u{176C}\u{176E}-\u{1770}\u{1772}\u{1773}\u{1780}-\u{17D3}\u{17D7}\u{17DC}\u{17DD}\u{17E0}-\u{17E9}\u{180B}-\u{180D}\u{1810}-\u{1819}\u{1820}-\u{1877}\u{1880}-\u{18AA}\u{18B0}-\u{18F5}\u{1900}-\u{191E}\u{1920}-\u{192B}\u{1930}-\u{193B}\u{1946}-\u{196D}\u{1970}-\u{1974}\u{1980}-\u{19AB}\u{19B0}-\u{19C9}\u{19D0}-\u{19DA}\u{1A00}-\u{1A1B}\u{1A20}-\u{1A5E}\u{1A60}-\u{1A7C}\u{1A7F}-\u{1A89}\u{1A90}-\u{1A99}\u{1AA7}\u{1AB0}-\u{1ABD}\u{1B00}-\u{1B4B}\u{1B50}-\u{1B59}\u{1B6B}-\u{1B73}\u{1B80}-\u{1BF3}\u{1C00}-\u{1C37}\u{1C40}-\u{1C49}\u{1C4D}-\u{1C7D}\u{1CD0}-\u{1CD2}\u{1CD4}-\u{1CF6}\u{1CF8}\u{1CF9}\u{1D00}-\u{1DF5}\u{1DFC}-\u{1F15}\u{1F18}-\u{1F1D}\u{1F20}-\u{1F45}\u{1F48}-\u{1F4D}\u{1F50}-\u{1F57}\u{1F59}\u{1F5B}\u{1F5D}\u{1F5F}-\u{1F7D}\u{1F80}-\u{1FB4}\u{1FB6}-\u{1FBC}\u{1FBE}\u{1FC2}-\u{1FC4}\u{1FC6}-\u{1FCC}\u{1FD0}-\u{1FD3}\u{1FD6}-\u{1FDB}\u{1FE0}-\u{1FEC}\u{1FF2}-\u{1FF4}\u{1FF6}-\u{1FFC}\u{200C}\u{200D}\u{203F}\u{2040}\u{2054}\u{2071}\u{207F}\u{2090}-\u{209C}\u{20D0}-\u{20DC}\u{20E1}\u{20E5}-\u{20F0}\u{2102}\u{2107}\u{210A}-\u{2113}\u{2115}\u{2118}-\u{211D}\u{2124}\u{2126}\u{2128}\u{212A}-\u{2139}\u{213C}-\u{213F}\u{2145}-\u{2149}\u{214E}\u{2160}-\u{2188}\u{2C00}-\u{2C2E}\u{2C30}-\u{2C5E}\u{2C60}-\u{2CE4}\u{2CEB}-\u{2CF3}\u{2D00}-\u{2D25}\u{2D27}\u{2D2D}\u{2D30}-\u{2D67}\u{2D6F}\u{2D7F}-\u{2D96}\u{2DA0}-\u{2DA6}\u{2DA8}-\u{2DAE}\u{2DB0}-\u{2DB6}\u{2DB8}-\u{2DBE}\u{2DC0}-\u{2DC6}\u{2DC8}-\u{2DCE}\u{2DD0}-\u{2DD6}\u{2DD8}-\u{2DDE}\u{2DE0}-\u{2DFF}\u{3005}-\u{3007}\u{3021}-\u{302F}\u{3031}-\u{3035}\u{3038}-\u{303C}\u{3041}-\u{3096}\u{3099}-\u{309F}\u{30A1}-\u{30FA}\u{30FC}-\u{30FF}\u{3105}-\u{312D}\u{3131}-\u{318E}\u{31A0}-\u{31BA}\u{31F0}-\u{31FF}\u{3400}-\u{4DB5}\u{4E00}-\u{9FD5}\u{A000}-\u{A48C}\u{A4D0}-\u{A4FD}\u{A500}-\u{A60C}\u{A610}-\u{A62B}\u{A640}-\u{A66F}\u{A674}-\u{A67D}\u{A67F}-\u{A6F1}\u{A717}-\u{A71F}\u{A722}-\u{A788}\u{A78B}-\u{A7AD}\u{A7B0}-\u{A7B7}\u{A7F7}-\u{A827}\u{A840}-\u{A873}\u{A880}-\u{A8C4}\u{A8D0}-\u{A8D9}\u{A8E0}-\u{A8F7}\u{A8FB}\u{A8FD}\u{A900}-\u{A92D}\u{A930}-\u{A953}\u{A960}-\u{A97C}\u{A980}-\u{A9C0}\u{A9CF}-\u{A9D9}\u{A9E0}-\u{A9FE}\u{AA00}-\u{AA36}\u{AA40}-\u{AA4D}\u{AA50}-\u{AA59}\u{AA60}-\u{AA76}\u{AA7A}-\u{AAC2}\u{AADB}-\u{AADD}\u{AAE0}-\u{AAEF}\u{AAF2}-\u{AAF6}\u{AB01}-\u{AB06}\u{AB09}-\u{AB0E}\u{AB11}-\u{AB16}\u{AB20}-\u{AB26}\u{AB28}-\u{AB2E}\u{AB30}-\u{AB5A}\u{AB5C}-\u{AB65}\u{AB70}-\u{ABEA}\u{ABEC}\u{ABED}\u{ABF0}-\u{ABF9}\u{AC00}-\u{D7A3}\u{D7B0}-\u{D7C6}\u{D7CB}-\u{D7FB}\u{F900}-\u{FA6D}\u{FA70}-\u{FAD9}\u{FB00}-\u{FB06}\u{FB13}-\u{FB17}\u{FB1D}-\u{FB28}\u{FB2A}-\u{FB36}\u{FB38}-\u{FB3C}\u{FB3E}\u{FB40}\u{FB41}\u{FB43}\u{FB44}\u{FB46}-\u{FBB1}\u{FBD3}-\u{FD3D}\u{FD50}-\u{FD8F}\u{FD92}-\u{FDC7}\u{FDF0}-\u{FDFB}\u{FE00}-\u{FE0F}\u{FE20}-\u{FE2F}\u{FE33}\u{FE34}\u{FE4D}-\u{FE4F}\u{FE70}-\u{FE74}\u{FE76}-\u{FEFC}\u{FF10}-\u{FF19}\u{FF21}-\u{FF3A}\u{FF3F}\u{FF41}-\u{FF5A}\u{FF66}-\u{FFBE}\u{FFC2}-\u{FFC7}\u{FFCA}-\u{FFCF}\u{FFD2}-\u{FFD7}\u{FFDA}-\u{FFDC}]"#);

const NORMALIZE_FORM_VALUES_TS: &str = r#"
// TODO: Make this part of a util lib instead of part of the autogen
export function normalizeFormValues(source: HTMLFormElement | SubmitEvent): any {
	const result: any = {};
	const [formElement, submitter] = (() => {
		if (source instanceof HTMLFormElement) {
			return [source, null];
		}
		return [source.target as HTMLFormElement, source.submitter];
	})();
	for (let i = 0; i < formElement.elements.length; i += 1) {
		const formControl = formElement.elements[i];
		if (formControl instanceof HTMLButtonElement) {
			if (formControl == submitter) {
				if (formControl.name) {
					result[formControl.name] = formControl.value;
				}
			}
		}else if (formControl instanceof HTMLInputElement) {
			switch(formControl.type) {
				case "checkbox": {
					result[formControl.name] = formControl.checked;
					break;
				}
				case "datetime-local": {
					result[formControl.name] = formControl.valueAsDate;
					break;
				}
				case "file": {
					result[formControl.name] = formControl.files;
					break;
				}
				case "number":
				case "range": {
					result[formControl.name] = formControl.valueAsNumber;
					break;
				}
				case "radio": {
					if (formControl.checked) {
						result[formControl.name] = formControl.value;
						break;
					}
				}
				default:
					result[formControl.name] = formControl.value;
			}
		}else if (
			formControl instanceof HTMLOutputElement ||
			formControl instanceof HTMLSelectElement ||
			formControl instanceof HTMLTextAreaElement
		) {
			result[formControl.name] = formControl.value;
		}
	}
	return result;
}
"#;


pub(crate) fn form_collection_code_gen<W: Write>(class_name: &str, form_elem: ElementRef, nonce: &mut u64, output: &mut W) -> Result<()> {
	*nonce += 1;
	let mut seen_names: HashSet<Rc<str>> = HashSet::new();
	// query selector based on data from https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/elements

	writeln!(output, "export type {}FormCollection{} = HTMLFormControlsCollection & {{", class_name, nonce)?;
	for form_control_ref in form_elem.select(selector!(
		"button[name],\
		fieldset[name],\
		input[name]:not([type=\"image\"]),\
		object[name],\
		output[name],\
		select[name],\
		textarea[name]"
	)) {
		// Unwraps are used here cuz the selector should make sure that they're always valid.
		let form_control_elem = form_control_ref.value();
		let form_control_name = form_control_elem.attrs.get(&ATTRIBUTE_NAME).unwrap() as &str;
		if seen_names.contains(form_control_name) {
			continue;
		}
		let form_control_class = if
			form_control_elem.name() == "input" &&
			form_control_elem.attrs.get(&ATTRIBUTE_TYPE).is_some_and(|val| {val as &str == "radio"})
		{
			"RadioNodeList"
		} else {
			HTML_TAG_TO_TYPE.get(form_control_elem.name()).unwrap()
		};

		// JS handles unicode differently. Too bad!
		let escaped_control_name = form_control_name.escape_default();

		writeln!(output, "\t\"{}\": {};", escaped_control_name, form_control_class)?;
		writeln!(output, "\tnamedItem(name: \"{}\"): {};", escaped_control_name, form_control_class)?;
		seen_names.insert(form_control_name.into());
	}
	writeln!(output, "}};")?;

	seen_names.clear();
	let mut radio_buttons: HashMap<Rc<str>, HashSet<Rc<str>>> = HashMap::new();
	let mut submit_buttons: HashMap<Rc<str>, HashSet<Rc<str>>> = HashMap::new();
	writeln!(output, "export type {}FormValues{} = {{", class_name, nonce)?;
	for form_control_ref in form_elem.select(selector!(
		"button[name],\
		input[name]:not([type=\"image\"]),\
		output[name],\
		select[name],\
		textarea[name]"
	)) {
		// Unwraps are used here cuz the selector should make sure that they're always valid.
		let form_control_elem = form_control_ref.value();
		let form_control_name = form_control_elem.attrs.get(&ATTRIBUTE_NAME).unwrap() as &str;
		// JS handles unicode differently. Too bad!
		let escaped_control_name = form_control_name.escape_default().to_string();
		match form_control_elem.name() {
			"button" => {
				if !submit_buttons.contains_key(escaped_control_name.as_str()) {
					submit_buttons.insert(escaped_control_name.as_str().into(), HashSet::new());
				}
				if let Some(form_control_value) = form_control_elem.attrs.get(&ATTRIBUTE_VALUE) {
					submit_buttons.get_mut(escaped_control_name.as_str()).unwrap()
						.insert((form_control_value as &str).into());
				}
			},
			"input" => {
				match form_control_elem.attrs.get(&ATTRIBUTE_TYPE).map_or("", |val| {val as &str}) {
					"checkbox" => {
						// Should we support indeterminate or array of const strings?
						writeln!(output, "\t\"{}\": boolean;", escaped_control_name)?;
					},
					"datetime-local" => {
						writeln!(output, "\t\"{}\": Date;", escaped_control_name)?;
					}
					"file" => {
						writeln!(output, "\t\"{}\": FileList | null;", escaped_control_name)?;
					}
					"number" => {
						writeln!(output, "\t\"{}\": number;", escaped_control_name)?;
					}
					"radio" => {
						if !radio_buttons.contains_key(escaped_control_name.as_str()) {
							radio_buttons.insert(escaped_control_name.as_str().into(), HashSet::new());
						}
						if let Some(form_control_value) = form_control_elem.attrs.get(&ATTRIBUTE_VALUE) {
							radio_buttons.get_mut(escaped_control_name.as_str()).unwrap()
								.insert((form_control_value as &str).into());
						}
					}
					"range" => {
						writeln!(output, "\t\"{}\": number;", escaped_control_name)?;
					}
					_ => {
						writeln!(output, "\t\"{}\": string;", escaped_control_name)?;
					}
				}
			},
			"output" => {
				writeln!(output, "\t\"{}\": string;", escaped_control_name)?;
			},
			"select" => {
				if form_control_ref.has_children() {
					write!(output, "\t\"{}\": \"\"", escaped_control_name)?;
					for select_option_ref in form_control_ref.select(selector!("option")) {
						if let Some(form_control_value) = select_option_ref.value().attrs.get(&ATTRIBUTE_VALUE) {
							write!(output, " | \"{}\"", form_control_value)?;
						}
						writeln!(output, ";")?;
					}
				}else{
					// Assume the options are client-side generated.
					writeln!(output, "\t\"{}\": string;", escaped_control_name)?;
				}
			},
			"textarea" => {
				writeln!(output, "\t\"{}\": string;", escaped_control_name)?;
			},
			_ => unreachable!("query selector should work")
		}
	}
	for (name, values) in radio_buttons.iter() {
		write!(output, "\t\"{}\": \"\"", name)?;
		for value in values.iter() {
			write!(output, " | \"{}\"", value)?;
		}
		writeln!(output, ";")?;
	}
	for (name, values) in submit_buttons.iter() {
		if values.len() == 0 {
			continue;
		}
		write!(output, "\t\"{}\"?: \"\"", name)?;
		for value in values.iter() {
			write!(output, " | \"{}\"", value)?;
		}
		writeln!(output, ";")?;
	}
	writeln!(output, "}};")?;
	Ok(())
}

pub(crate) fn do_code_gen(file_path: &Path, base_name_hint: Option<&str>) -> Result<()> {
	debug!("do_code_gen: process file: {}", file_path.to_string_lossy());
	let template_markup = Html::parse_fragment(
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
	// if there are no elements which require autogen, return early
	if !template_markup_root_elem.children().any(|node| {
		match node.value() {
			HtmlNode::Element(elem) => {
				elem.name() == "template" &&
				elem.attrs.contains_key(&ATTRIBUTE_ACE_NAME) &&
				!elem.attrs.contains_key(&ATTRIBUTE_INLINE)
			}
			_ => false
		}
	}) {
		warn!("No templates here!");
		return Ok(());
	}

	let mut file_path = file_path.to_path_buf();
	file_path.pop();
	match base_name_hint {
		Some(template_name) => {
			file_path.push("_autogen");
			fs::create_dir_all(&file_path)?;
			file_path.push(format!("{}.ts", template_name));
		},
		None => file_path.push("_autogen.ts"),
	}
	info!("Create file {}", file_path.to_string_lossy());

	let mut form_collection_nonce = 0u64;
	let mut form_collections_buf = Vec::new();
	let mut file_handle = fs::File::create(&file_path)?;
	writeln!(file_handle, "// auto-generated by acetewm")?;
	writeln!(file_handle, "// DO NOT EDIT BY HAND!!")?;
	for node_ref in template_markup_root_elem.children() {
		let HtmlNode::Element(elem) = node_ref.value() else {
			continue;
		};
		if
			elem.name() != "template" ||
			elem.attrs.contains_key(&ATTRIBUTE_INLINE)
		{
			continue;
		}
		let Some(template_elem_tag) = elem.attrs.get(&ATTRIBUTE_ACE_NAME) else {
			warn!("Skipping template without \"ace-name\" attribute");
			continue;
		};
		debug!("Found client template: {}", template_elem_tag);
		if
			!VALID_CUSTOM_ELEMENT_NAME.is_match(&template_elem_tag) ||
			INVALID_CUSTOM_ELEMENT_NAME.is_match(&template_elem_tag)
		{
			error!("\"{}\" is not a valid custom element name", template_elem_tag);
			continue;
		}
		let template_template_id = format!("ace-template-{}", template_elem_tag);
		let template_class_name = template_elem_tag.as_ref().to_case(Case::Pascal);
		let template_class_name = INVALID_JS_VAR_REGEX.replace_all(
			&template_class_name,
			|invalid_char: &Captures | {
				format!("U{:x}", invalid_char.get(0).unwrap().as_str().chars().nth(0).unwrap_or('\0') as u32)
			}
		);

		let template_extends_tag = elem.attrs.get(&ATTRIBUTE_ACE_EXTENDS);
		let template_extends_class = template_extends_tag.and_then(|v| {
			HTML_TAG_TO_TYPE.get(v as &str)
		}).unwrap_or(&"HTMLElement");

		// TODO: Enforce the rules mentioned here https://stackoverflow.com/a/25033330
		let template_observed_attributes = elem.attrs.get(&ATTRIBUTE_ACE_ATTRIBUTES)
			.map(|attribute_str| {
				let mut attributes = HashSet::new();
				for attribute in (attribute_str as &str).split(',') {
					attributes.insert(attribute.trim());
				}

				return attributes;
			})
			.unwrap_or_default();
		
		if template_extends_tag.is_none() {
			// Write slots
			writeln!(file_handle, "export class {}Slots {{", template_class_name)?;
			writeln!(file_handle, "\tprivate _element: HTMLElement;")?;
			writeln!(file_handle, "\tconstructor(element: HTMLElement) {{")?;
			writeln!(file_handle, "\t\tthis._element = element;")?;
			writeln!(file_handle, "\t}}")?;
			for child_node_ref in node_ref.descendants() {
				let HtmlNode::Element(child_elem) = child_node_ref.value() else {
					continue;
				};
				if child_elem.name() != "slot" {
					continue;
				}
				let Some(slot_raw_name) = child_elem.attrs.get(&ATTRIBUTE_NAME) else {
					warn!("template \"{}\" has a nameless slot!", template_elem_tag);
					continue;
				};
				debug!("... with slot: {}", slot_raw_name);
				let slot_element_type = child_node_ref
					.children()
					.find_map(|v| v.value().as_element())
					.map(|elem|{elem.name()})
					.unwrap_or("span");
				let slot_property_name = slot_raw_name.as_ref().to_case(Case::Camel);
				writeln!(
					file_handle,
					"\tprivate _{}?: {};",
					slot_property_name,
					HTML_TAG_TO_TYPE.get(slot_element_type).unwrap_or(&"HTMLElement")
				)?;
				writeln!(file_handle, "\tget {}() {{", slot_property_name)?;
				writeln!(file_handle, "\t\tif (this._{} === undefined) {{", slot_property_name)?;
				writeln!(
					file_handle,
					"\t\t\tthis._{} = this._element.querySelector(\"[slot=\\\"{}\\\"]\") ?? \
						document.createElement(\"{}\");",
					slot_property_name,
					slot_raw_name,
					slot_element_type
					
				)?;
				writeln!(file_handle, "\t\t\tthis._{}.slot = \"{}\";", slot_property_name, slot_raw_name)?;
				writeln!(file_handle, "\t\t\tthis._element.appendChild(this._{});", slot_property_name)?;
				writeln!(file_handle, "\t\t}}")?;
				writeln!(file_handle, "\t\treturn this._{};", slot_property_name)?;
				writeln!(file_handle, "\t}}")?;
			}
			writeln!(file_handle, "}}")?;
		}

		// Write refs
		writeln!(file_handle, "export class {}Refs {{", template_class_name)?;
		writeln!(file_handle, "\tprivate _element: HTMLElement;")?;
		writeln!(file_handle, "\tconstructor(element: HTMLElement) {{")?;
		writeln!(file_handle, "\t\tthis._element = element;")?;
		writeln!(file_handle, "\t}}")?;
		for child_node_ref in node_ref.descendants() {
			let HtmlNode::Element(child_elem) = child_node_ref.value() else {
				continue;
			};
			let Some(ref_raw_name) = child_elem.attrs.get(&ATTRIBUTE_ACE_REF) else {
				continue;
			};
			debug!("... with ref: {}", ref_raw_name);
			let ref_property_name = ref_raw_name.as_ref().to_case(Case::Camel);
			writeln!(
				file_handle,
				"\tprivate _{}?: {};",
				ref_property_name,
				if child_elem.name() == "form" {
					form_collection_code_gen(
						&template_class_name,
						ElementRef::wrap(child_node_ref).unwrap(),
						&mut form_collection_nonce,
						&mut form_collections_buf
					)?;
					format!(
						"HTMLFormElementKnownControls<{0}FormCollection{1}, {0}FormValues{1}>",
						template_class_name,
						form_collection_nonce
					)
				}else{
					// I know, useless clone, haven't had much sleep.
					HTML_TAG_TO_TYPE.get(child_elem.name()).unwrap_or(&"HTMLElement").to_string()
				}
			)?;
			writeln!(file_handle, "\tget {}() {{", ref_property_name)?;
			writeln!(file_handle, "\t\tif (this._{} === undefined) {{", ref_property_name)?;
			if template_extends_tag.is_none() {
				writeln!(
					file_handle,
					"\t\t\tthis._{} = this._element.shadowRoot!.querySelector(\":not([is]) [ace-ref=\\\"{}\\\"]\")!;",
					ref_property_name,
					ref_raw_name,
				)?;
			}else{
				writeln!(
					file_handle,
					"\t\t\tthis._{} = this._element.querySelector(\"[ace-ref=\\\"{}\\\"]:not(:not(:scope)[is] *)\")!;",
					ref_property_name,
					ref_raw_name,
				)?;
			}
			
			if child_elem.name() == "form" {
				writeln!(
					file_handle,
					"\t\t\tthis._{0}.values = normalizeFormValues.bind(this._{0}, this._{0});",
					ref_property_name
				)?;
			}
			writeln!(file_handle, "\t\t}}")?;
			writeln!(file_handle, "\t\treturn this._{};", ref_property_name)?;
			writeln!(file_handle, "\t}}")?;
		}
		writeln!(file_handle, "}}")?;

		// Write base autogen
		writeln!(
			file_handle,
			"export class {}Autogen extends {} {{",
			template_class_name,
			template_extends_class
		)?;
		if template_extends_tag.is_none() {
			writeln!(file_handle, "\treadonly slots: {}Slots;", template_class_name)?;
		}
		writeln!(file_handle, "\treadonly refs: {}Refs;", template_class_name)?;
		if !template_observed_attributes.is_empty() {
			writeln!(file_handle, "\tstatic get observedAttributes() {{")?;
			writeln!(
				file_handle,
				"\t\treturn [{}];",
				template_observed_attributes.iter()
					.map(|v| {format!("\"{}\"", v.escape_default())})
					.collect::<Vec<String>>()
					.join(", ")
			)?;
			writeln!(file_handle, "\t}}")?;
			let mut cb_ts = Vec::new();
			writeln!(
				cb_ts,
				"\tattributeChangedCallback(name: string, oldValue: string | null, newValue: string | null) {{"
			)?;
			writeln!(cb_ts, "\t\tswitch(name) {{")?;

			for attrib in template_observed_attributes.iter() {
				let attrib_property = attrib.to_case(Case::Camel);
				let attrib_property = if INVALID_JS_VAR_REGEX.is_match(&attrib_property) {
					format!("[\"{}\"]", attrib.escape_default())
				}else{
					attrib_property.to_string()
				};
				let attrib_callback_name = attrib.to_case(Case::Pascal);
				let attrib_callback_name = INVALID_JS_VAR_REGEX.replace_all(
					&attrib_callback_name,
					|invalid_char: &Captures | {
						format!("U{:x}", invalid_char.get(0).unwrap().as_str().chars().nth(0).unwrap_or('\0') as u32)
					}
				);
				writeln!(file_handle, "\tprivate _attribute{}Value: string | null = null;", attrib_callback_name)?;
				writeln!(file_handle, "\tget {}(): string | null {{", attrib_property)?;
				writeln!(file_handle, "\t\treturn this._attribute{}Value;", attrib_callback_name)?;
				writeln!(file_handle, "\t}}")?;
				writeln!(file_handle, "\tset {}(v: string | null) {{", attrib_property)?;
				writeln!(file_handle, "\t\tif (v == null) {{")?;
				writeln!(file_handle, "\t\t\tthis.removeAttribute(\"{}\");", attrib.escape_default())?;
				writeln!(file_handle, "\t\t}}else{{")?;
				writeln!(file_handle, "\t\t\tthis.setAttribute(\"{}\", v);", attrib.escape_default())?;
				writeln!(file_handle, "\t\t}}")?;
				writeln!(file_handle, "\t}}")?;
				writeln!(
					file_handle,
					"\tprotected on{}Changed(oldValue: string | null, newValue: string | null) {{",
					attrib_callback_name
				)?;
				writeln!(file_handle, "\t\t// To be overridden by child class")?;
				writeln!(file_handle, "\t}}")?;

				writeln!(cb_ts, "\t\t\tcase \"{}\":", attrib.escape_default())?;
				writeln!(cb_ts, "\t\t\t\tthis._attribute{}Value = newValue;", attrib_callback_name)?;
				writeln!(cb_ts, "\t\t\t\tthis.on{}Changed(oldValue, newValue);", attrib_callback_name)?;
				writeln!(cb_ts, "\t\t\t\tbreak;")?;

			}
			writeln!(cb_ts, "\t\t\tdefault:")?;
			writeln!(cb_ts, "\t\t\t\t// Shouldn't happen")?;
			writeln!(cb_ts, "\t\t}}")?;
			writeln!(cb_ts, "\t}}")?;
			file_handle.write(cb_ts.as_bytes())?;
		}

		writeln!(file_handle, "\tconstructor() {{")?;
		writeln!(file_handle, "\t\tsuper();")?;
		if template_extends_tag.is_none() {
			writeln!(file_handle, "\t\tconst shadowRoot = this.attachShadow({{ mode: \"open\" }});")?;
			writeln!(file_handle, "\t\tshadowRoot.appendChild(")?;
			writeln!(file_handle, "\t\t\t(document.getElementById(\"{}\") as HTMLTemplateElement)", template_template_id)?;
			writeln!(file_handle, "\t\t\t\t.content")?;
			writeln!(file_handle, "\t\t\t\t.cloneNode(true)")?;
			writeln!(file_handle, "\t\t);")?;
			writeln!(file_handle, "\t\tthis.slots = new {}Slots(this);", template_class_name)?;
		}else{
			writeln!(file_handle, "\t\tif (this.childElementCount == 0) {{")?;
			writeln!(file_handle, "\t\t\tthis.appendChild(")?;
			writeln!(file_handle, "\t\t\t\t(document.getElementById(\"{}\") as HTMLTemplateElement)", template_template_id)?;
			writeln!(file_handle, "\t\t\t\t\t.content")?;
			writeln!(file_handle, "\t\t\t\t\t.cloneNode(true)")?;
			writeln!(file_handle, "\t\t\t);")?;
			writeln!(file_handle, "\t\t}}")?;
			writeln!(file_handle, "\t\tthis.setAttribute(\"is\", \"{}\"); // allow for easy query selecting", template_elem_tag)?;
		}
		writeln!(file_handle, "\t\tthis.refs = new {}Refs(this);", template_class_name)?;
		writeln!(file_handle, "\t}}")?;

		writeln!(file_handle, "\tpublic static registerElement() {{")?;
		if let Some(base_tag) = template_extends_tag {
			writeln!(file_handle, "\t\tcustomElements.define(\"{}\", this, {{ extends: \"{}\"}});", template_elem_tag, base_tag)?;
		}else{
			writeln!(file_handle, "\t\tcustomElements.define(\"{}\", this);", template_elem_tag)?;
		}
		
		writeln!(file_handle, "\t}}")?;
		writeln!(file_handle, "}}")?;
	}
	debug!("Forms found: {}", form_collection_nonce);
	if form_collection_nonce > 0 {
		file_handle.write(&form_collections_buf)?;
		writeln!(file_handle, "interface HTMLFormElementKnownControls<C extends HTMLFormControlsCollection, V> extends HTMLFormElement {{")?;
		writeln!(file_handle, "\treadonly elements: C;")?;
		writeln!(file_handle, "\tvalues: () => V;")?;
		writeln!(file_handle, "}};")?;

		file_handle.write(NORMALIZE_FORM_VALUES_TS.as_bytes())?;
	}
	Ok(())
}
