pub(crate) enum FileFilter {
    None,
    Extension(ExtensionFilter),
}

impl FileFilter {
    pub(crate) fn apply(&self, file_name: &str) -> bool {
        match self {
            FileFilter::None => true,
            FileFilter::Extension(filter) => filter.apply(file_name),
        }
    }

    pub(crate) fn extension(ext: &str) -> FileFilter {
        FileFilter::Extension(ExtensionFilter::new(ext))
    }
}

pub(crate) struct ExtensionFilter {
    ext: String,
}

impl ExtensionFilter {
    fn new(ext: &str) -> ExtensionFilter {
        ExtensionFilter {
            ext: ext.to_string(),
        }
    }

    fn apply(&self, file_name: &str) -> bool {
        let ext = file_name.split('.').last();
        if let Some(ext) = ext {
            return ext == self.ext;
        }

        false
    }
}
