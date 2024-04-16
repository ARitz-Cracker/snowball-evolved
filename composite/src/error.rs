#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RangeRequestParseError {
	#[error("Range header couldn't be parsed")]
	SyntaxError,
	#[error("Range unit unsupported")]
	UnsupportedUnit,
	#[error("Range start index was larger than end index")]
	EndBeforeStart,
	#[error("Range values not sequential")]
	NotSequential,
	#[error("Tail must be greater than 0")]
	ZeroLengthTail
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum RangeResponseError {
	#[error("Rage specified is outside of the resouce's bounds")]
	OutOfRange(u64),
	#[error("Cannot use a \"skip\" or \"trail\" range request")]
	RelativeSliceWithUnknownFullLength,
	#[error("Requested ranges overlap")]
	RagesOverlap
}
