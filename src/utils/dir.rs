use std::cmp::Ordering;
use std::fs;

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

fn entry_name(entry: &fs::DirEntry) -> String {
    entry.file_name().into_string().unwrap_or_default()
}

pub fn compare_entries(a: &fs::DirEntry, b: &fs::DirEntry, sorting: &DirSorting) -> Ordering {
    let a_is_dir = a.metadata().map(|m| m.is_dir()).unwrap_or(false);
    let b_is_dir = b.metadata().map(|m| m.is_dir()).unwrap_or(false);

    let order = match (a_is_dir, b_is_dir) {
        // dirs will always come first, no matter the ordering
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => match sorting {
            DirSorting::FileNameAlphabetically(SortOrder::Asc) => entry_name(a).cmp(&entry_name(b)),
            DirSorting::FileNameAlphabetically(SortOrder::Desc) => {
                entry_name(b).cmp(&entry_name(a))
            }
            DirSorting::FileSize(SortOrder::Asc) => {
                let a_size = a.metadata().map(|m| m.len()).unwrap_or(0);
                let b_size = b.metadata().map(|m| m.len()).unwrap_or(0);
                a_size.cmp(&b_size)
            }
            DirSorting::FileSize(SortOrder::Desc) => {
                let a_size = a.metadata().map(|m| m.len()).unwrap_or(0);
                let b_size = b.metadata().map(|m| m.len()).unwrap_or(0);
                b_size.cmp(&a_size)
            }
            // _ => Ordering::Equal,
        },
    };

    return order;
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

        let file_a = fs::read_dir(dir_path)
            .unwrap()
            .find(|e| e.as_ref().unwrap().path() == file_a_path)
            .unwrap()
            .unwrap();
        let file_b = fs::read_dir(dir_path)
            .unwrap()
            .find(|e| e.as_ref().unwrap().path() == file_b_path)
            .unwrap()
            .unwrap();
        let dir_c = fs::read_dir(dir_path)
            .unwrap()
            .find(|e| e.as_ref().unwrap().path() == dir_c_path)
            .unwrap()
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

        let mut entries: Vec<fs::DirEntry> = fs::read_dir(dir_path)
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        let sorting = DirSorting::FileNameAlphabetically(SortOrder::Asc);

        entries.sort_by(|a, b| compare_entries(a, b, &sorting));

        let sorted_names: Vec<String> = entries.iter().map(entry_name).collect();

        assert_eq!(sorted_names, vec!["c_dir", "a.txt", "b.txt"]);
    }
}
