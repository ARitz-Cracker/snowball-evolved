use std::{sync::Arc, str::FromStr, num::NonZeroU64};

use crate::error::RangeRequestParseError;
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum RangeRequestPart {
	Range(u64, u64),
	Skip(u64),
	Tail(NonZeroU64)
}
impl RangeRequestPart {
	#[inline]
	pub fn is_out_of_range(&self, total_size: u64) -> bool {
		match self {
			RangeRequestPart::Range(start, end) => *start >= total_size || *end >= total_size,
			RangeRequestPart::Skip(start) => *start >= total_size,
			RangeRequestPart::Tail(trail_val) => trail_val.get() > total_size,
		}
	}
	#[inline]
	pub fn is_range(&self) -> bool {
		match self {
			RangeRequestPart::Range(_, _) => true,
			_ => false,
		}
	}
	pub fn to_string_with_length(&self, total_size: u64) -> String {
		let mut result = String::new();
		match self {
			RangeRequestPart::Range(start, end) => {
				result.push_str(&start.to_string());
				result.push('-');
				result.push_str(&end.to_string());
			}
			RangeRequestPart::Skip(start) => {
				result.push_str(&start.to_string());
					result.push('-');
					result.push_str(&total_size.saturating_sub(1).to_string())
			},
			RangeRequestPart::Tail(trail_val) => {
				// a trail size of zero will result in something funky like "1024-1023", so that's fun.
				result.push_str(&
					total_size.saturating_sub(trail_val.get()).to_string()
				);
				result.push('-');
				result.push_str(&total_size.saturating_sub(1).to_string())
			},
		}
		result.push('/');
		result.push_str(&total_size.to_string());
		result
	}
}
impl std::fmt::Display for RangeRequestPart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			RangeRequestPart::Range(start, end) => {
				write!(f, "{}-{}", start, end)
			}
			RangeRequestPart::Skip(start) => {
				write!(f, "{}-", start)
			},
			RangeRequestPart::Tail(trail_val) => {
				write!(f, "-{}", trail_val)
			},
		}
	}
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RangeReqest {
	Single(RangeRequestPart),
	Multi(Arc<[RangeRequestPart]>)
}
impl RangeReqest {
	#[inline]
	pub fn is_single(&self) -> bool {
		match &self {
			RangeReqest::Single(_) => true,
			RangeReqest::Multi(_) => false,
		}
	}
	#[inline]
	pub fn is_multi(&self) -> bool {
		match &self {
			RangeReqest::Single(_) => true,
			RangeReqest::Multi(_) => false,
		}
	}
	#[inline]
	pub fn is_out_of_range(&self, total_size: u64) -> bool {
		match self {
			RangeReqest::Single(range_part) => range_part.is_out_of_range(total_size),
			RangeReqest::Multi(range_parts) => range_parts.iter().any(|range_part| {
				range_part.is_out_of_range(total_size)
			}),
		}
	}

	/// Assumes that the [RangeRequestPart] in an `RangeReqest::Multi` is sorted.
	/// Always returns true if `RangeReqest::Multi` contains an `RangeRequestPart::Tail` and total_size is unspecified
	pub fn has_overlapping_ranges(&self, total_size: Option<u64>) -> bool {
		let RangeReqest::Multi(parts) = self else {
			return false;
		};
		let mut minimum_value = 0;
		for part in parts.iter() {
			// If we already have a range that goes to EOF, 
			if minimum_value == u64::MAX {
				return true;
			}
			match part {
				RangeRequestPart::Range(start, end) => {
					if *start < minimum_value {
						// Overlapping ranges found
						return true;
					}
					minimum_value = end.saturating_add(1);
				}
				RangeRequestPart::Skip(start) => {
					if *start < minimum_value {
						// Overlapping ranges found
						return true;
					}
					minimum_value = u64::MAX; // Goes until EOF
				}
				RangeRequestPart::Tail(trail_val) => {
					if let Some(total_size) = total_size {
						let start = total_size.saturating_sub(trail_val.get());
						if start < minimum_value {
							// Overlapping ranges found
							return true;
						}
						minimum_value = u64::MAX; // Goes until EOF
					}else{
						return true;
					}
				}
			}
		}
		false
	}

}

