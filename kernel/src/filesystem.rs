use alloc::{string::String, vec, vec::Vec};

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub data: Vec<u8>,
}
#[derive(Debug)]
pub struct Directory {
    pub name: String,
    pub files: Vec<FileType>,
}

#[derive(Debug)]
enum FileType {
    File(File),
    Directory(Directory),
}

impl File {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: vec![],
        }
    }
    pub fn with_data(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.data = data.into();
        self
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}
impl Directory {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            files: vec![],
        }
    }
    pub fn move_file(&mut self, file: File) {
        self.files.push(FileType::File(file));
    }

    pub fn files(&self) -> Vec<&File> {
        self.files
            .iter()
            .filter_map(|f| {
                if let FileType::File(file) = f {
                    Some(file)
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn files_mut(&mut self) -> Vec<&mut File> {
        self.files
            .iter_mut()
            .filter_map(|f| {
                if let FileType::File(file) = f {
                    Some(file)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn directories(&self) -> Vec<&Directory> {
        self.files
            .iter()
            .filter_map(|f| {
                if let FileType::Directory(directory) = f {
                    Some(directory)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn move_directory(&mut self, directory: Directory) {
        self.files.push(FileType::Directory(directory));
    }
}
pub fn create_directory(name: impl Into<String>) {}
