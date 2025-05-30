use {
    relative_path::PathExt,
    std::{
        ffi::OsStr,
        path::{Path, PathBuf},
    },
};

pub trait PackLoaderContext {
    type AssetReader<'ctx>: std::io::Read + 'ctx
    where
        Self: 'ctx;

    fn load_asset<'ctx>(&'ctx self, name: &str) -> anyhow::Result<Self::AssetReader<'ctx>>;

    fn all_files_with_ext(&self, ext: &str) -> anyhow::Result<Vec<String>>;
}

pub struct DirectoryLoader {
    root: PathBuf,
}

impl DirectoryLoader {
    pub fn new<P: Into<PathBuf>>(root: P) -> DirectoryLoader {
        DirectoryLoader { root: root.into() }
    }
}

impl PackLoaderContext for DirectoryLoader {
    type AssetReader<'ctx> = std::fs::File;

    fn load_asset<'ctx>(&'ctx self, name: &str) -> anyhow::Result<Self::AssetReader<'ctx>> {
        Ok(std::fs::File::open(self.root.join(name))?)
    }

    fn all_files_with_ext(&self, ext: &str) -> anyhow::Result<Vec<String>> {
        let mut files = vec![];

        visit_dir_ext(&mut files, &self.root, &self.root, ext)?;

        Ok(files)
    }
}

fn visit_dir_ext(
    files: &mut Vec<String>,
    base: &Path,
    dir: &Path,
    ext: &str,
) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir_ext(files, base, &path, ext)?;
        } else if path.extension() == Some(OsStr::new(ext)) {
            files.push(path.relative_to(base)?.into_string());
        }
    }
    Ok(())
}
