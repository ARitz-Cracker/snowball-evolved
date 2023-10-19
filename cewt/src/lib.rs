pub mod consts;
pub mod workarounds;

#[macro_export]
macro_rules! selector {
	($e: expr) => {{
		static SELECTOR: std::sync::OnceLock<scraper::Selector> = std::sync::OnceLock::new();
		SELECTOR.get_or_init(|| {
			scraper::Selector::parse($e).unwrap()
		})
	}};
}

// This is stupid, but because scraper::Html uses html5ever which uses tendril,
// I apparently cannot use scraper::Html accross threads, even if it is static
#[macro_export]
macro_rules! html {
	($e: expr) => {{
		scraper::Html::parse_fragment($e)
	}};
}
