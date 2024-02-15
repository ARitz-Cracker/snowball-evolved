use scraper::{Node as HtmlNode, node::{Doctype, Comment, Text, Element, ProcessingInstruction}};
pub trait EditableHtmlNode {
	fn as_doctype_mut(&mut self) -> Option<&mut Doctype>;
	/// Returns self as a comment.
	fn as_comment_mut(&mut self) -> Option<&mut Comment>;
	/// Returns self as text.
	fn as_text_mut(&mut self) -> Option<&mut Text>;
	/// Returns self as an element.
	fn as_element_mut(&mut self) -> Option<&mut Element>;
	/// Returns self as an element.
	fn as_processing_instruction_mut(&mut self) -> Option<&mut ProcessingInstruction>;
}

impl EditableHtmlNode for HtmlNode {
	fn as_doctype_mut(&mut self) -> Option<&mut Doctype> {
		match self {
			HtmlNode::Doctype(c) => Some(c),
			_ => None,
		}
	}
	fn as_comment_mut(&mut self) -> Option<&mut Comment> {
		match self {
			HtmlNode::Comment(c) => Some(c),
			_ => None,
		}
	}
	fn as_text_mut(&mut self) -> Option<&mut Text> {
		match self {
			HtmlNode::Text(t) => Some(t),
			_ => None,
		}
	}
	fn as_element_mut(&mut self) -> Option<&mut Element> {
		match self {
			HtmlNode::Element(e) => Some(e),
			_ => None,
		}
	}
	fn as_processing_instruction_mut(&mut self) -> Option<&mut ProcessingInstruction> {
		match self {
			HtmlNode::ProcessingInstruction(pi) => Some(pi),
			_ => None,
		}
	}
}
