use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{fs, io};

// QuickAccessEntry represents each Quick Access list entry.
// It is always a directory.
#[derive(Serialize, Deserialize, Clone)]
pub struct QuickAccessEntry {
    pub name: OsString,
    pub path: PathBuf,
}

// FeEntry represents an entry in the file explorer.
// It is mostly a subset of fs::DirEntry, but we abstract
// away the properties we don't need to make it easier to handle
// and mock for testing purposes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeEntry {
    pub name: OsString,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_exe: bool,
    // entry size in bytes
    pub size: u64,
}

pub fn fs_to_fe_entry(fs_entry: fs::DirEntry) -> Result<FeEntry, io::Error> {
    let file_type = fs_entry.file_type()?;
    let metadata = fs_entry.metadata()?;
    let is_dir = file_type.is_dir();

    Ok(FeEntry {
        name: fs_entry.file_name(),
        path: fs_entry.path(),
        is_dir,
        is_exe: is_exe(fs_entry),
        size: metadata.len(),
    })
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn toggle(&self) -> SortOrder {
        match self {
            SortOrder::Asc => SortOrder::Desc,
            SortOrder::Desc => SortOrder::Asc,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]

pub enum DirSorting {
    FileNameAlphabetically(SortOrder),
    FileSize(SortOrder),
}

impl DirSorting {
    pub fn get_sort_icon(&self) -> &'static str {
        match self {
            DirSorting::FileNameAlphabetically(SortOrder::Asc)
            | DirSorting::FileSize(SortOrder::Asc) => "⬆",
            DirSorting::FileNameAlphabetically(SortOrder::Desc)
            | DirSorting::FileSize(SortOrder::Desc) => "⬇",
        }
    }
}

pub fn compare_entries(a: &FeEntry, b: &FeEntry, sorting: &DirSorting) -> Ordering {
    let a_is_dir = a.is_dir;
    let b_is_dir = b.is_dir;

    let order = match (a_is_dir, b_is_dir) {
        // dirs will always come first, no matter the ordering
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => match sorting {
            DirSorting::FileNameAlphabetically(SortOrder::Asc) => a.name.cmp(&b.name),
            DirSorting::FileNameAlphabetically(SortOrder::Desc) => b.name.cmp(&a.name),
            DirSorting::FileSize(SortOrder::Asc) => {
                let a_size = a.size;
                let b_size = b.size;
                a_size.cmp(&b_size)
            }
            DirSorting::FileSize(SortOrder::Desc) => {
                let a_size = a.size;
                let b_size = b.size;
                b_size.cmp(&a_size)
            } // _ => Ordering::Equal,
        },
    };

    return order;
}

// returns a valid file name. If a file with that name already exists
// among the entries, append `-2`. If a `-2` already exists, append `-3` and so on.
pub fn get_valid_new_file(new_file_name: &OsString, entries: &Vec<FeEntry>) -> OsString {
    let mut valid_name = new_file_name.clone();
    let mut counter = 1;

    let is_name_taken =
        |name: &OsString| -> bool { entries.iter().any(|entry| &entry.name == name) };

    while is_name_taken(&valid_name) {
        counter += 1;
        valid_name = append_suffix(new_file_name, counter);
    }

    valid_name
}

fn append_suffix(base: &OsString, counter: usize) -> OsString {
    let mut new_name = base.clone().into_string().unwrap_or_default();
    new_name.push_str(&format!("-{}", counter));
    OsString::from(new_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::path::Path;
    use tempfile::tempdir;

    fn create_file(path: &Path) {
        File::create(path).unwrap();
    }

    fn create_dir(path: &Path) {
        fs::create_dir(path).unwrap();
    }

    #[test]
    fn test_compare_entries() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        let file_a_path = dir_path.join("a.txt");
        let file_b_path = dir_path.join("b.txt");
        let dir_c_path = dir_path.join("c_dir");

        create_file(&file_a_path);
        create_file(&file_b_path);
        create_dir(&dir_c_path);

        let file_a = fs_to_fe_entry(
            fs::read_dir(dir_path)
                .unwrap()
                .find(|e| e.as_ref().unwrap().path() == file_a_path)
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        let file_b = fs_to_fe_entry(
            fs::read_dir(dir_path)
                .unwrap()
                .find(|e| e.as_ref().unwrap().path() == file_b_path)
                .unwrap()
                .unwrap(),
        )
        .unwrap();
        let dir_c = fs_to_fe_entry(
            fs::read_dir(dir_path)
                .unwrap()
                .find(|e| e.as_ref().unwrap().path() == dir_c_path)
                .unwrap()
                .unwrap(),
        )
        .unwrap();

        // Test sorting alphabetically ascending
        let sorting = DirSorting::FileNameAlphabetically(SortOrder::Asc);
        assert_eq!(compare_entries(&file_a, &file_b, &sorting), Ordering::Less);
        assert_eq!(
            compare_entries(&file_b, &file_a, &sorting),
            Ordering::Greater
        );
        assert_eq!(compare_entries(&file_a, &file_a, &sorting), Ordering::Equal);
        assert_eq!(compare_entries(&dir_c, &file_a, &sorting), Ordering::Less);
        assert_eq!(
            compare_entries(&file_a, &dir_c, &sorting),
            Ordering::Greater
        );

        // Test sorting alphabetically descending
        let sorting = DirSorting::FileNameAlphabetically(SortOrder::Desc);
        assert_eq!(
            compare_entries(&file_a, &file_b, &sorting),
            Ordering::Greater
        );
        assert_eq!(compare_entries(&file_b, &file_a, &sorting), Ordering::Less);
        assert_eq!(compare_entries(&file_a, &file_a, &sorting), Ordering::Equal);
        assert_eq!(compare_entries(&dir_c, &file_a, &sorting), Ordering::Less);
        assert_eq!(
            compare_entries(&file_a, &dir_c, &sorting),
            Ordering::Greater
        );
    }

    #[test]
    fn test_sort_vec_of_direntry() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();

        let file_a_path = dir_path.join("a.txt");
        let file_b_path = dir_path.join("b.txt");
        let dir_c_path = dir_path.join("c_dir");

        create_file(&file_a_path);
        create_file(&file_b_path);
        create_dir(&dir_c_path);

        let mut entries: Vec<FeEntry> = fs::read_dir(dir_path)
            .unwrap()
            .filter_map(Result::ok)
            .map(fs_to_fe_entry)
            .filter_map(Result::ok)
            .collect();

        let sorting = DirSorting::FileNameAlphabetically(SortOrder::Asc);

        entries.sort_by(|a, b| compare_entries(a, b, &sorting));

        let sorted_names: Vec<String> = entries
            .iter()
            .map(|e| e.name.clone().into_string().to_owned().unwrap())
            .collect();

        assert_eq!(sorted_names, vec!["c_dir", "a.txt", "b.txt"]);

        // TODO test other sortings
    }

    // TODO:
    // test_fs_to_fe_entry
}

pub fn is_exe(fs_entry: fs::DirEntry) -> bool {
    #[cfg(unix)]
    {
        let metadata = fs_entry.metadata().unwrap();
        let permissions = metadata.permissions();
        // On Unix, check the execute bits
        Ok(permissions.mode() & 0o111 != 0)
    }

    #[cfg(windows)]
    {
        let file_type = fs_entry.file_type().unwrap();
        if file_type.is_file() {
            // Simple heuristic: check if the file has an executable extension
            let path = fs_entry.path();
            if let Some(extension) = path.extension() {
                let extensions = ["exe", "bat", "cmd", "com"];
                return extensions.iter().any(|&ext| extension == ext);
            } else {
                return false;
            }
        } else {
            return false;
        }
    }
}
