use std::path::PathBuf;

pub fn get_all_files_from_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for path in paths {
        if path.is_dir() {
            files.extend(get_all_files_from_dir(path));
        } else {
            files.push(path);
        }
    }

    files
}

pub fn filter_filetypes(files: Vec<PathBuf>, filetypes: &[&str]) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|path| {
            filetypes.contains(
                &path
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            )
        })
        .collect()
}

fn get_all_files_from_dir(dir: PathBuf) -> Vec<PathBuf> {
    let mut filepaths: Vec<PathBuf> = Vec::new();

    visit_dirs(dir, &mut |file| filepaths.push(file)).unwrap();

    filepaths
}

fn visit_dirs(dir: PathBuf, cb: &mut dyn FnMut(PathBuf)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(path, cb)?;
            } else {
                cb(entry.path());
            }
        }
    }
    Ok(())
}
