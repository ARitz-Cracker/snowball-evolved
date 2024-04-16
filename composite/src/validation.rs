use lazy_static::lazy_static;
use bitflags::{bitflags, Flags};
use std::{borrow::Cow, collections::HashMap};

/*
"	=>	&quot;
&	=>	&amp;
'	=>	&#039;
<	=>	&lt;
>	=>	&gt;

*/
lazy_static! {
	pub static ref HTML_SPECIAL_CHARS_MAP: HashMap<char, &'static str> = {
		// get_html_translation_table(0, ENT_QUOTES | ENT_SUBSTITUTE); on PHP
		let mut m = HashMap::new();
		m.insert('"', "&quot");
		m.insert('&', "&amp;");
		m.insert('\'', "&apos;"); // HTML5! woo!
		m.insert('<', "&lt");
		m.insert('>', "&gt");
		m
	};
	pub static ref HTML_ENTITIES_MAP: HashMap<char, &'static str> = {
		// get_html_translation_table(1, ENT_QUOTES | ENT_SUBSTITUTE); on PHP
		let mut m = HashMap::new();
		m.insert('"', "&quot");
		m.insert('&', "&amp;");
		m.insert('\'', "&apos;"); // HTML5! woo!
		m.insert('<', "&lt");
		m.insert('>', "&gt");
		m.insert('\u{00a0}', "&nbsp;");
		m.insert('¡', "&iexcl;");
		m.insert('¢', "&cent;");
		m.insert('£', "&pound;");
		m.insert('¤', "&curren;");
		m.insert('¥', "&yen;");
		m.insert('¦', "&brvbar;");
		m.insert('§', "&sect;");
		m.insert('¨', "&uml;");
		m.insert('©', "&copy;");
		m.insert('ª', "&ordf;");
		m.insert('«', "&laquo;");
		m.insert('¬', "&not;");
		m.insert('\u{00ad}', "&shy;");
		m.insert('®', "&reg;");
		m.insert('¯', "&macr;");
		m.insert('°', "&deg;");
		m.insert('±', "&plusmn;");
		m.insert('²', "&sup2;");
		m.insert('³', "&sup3;");
		m.insert('´', "&acute;");
		m.insert('µ', "&micro;");
		m.insert('¶', "&para;");
		m.insert('·', "&middot;");
		m.insert('¸', "&cedil;");
		m.insert('¹', "&sup1;");
		m.insert('º', "&ordm;");
		m.insert('»', "&raquo;");
		m.insert('¼', "&frac14;");
		m.insert('½', "&frac12;");
		m.insert('¾', "&frac34;");
		m.insert('¿', "&iquest;");
		m.insert('À', "&Agrave;");
		m.insert('Á', "&Aacute;");
		m.insert('Â', "&Acirc;");
		m.insert('Ã', "&Atilde;");
		m.insert('Ä', "&Auml;");
		m.insert('Å', "&Aring;");
		m.insert('Æ', "&AElig;");
		m.insert('Ç', "&Ccedil;");
		m.insert('È', "&Egrave;");
		m.insert('É', "&Eacute;");
		m.insert('Ê', "&Ecirc;");
		m.insert('Ë', "&Euml;");
		m.insert('Ì', "&Igrave;");
		m.insert('Í', "&Iacute;");
		m.insert('Î', "&Icirc;");
		m.insert('Ï', "&Iuml;");
		m.insert('Ð', "&ETH;");
		m.insert('Ñ', "&Ntilde;");
		m.insert('Ò', "&Ograve;");
		m.insert('Ó', "&Oacute;");
		m.insert('Ô', "&Ocirc;");
		m.insert('Õ', "&Otilde;");
		m.insert('Ö', "&Ouml;");
		m.insert('×', "&times;");
		m.insert('Ø', "&Oslash;");
		m.insert('Ù', "&Ugrave;");
		m.insert('Ú', "&Uacute;");
		m.insert('Û', "&Ucirc;");
		m.insert('Ü', "&Uuml;");
		m.insert('Ý', "&Yacute;");
		m.insert('Þ', "&THORN;");
		m.insert('ß', "&szlig;");
		m.insert('à', "&agrave;");
		m.insert('á', "&aacute;");
		m.insert('â', "&acirc;");
		m.insert('ã', "&atilde;");
		m.insert('ä', "&auml;");
		m.insert('å', "&aring;");
		m.insert('æ', "&aelig;");
		m.insert('ç', "&ccedil;");
		m.insert('è', "&egrave;");
		m.insert('é', "&eacute;");
		m.insert('ê', "&ecirc;");
		m.insert('ë', "&euml;");
		m.insert('ì', "&igrave;");
		m.insert('í', "&iacute;");
		m.insert('î', "&icirc;");
		m.insert('ï', "&iuml;");
		m.insert('ð', "&eth;");
		m.insert('ñ', "&ntilde;");
		m.insert('ò', "&ograve;");
		m.insert('ó', "&oacute;");
		m.insert('ô', "&ocirc;");
		m.insert('õ', "&otilde;");
		m.insert('ö', "&ouml;");
		m.insert('÷', "&divide;");
		m.insert('ø', "&oslash;");
		m.insert('ù', "&ugrave;");
		m.insert('ú', "&uacute;");
		m.insert('û', "&ucirc;");
		m.insert('ü', "&uuml;");
		m.insert('ý', "&yacute;");
		m.insert('þ', "&thorn;");
		m.insert('ÿ', "&yuml;");
		m.insert('Œ', "&OElig;");
		m.insert('œ', "&oelig;");
		m.insert('Š', "&Scaron;");
		m.insert('š', "&scaron;");
		m.insert('Ÿ', "&Yuml;");
		m.insert('ƒ', "&fnof;");
		m.insert('ˆ', "&circ;");
		m.insert('˜', "&tilde;");
		m.insert('Α', "&Alpha;");
		m.insert('Β', "&Beta;");
		m.insert('Γ', "&Gamma;");
		m.insert('Δ', "&Delta;");
		m.insert('Ε', "&Epsilon;");
		m.insert('Ζ', "&Zeta;");
		m.insert('Η', "&Eta;");
		m.insert('Θ', "&Theta;");
		m.insert('Ι', "&Iota;");
		m.insert('Κ', "&Kappa;");
		m.insert('Λ', "&Lambda;");
		m.insert('Μ', "&Mu;");
		m.insert('Ν', "&Nu;");
		m.insert('Ξ', "&Xi;");
		m.insert('Ο', "&Omicron;");
		m.insert('Π', "&Pi;");
		m.insert('Ρ', "&Rho;");
		m.insert('Σ', "&Sigma;");
		m.insert('Τ', "&Tau;");
		m.insert('Υ', "&Upsilon;");
		m.insert('Φ', "&Phi;");
		m.insert('Χ', "&Chi;");
		m.insert('Ψ', "&Psi;");
		m.insert('Ω', "&Omega;");
		m.insert('α', "&alpha;");
		m.insert('β', "&beta;");
		m.insert('γ', "&gamma;");
		m.insert('δ', "&delta;");
		m.insert('ε', "&epsilon;");
		m.insert('ζ', "&zeta;");
		m.insert('η', "&eta;");
		m.insert('θ', "&theta;");
		m.insert('ι', "&iota;");
		m.insert('κ', "&kappa;");
		m.insert('λ', "&lambda;");
		m.insert('μ', "&mu;");
		m.insert('ν', "&nu;");
		m.insert('ξ', "&xi;");
		m.insert('ο', "&omicron;");
		m.insert('π', "&pi;");
		m.insert('ρ', "&rho;");
		m.insert('ς', "&sigmaf;");
		m.insert('σ', "&sigma;");
		m.insert('τ', "&tau;");
		m.insert('υ', "&upsilon;");
		m.insert('φ', "&phi;");
		m.insert('χ', "&chi;");
		m.insert('ψ', "&psi;");
		m.insert('ω', "&omega;");
		m.insert('ϑ', "&thetasym;");
		m.insert('ϒ', "&upsih;");
		m.insert('ϖ', "&piv;");
		m.insert('\u{2002}', "&ensp;");
		m.insert('\u{2003}', "&emsp;");
		m.insert('\u{2009}', "&thinsp;");
		m.insert('\u{200e}', "&lrm;");
		m.insert('\u{200f}', "&rlm;");
		m
	};
}