fn skip_over_spaces(str_iter: &mut std::iter::Peekable<std::str::Chars>) {
	while str_iter.peek().is_some_and(|char| {*char <= ' '}) {
		str_iter.next();	
	}
}
#[inline]
fn is_next_char(str_iter: &mut std::iter::Peekable<std::str::Chars>, chr: char) -> bool {
	str_iter.next() == Some(chr)
}
fn parse_uint(str_iter: &mut std::iter::Peekable<std::str::Chars>) -> Option<u64> {
	if !str_iter.peek().is_some_and(|char| {*char >= '0' && *char <= '9'}) {
		return None;
	}
	let mut result: u64 = 0;
	while let Some(digit) = str_iter.peek().and_then(|char| {char.to_digit(10)}) {
		result = result.saturating_mul(10);
		result = result.saturating_add(digit as u64);
		str_iter.next();
	}
	return Some(result);
}
#[inline]
fn has_ended(str_iter: &mut std::iter::Peekable<std::str::Chars>) -> bool {
	str_iter.peek() == None
}

impl FromStr for RangeReqest {
    type Err = RangeRequestParseError;

	/// Parser for the http `Range` header, the `[RangeRequestPart]` in the returned `RangeReqest::Multi` is sorted.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
		if string.len() == 0 {
			return Err(RangeRequestParseError::SyntaxError);
		}
		if !string.starts_with("bytes") {
			return Err(RangeRequestParseError::UnsupportedUnit);
		}
		let str_iter = &mut string[5..].chars().peekable(); // skip "bytes"
		skip_over_spaces(str_iter);
		if !is_next_char(str_iter, '=') {
			return Err(RangeRequestParseError::UnsupportedUnit);
		}
		let mut result = Vec::with_capacity(1);
		loop {
			skip_over_spaces(str_iter);
			let maybe_start = parse_uint(str_iter);
			skip_over_spaces(str_iter);
			if !is_next_char(str_iter, '-') {
				return Err(RangeRequestParseError::SyntaxError);
			}
			skip_over_spaces(str_iter);
			let maybe_end = parse_uint(str_iter);
			match (maybe_start, maybe_end) {
				(Some(start), Some(end)) => {
					if end < start {
						return Err(RangeRequestParseError::EndBeforeStart);
					}
					result.push(RangeRequestPart::Range(start, end));
				}
				(Some(start), None) => {
					/* 
					let Some(start) = NonZeroU64::new(start) else {
						return Err(RangeRequestParseError::UselessSkip);
					};
					*/
					result.push(RangeRequestPart::Skip(start));
				}
				(None, Some(end)) => {
					let Some(end) = NonZeroU64::new(end) else {
						return Err(RangeRequestParseError::ZeroLengthTail);
					};
					
					result.push(RangeRequestPart::Tail(end));
				}
				(None, None) => {
					return Err(RangeRequestParseError::SyntaxError);
				}
			}
			skip_over_spaces(str_iter);
			if has_ended(str_iter) {
				return match result.len() {
					0 => unreachable!("result vec should have been pushed at least once"),
					1 => Ok(RangeReqest::Single(result[0])),
					_ => {
						result.sort_unstable();
						Ok(RangeReqest::Multi(result.into()))
					}
				}
			}
			if !is_next_char(str_iter, ',') {
				return Err(RangeRequestParseError::SyntaxError);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		assert_eq!(RangeReqest::from_str("bytes=200-1000, 2000-6576, 19000- "), Ok(RangeReqest::Multi(vec![RangeRequestPart::Range(200, 1000), RangeRequestPart::Range(2000, 6576), RangeRequestPart::Skip(19000)].into())))
	}
	#[cfg(test)]

	// Remaining tests brought to you by ChatGPT
	#[test]
	fn test_parse_range_request_none() {
		let result = RangeReqest::from_str("bytes=");
		assert_eq!(result, Err(RangeRequestParseError::SyntaxError));
	}

	#[test]
	fn test_parse_range_request_single_tail() {
		let result = RangeReqest::from_str("bytes=-100");
		assert_eq!(result, Ok(RangeReqest::Single(RangeRequestPart::Tail(NonZeroU64::new(100).unwrap()))));
	}

	#[test]
	fn test_parse_range_request_single_range() {
		let result = RangeReqest::from_str("bytes=500-1000");
		assert_eq!(
			result,
			Ok(RangeReqest::Single(RangeRequestPart::Range(500, 1000)))
		);
	}

	#[test]
	fn test_parse_range_request_single_skip_to_end() {
		let result = RangeReqest::from_str("bytes=1000-");
		assert_eq!(result, Ok(RangeReqest::Single(RangeRequestPart::Skip(1000))));
	}

	#[test]
	fn test_parse_range_request_multi() {
		let result = RangeReqest::from_str("bytes=1000-2000,3000-4000,5000-");
		let expected_parts = vec![
			RangeRequestPart::Range(1000, 2000),
			RangeRequestPart::Range(3000, 4000),
			RangeRequestPart::Skip(5000),
		];
		assert_eq!(
			result,
			Ok(RangeReqest::Multi(expected_parts.into()))
		);
	}

	#[test]
	fn test_parse_range_request_syntax_error() {
		let result = RangeReqest::from_str("invalid-header");
		assert_eq!(result, Err(RangeRequestParseError::UnsupportedUnit));
	}

	#[test]
	fn test_parse_range_request_unsupported_unit() {
		let result = RangeReqest::from_str("unsupported=0-100");
		assert_eq!(result, Err(RangeRequestParseError::UnsupportedUnit));
	}

	#[test]
	fn test_parse_range_request_end_before_start() {
		let result = RangeReqest::from_str("bytes=1000-500");
		assert_eq!(result, Err(RangeRequestParseError::EndBeforeStart));
	}

	/*
	#[test]
	fn test_parse_range_request_useless_skip() {
		let result = RangeReqest::from_str("bytes=0-");
		assert_eq!(result, Err(RangeRequestParseError::UselessSkip));
	}
	*/

	#[test]
	fn test_parse_range_request_invalid_tail() {
		let result = RangeReqest::from_str("bytes=-0");
		assert_eq!(result, Err(RangeRequestParseError::ZeroLengthTail));
	}
	/* 
	#[test]
	fn test_parse_range_request_not_sequential() {
		let result = RangeReqest::from_str("bytes=1000-500,600-700");
		assert_eq!(result, Err(RangeRequestParseError::NotSequential));
	}
	*/
	#[test]
	fn test_parse_range_request_malformed_numbers() {
		// Invalid start index
		let result = RangeReqest::from_str("bytes=a-100");
		assert_eq!(result, Err(RangeRequestParseError::SyntaxError));

		// Invalid end index
		let result = RangeReqest::from_str("bytes=100-b");
		assert_eq!(result, Err(RangeRequestParseError::SyntaxError));

		// Invalid range syntax
		let result = RangeReqest::from_str("bytes=100-200-300");
		assert_eq!(result, Err(RangeRequestParseError::SyntaxError));
	}

	#[test]
	fn test_parse_range_request_malformed_ranges() {
		// Non-numeric range value
		let result = RangeReqest::from_str("bytes=100-abc");
		assert_eq!(result, Err(RangeRequestParseError::SyntaxError));

		let result = RangeReqest::from_str("bytes=abc-,200-300");
		assert_eq!(result, Err(RangeRequestParseError::SyntaxError));

		let result: Result<RangeReqest, RangeRequestParseError> = RangeReqest::from_str("bytes=100-,abc-300");
		assert_eq!(result, Err(RangeRequestParseError::SyntaxError));
	}

	#[test]
	fn test_parse_range_request_spacing() {
		// Test with spaces around punctuation and numbers
		let result = RangeReqest::from_str("bytes= 100 - 200, 300 - 400 , 500 - ");
		let expected_parts = vec![
			RangeRequestPart::Range(100, 200),
			RangeRequestPart::Range(300, 400),
			RangeRequestPart::Skip(500),
		];
		assert_eq!(result, Ok(RangeReqest::Multi(expected_parts.into())));
	}
}
