use std::{num::NonZeroU64, fmt::Write};

use crate::{parsers::ranges::RangeRequestPart, error::RangeResponseError};

#[inline]
fn base_10_str_len(number: u64) -> u32 {
	number.checked_ilog10().unwrap_or_default() + 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeResponseHeader {
	start: u64,
	end: u64,
	total_size: Option<NonZeroU64>
}
impl RangeResponseHeader {
	/// Makes a new `RangeResponseHeader`. This will return `None` if the `RangeRequestPart` is a partial range and if
	/// `total_size` is also `None`. `total_size`
	pub fn new(req: &RangeRequestPart, total_size: Option<NonZeroU64>) -> Result<Self, RangeResponseError> {
		if total_size.is_some_and(|total_size| {
			req.is_out_of_range(total_size.get())
		}) {
			return Err(RangeResponseError::OutOfRange(total_size.expect("is_some() was called to get here").get()));
		}
		match req {
			RangeRequestPart::Range(start, end) => {
				Ok(
					Self {
						start: *start,
						end: *end,
						total_size
					}
				)
			},
			RangeRequestPart::Skip(start) => {
				let Some(total_size) = total_size else {
					return Err(RangeResponseError::RelativeSliceWithUnknownFullLength);
				};
				Ok(
					Self {
						start: *start,
						end: total_size.get() - 1,
						total_size: Some(total_size)
					}
				)
			},
			RangeRequestPart::Tail(trail_val) => {
				let Some(total_size) = total_size else {
					return Err(RangeResponseError::RelativeSliceWithUnknownFullLength);
				};
				Ok(
					Self {
						start: total_size.get().saturating_sub(trail_val.get()),
						end: total_size.get() - 1,
						total_size: Some(total_size)
					}
				)
			},
		}
	}
	
	/// Returns how big the resulting .to_string() will be
	pub fn str_len(&self) -> u32 {
		base_10_str_len(self.start) +
		base_10_str_len(self.end) +
		2 + // '-', '/'
		if let Some(total_size) = self.total_size {
			base_10_str_len(total_size.get())
		} else {
			1 // '*'
		} as u32
	}

	#[inline]
	pub fn range_len(&self) -> NonZeroU64 {
		NonZeroU64::new(
			self.end.saturating_sub(self.start).saturating_add(1)
		).expect("a saturating add of 1 after a saturating sub on a uint should always be > 0")
	}

	#[inline]
	pub fn start(&self) -> u64 {
		self.start
	}
}
impl std::fmt::Display for RangeResponseHeader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}-{}/", self.start, self.end)?;
		if let Some(total_size) = self.total_size {
			write!(f, "{}", total_size)
		} else {
			f.write_char('*')
		}
	}
}
