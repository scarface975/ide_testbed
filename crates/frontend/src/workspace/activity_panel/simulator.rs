use std::{path::{Path, PathBuf}, sync::Arc};

pub struct File {
    pub data: Vec<u8>,
    pub path: PathBuf,
    pub mode: u32
}


pub struct Directory {
    pub path: PathBuf,
}
// I have to do the conversion from the vfs to the fs on this side since Arcs etc will not serialize
// I assume the parent path has already been made for me
pub fn convert(directory: &Arc<crate::vfs::Directory>, parent_path: &Path) -> (Vec<File>, Vec<Directory>) {
    let path = parent_path.join(&*directory.name.lock_ref());
    let mut current_files: Vec<File> = directory.files.lock_ref()
        .iter()
        .map(|file| File {
            data: file.data.lock_ref().clone(),
            path: path.join(&*file.name.lock_ref()),
            mode: file.mode.get(),
        })
        .collect();

    let mut current_directories: Vec<Directory> = directory.directories.lock_ref()
        .iter()
        .flat_map(|directory| {
            let (files, directories) = convert(directory, &path);
            current_files.extend(files);
            directories.into_iter()
        })
        .collect();
    current_directories.insert(0, Directory { path });
        
    (current_files, current_directories)
}





pub struct Simulator {
    
}