bitflags! {
	/// Options for HTML escaping. If you're paranoid, use `EscapeHtmlFlags::all()`.
	#[derive(Debug, Clone, Copy, Default)]
	pub struct EscapeHtmlFlags: u8 {
		/// If true, all characters > 0x7F will be escaped.
		const UNICODE = 0b00000001;
		/// If true, all characters < 0x20 and 0x7F will be escaped
		const CONTROL = 0b00000010;
	}
}

#[inline]
fn is_control_character(char: char) -> bool {
	char < ' ' || char == '\x7F'
}
#[inline]
fn is_unicode(char: char) -> bool {
	char > '\x7F'
}
#[inline]
fn is_special_html_char(char: char) -> bool {
	char == '"' || char == '&' || char == '\'' || char == '<' || char == '>'
}

fn escape_html_inner(input: &str, flags: EscapeHtmlFlags, str_index: usize) -> String {
	let mut result = String::with_capacity(input.len());
	result.push_str(&input[0..str_index]);
	for char in input[str_index..].chars() {
		if flags.contains(EscapeHtmlFlags::CONTROL) && is_control_character(char) {
			result.push_str(&format!("&#{};", char as u32));
			continue;
		}
		if flags.contains(EscapeHtmlFlags::UNICODE) {
			if let Some(escaped) = HTML_ENTITIES_MAP.get(&char) {
				result.push_str(escaped);
			} else if is_unicode(char) {
				result.push_str(&format!("&#{};", char as u32));
			} else {
				result.push(char);
			}
		} else {
			if let Some(escaped) = HTML_SPECIAL_CHARS_MAP.get(&char) {
				result.push_str(escaped);
			} else {
				result.push(char);
			}
		}
	}
	result
}

///
pub fn escape_html<'a>(input: &'a str, flags: EscapeHtmlFlags) -> Cow<'a, str> {
	for (str_index, char) in input.char_indices() {
		if
			flags.contains(EscapeHtmlFlags::UNICODE) && is_unicode(char) ||
			is_special_html_char(char) || 
			flags.contains(EscapeHtmlFlags::CONTROL) && is_control_character(char)
		{
			return Cow::Owned(escape_html_inner(input, flags, str_index));
		}
	}
	Cow::Borrowed(input)
}
fn test() {
	
}
