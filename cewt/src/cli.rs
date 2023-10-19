use std::{fs, path::{Path, PathBuf}, ffi::OsString, collections::HashSet};
use color_eyre::eyre::Result;
pub(crate) mod codegen;
pub(crate) mod bundle;


pub(crate) fn recursive_template_search<F: FnMut(&Path, Option<&str>) -> Result<()>>(path_dir: PathBuf, exclude: &HashSet<OsString>, callback: &mut F) -> Result<()> {
	let dir_contents = fs::read_dir(path_dir)?;
	for dir_entry in dir_contents.into_iter() {
		let dir_entry = dir_entry?;
		let raw_file_name = dir_entry.file_name();
		if exclude.contains(&raw_file_name) {
			continue;
		}
		if dir_entry.file_type()?.is_dir() {
			recursive_template_search(dir_entry.path(), exclude, callback)?;
		}else if !dir_entry.file_type()?.is_file() {
			continue;
		}
		let file_name = raw_file_name.to_string_lossy();
		if !file_name.ends_with(".html") {
			continue;
		}
		(callback)(
			&dir_entry.path(),
			if file_name == "template.html" {
				None
			} else {
				Some(&file_name[0..{file_name.len() - 5}])
			}
		)?;
		
	}
	Ok(())
}
