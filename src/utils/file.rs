use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
};

pub const BUFFER_SIZE: usize = 8192;

/// Read a file and apply a function to it's contents
pub fn read_file_and<F>(filepath: &str, skip_bytes: u64, mut func: F) -> std::io::Result<()>
where
    F: FnMut(&[u8], usize),
{
    let mut file = File::open(filepath)?;
    let file_size = file.metadata().unwrap().len() as usize;

    let mut pos_in_file = file.seek(SeekFrom::Start(skip_bytes)).unwrap_or(0) as usize;
    let mut reader = BufReader::new(file);

    if file_size <= BUFFER_SIZE {
        let mut buffer = Vec::with_capacity(BUFFER_SIZE);
        reader.read_to_end(&mut buffer)?;

        func(&buffer, pos_in_file);
    } else {
        let mut buffer = [0; BUFFER_SIZE];
        loop {
            // NOTE: When reading a file that can fill a entire buffer of BUFFER_SIZE but it's not big enough to
            // fill a second buffer of BUFFER_SIZE, the read method will only change the
            // contents of the buffer from 0..total_bytes_read, the rest of the contents in the
            // buffer will be from the read that happened in the previous iteration.
            let total_bytes_read = reader.read(&mut buffer).unwrap();

            if total_bytes_read == 0 {
                break;
            }

            func(&buffer[..total_bytes_read], pos_in_file);

            pos_in_file += total_bytes_read;
        }
    }

    Ok(())
}

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
        .filter(|path| filetypes.contains(&path.extension().unwrap_or_default().to_str().unwrap_or_default()))
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
