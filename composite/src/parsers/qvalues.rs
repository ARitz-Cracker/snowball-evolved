use std::{sync::Arc, cmp::Ordering, ops::Deref};
use lazy_regex::regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchPrecedent {
	None,
	Wildcard,
	Exact
}

#[derive(Clone, Debug)]
pub(crate) struct QualityValue {
	quality: u16, // RFC9110: A sender of qvalue MUST NOT generate more than three digits after the decimal point.
	value: Arc<str>
}
impl QualityValue {
	pub fn new<S>(value: S, quality: u16) -> Self where S: Into<Arc<str>> {
		Self {
			quality,
			value: value.into()
		}
	}
	pub fn quality(&self) -> u16 {
		self.quality
	}
	pub fn value(&self) -> &str {
		&self.value
	}
	pub fn value_matches(&self, check_value: &str) -> MatchPrecedent {
		if self.value.ends_with("/*") {
			let mime_major = &self.value[0..self.value.len() - 1];
			if check_value.starts_with(mime_major) {
				return MatchPrecedent::Wildcard;
			}else{
				return MatchPrecedent::None;
			}
		}
		if self.value() == check_value {
			MatchPrecedent::Exact
		}else{
			MatchPrecedent::None
		}
	}
}
impl PartialEq for QualityValue {
	fn eq(&self, other: &Self) -> bool {
		self.quality.eq(&other.quality)
	}
}
impl Eq for QualityValue {}
impl PartialOrd for QualityValue {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.quality.partial_cmp(&other.quality)
	}
}
impl Ord for QualityValue {
	fn cmp(&self, other: &Self) -> Ordering {
		self.quality.cmp(&other.quality)
	}
}

fn parse_quality_value(input: &str) -> u16 {
	if input == "0" {
		return 0;
	}else if input.starts_with("0.") {
		// Only read up to 3 digits, since the RFC says clients can't make more than that
		let max_len = input.len().min(5);
		let mul_val = 10u16.pow(5 - max_len as u32);
		return input[2 .. input.len().min(5)].parse().unwrap_or(1) * mul_val;
	}else{
		// effective clamp to 1
		return 1000;
	}
}

#[derive(Clone, Debug)]
pub struct QualitySorter {
	// The use of arrays instead of HashMap and HashSet is intentional here, it is assumed that for most situations,
	// the client will have a list of preferences that are <=7 items long, and therefore comparing against all items in
	// the arrays will be faster than calculating the hash potentially multiple times.
	values: Arc<[QualityValue]>,
	wildcard_quality: u16
}
impl QualitySorter {
	pub fn new(header_value: &str, wildcard_implicitly_allowed: bool) -> Self {
		let mut values: Vec::<QualityValue> = Vec::with_capacity(8);
		let mut wildcard_quality: u16 = wildcard_implicitly_allowed.into();
		for captures in regex!(
			r"\s*(\S+?)(?:\s*;\s*[^q]\S+?\s*)*(?:\s*;\s*q\s*=\s*([0-1]?\.?\d*?))?\s*(?:\s*;\s*[^q]\S+?\s*)*?\s*(?:,|$)"
		).captures_iter(header_value) {
			let Some(value) = captures.get(1) else {
				continue;
			};
			let value = value.as_str().to_string();
			let quality = parse_quality_value(captures.get(2).map(|f|{f.as_str()}).unwrap_or("1"));
			if value == "*/*" || value == "*" {
				wildcard_quality = quality;
			}else{
				values.push(QualityValue::new(value, quality));
			}
		}
		Self {
			values: values.into(),
			wildcard_quality
		}
	}
	
	pub fn allows(&self, check_value: &str) -> bool {
		if self.wildcard_quality > 0 {
			return true;
		}
		let Some(qvalue) = self.values.iter().find(|qvalue| {
			qvalue.value_matches(check_value) != MatchPrecedent::None
		}) else {
			return false;
		};
		qvalue.quality() > 0
	}
	pub fn quality_of(&self, check_value: &str) -> (u16, MatchPrecedent) {
		let Some((qvalue, match_precedent)) = self.values.iter().find_map(|qvalue| {
			let match_precedent = qvalue.value_matches(check_value);
			if match_precedent == MatchPrecedent::None {
				return None;
			}
			Some((qvalue, match_precedent))
		}) else {
			return (
				self.wildcard_quality,
				if self.wildcard_quality == 0 {
					MatchPrecedent::None
				} else {
					MatchPrecedent::Wildcard
				}
			);
		};
		(qvalue.quality, match_precedent)
	}
	pub fn negotiate<'a, I, S>(
		&'a self,
		iter: &mut I
	) -> Option<&'a str> where I: Iterator<Item = S>, S: Deref<Target = &'a str> {
		let mut result = "";
		let mut highest_quality = 0;
		let mut highest_match_precedent = MatchPrecedent::None;
		for check_value in iter {
			let (quality, match_precedent) = self.quality_of(&check_value);
			if
				(quality > highest_quality) ||
				// Some people might be like "image/*, image/jpeg;q=0"
				(quality == 0 && match_precedent > highest_match_precedent)
			{
				result = &check_value;
				highest_quality = quality;
				highest_match_precedent = match_precedent;
			}
		}
		if highest_quality == 0 {
			return None;
		}
		return Some(result);
	}
	pub fn ordered_preferences(&self) -> OrderedRreferencesResult {
		let mut values: Vec<QualityValue> = self.values.to_vec();
		values.sort_unstable();
		return OrderedRreferencesResult {
			allowed: values.iter().filter_map(|v| {
				if v.quality == 0 {
					return None;
				}
				Some(v.value.clone())
			}).collect(),
			disallowed: values.iter().filter_map(|v| {
				if v.quality > 0 {
					return None;
				}
				Some(v.value.clone())
			}).collect(),
			any_allowed: self.wildcard_quality > 0
		}
		//values.iter().map(|f|{f.value.clone()}).collect()
	}
}
#[derive(Clone, Debug)]
pub struct OrderedRreferencesResult {
	pub allowed: Vec<Arc<str>>,
	pub disallowed: Vec<Arc<str>>,
	pub any_allowed: bool
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let qsorter = QualitySorter::new("*/*;q=0,text/*,image/svg+xml;q=0.5;uselessdata;useless=data", true);
		println!("qsorter 1: {:#?}", qsorter);
		assert_eq!(qsorter.allows("image/jpeg"), false);
		assert_eq!(qsorter.allows("text/html"), true);
		assert_eq!(qsorter.allows("image/svg+xml"), true);

		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "text/plain"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["text/plain", "image/svg+xml"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "image/jpeg"].iter()), Some("image/svg+xml"));

		let qsorter = QualitySorter::new("text/*,image/svg+xml;q=0.5;uselessdata;useless=data", true);
		println!("qsorter 2: {:#?}", qsorter);
		assert_eq!(qsorter.allows("image/jpeg"), true);
		assert_eq!(qsorter.allows("text/html"), true);
		assert_eq!(qsorter.allows("image/svg+xml"), true);

		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "text/plain"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["text/plain", "image/svg+xml"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "image/jpeg"].iter()), Some("image/svg+xml"));

		let qsorter = QualitySorter::new("text/*,image/svg+xml;q=0.5;uselessdata;useless=data", false);
		println!("qsorter 3: {:#?}", qsorter);
		assert_eq!(qsorter.allows("image/jpeg"), false);
		assert_eq!(qsorter.allows("text/html"), true);
		assert_eq!(qsorter.allows("image/svg+xml"), true);

		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "text/plain"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["text/plain", "image/svg+xml"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "image/jpeg"].iter()), Some("image/svg+xml"));

		let qsorter = QualitySorter::new("text/*,image/svg+xml;q=0.5;uselessdata;useless=data,*/*;q=0.11111111111", false);
		println!("qsorter 4: {:#?}", qsorter);
		assert_eq!(qsorter.allows("image/jpeg"), true);
		assert_eq!(qsorter.allows("text/html"), true);
		assert_eq!(qsorter.allows("image/svg+xml"), true);

		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "text/plain"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["text/plain", "image/svg+xml"].iter()), Some("text/plain"));
		assert_eq!(qsorter.negotiate(&mut ["image/svg+xml", "image/jpeg"].iter()), Some("image/svg+xml"));
	}
}
