use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::io::Read;
use std::io::Write;
use index_bloom::Index;
use crate::errors::Error;

/// A full-text search index with file system operations.
pub struct FsIndex {
    index: Index
}

impl FsIndex {
    /// Constructs a new, empty `FsIndex` with the specified error_rate.
    ///
    /// The `error_rate` is the probability of false positive when searching for keywords
    ///
    /// # Example
    ///
    /// ```
    /// # use cli_bloom::FsIndex;
    /// let mut fs_index = FsIndex::new(0.00001);
    /// ```
    pub fn new(error_rate: f32) -> Self {
        FsIndex {
            index: Index::new(error_rate)
        }
    }

    /// Ingest a file or a directory content.
    ///
    /// Insert the content designated by the `source` parameter.
    /// If `source` is a file, ingest its content. If `source` is a directory, ingests all these files at the first level.
    /// The document key is the file path.
    ///
    /// # Panics
    ///
    /// Panics if the `source` parameter is not a regular file, directory or if the content cannot be read.
    ///
    /// # Example
    ///
    /// ```
    /// # use cli_bloom::FsIndex;
    /// # fn search_index()  {
    /// let mut fs_index = FsIndex::new(0.00001);
    /// fs_index.ingest("/foo/bar");
    /// # }
    /// ```
    pub fn ingest(&mut self, source: &str) {
        let src_path = PathBuf::from(source);
        if src_path.is_file() {
            match self.index_file(src_path) {
                Ok(_) => return,
                Err(error) => panic!("{}", error)
            }
        } else if src_path.is_dir() {
            match self.index_directory(src_path) {
                Ok(_) => return,
                Err(error) => panic!("{}", error)
            }
        } else {
            panic!("source type must be file or directory");
        }
    }

    /// Search keywords in every files.
    ///
    /// Splits `keywords` and searches for each word in all documents with a boolean AND.
    /// The result may contain false positives (documents not containing all the keywords) according to an error rate set at the creation of the `FsIndex` (see [`FsIndex::new`]).
    /// Return `None` if nothing match.
    ///
    /// # Panics
    ///
    /// Panics if the `keywords` cannot be processed.
    ///
    /// # Example
    ///
    /// ```
    /// # use cli_bloom::FsIndex;
    /// # fn search_index() {
    /// # let fs_index = FsIndex::new(0.00001);
    /// let hits = fs_index.search("content");
    /// match hits {
    ///      Some(documents) => {
    ///          for doc in documents {
    ///             println!("Found at {}", doc);
    ///          }
    ///      },
    ///      None => println!("Not found")
    /// }
    /// # }
    /// ```
    pub fn search(&self, keywords: &str) -> Option<Vec<&String>> {
        match self.index.search(keywords) {
            Ok(result) => return result,
            Err(error) => panic!("Error while searching for {} : {}", keywords, error)
        }
    }

    /// Restore a `FsIndex` from a previous dump.
    ///
    /// A dump is a `FsIndex` serialized in JSON format.
    ///
    /// # Panics
    ///
    /// Panics if the content is not a valid `FsIndex` representation.
    ///
    /// # Example
    ///
    /// ```
    /// # use cli_bloom::FsIndex;
    /// # fn search_index()  {
    /// let fs_index = FsIndex::restore("/foo/dump.json");
    /// # }
    /// ```
    pub fn restore(path :&str) -> Self {
        if Path::new(path).is_file() {
            let serialized = fs::read_to_string(path).expect(format!("Unable to read dump file {}", &path).as_str());
            let deserialized = Index::restore(&serialized);
            FsIndex {
                index: deserialized
            }
        } else {
            panic!(format!("File not found {}", &path));
        }
    }

    /// Dump a `FsIndex` in a file.
    ///
    /// Create a Json representation of the current `FsIndex` and write it at the location designated by `path`.
    ///
    /// # Panics
    ///
    /// Panics if it is not possible to create the file at `path` or if it is impossible to serialize the `FsIndex`.
    ///
    /// # Example
    ///
    /// ```
    /// # use cli_bloom::FsIndex;
    /// # fn search_index()  {
    /// let mut fs_index = FsIndex::new(0.00001);
    /// fs_index.ingest("/foo/bar.txt");
    /// fs_index.dump("/foo/dump.json");
    /// # }
    /// ```
    pub fn dump(&self, path: &str) {
        let dest = Path::new(&path);
        let mut output_file = File::create(dest).expect(format!("Impossible to create dump file {}", &path).as_str());
        let serialized = serde_json::to_string(&self.index).expect("Impossible to serialize file");
        write!(output_file, "{}\n", serialized).expect("Impossible to write dump file");
    }

    fn index_directory(&mut self, path: PathBuf) -> Result<(), Error> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::metadata(&path)?;
            if metadata.is_file() {
                match self.index_file(path) {
                    Ok(_) => continue,
                    Err(error) => match error {
                        Error::IndexInvalidData(_) => continue,
                        _ => return Err(error)
                    }
                }
            }
        }
        Ok(())
    }

    fn index_file(&mut self, path: PathBuf) -> Result<(), Error> {
        let mut content = String::new();
        let mut file = File::open(&path)?;
        file.read_to_string(&mut content)?;
        self.index.ingest(path.to_str().unwrap().to_string(), &content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_source_is_file() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/simple_content.txt");
        assert_eq!(vec!["./test/data/simple_content.txt"], index.search("word1").unwrap());
    }

    #[test]
    fn index_source_is_directory() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/simple_directory");
        assert_eq!(vec!["./test/data/simple_directory/file1.txt"], index.search("word1").unwrap());
        assert_eq!(vec!["./test/data/simple_directory/file2.txt"], index.search("word4").unwrap());
    }

    #[test]
    #[should_panic(expected="Error source must be an UTF-8 text file")]
    fn index_source_is_binary_file() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/image_file.png");
    }

    #[test]
    #[should_panic(expected="source type must be file or directory")]
    fn index_source_is_unsupported() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/unknown_source");
    }

    #[test]
    fn index_source_is_directory_with_mixed_content() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/directory_with_mixed_content");
        assert_eq!(vec!["./test/data/directory_with_mixed_content/simple_content.txt"], index.search("word1").unwrap());
    }

    #[test]
    fn file_simple_content() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/simple_content.txt");
        assert_eq!(vec!["./test/data/simple_content.txt"], index.search("word1").unwrap());
        assert_eq!(vec!["./test/data/simple_content.txt"], index.search("word2").unwrap());
        assert_eq!(vec!["./test/data/simple_content.txt"], index.search("word3").unwrap());
        assert_eq!(vec!["./test/data/simple_content.txt"], index.search("word4").unwrap());
    }

    #[test]
    fn simple_directory_content() {
       let mut index = FsIndex::new(0.01);
       index.ingest("./test/data/simple_directory");
       assert_eq!(vec!["./test/data/simple_directory/file1.txt"], index.search("word1").unwrap());
       assert_eq!(vec!["./test/data/simple_directory/file1.txt"], index.search("word2").unwrap());
       assert_eq!(vec!["./test/data/simple_directory/file1.txt"], index.search("word3").unwrap());
       assert_eq!(vec!["./test/data/simple_directory/file2.txt"], index.search("word4").unwrap());
       assert_eq!(vec!["./test/data/simple_directory/file2.txt"], index.search("word5").unwrap());
    }

    #[test]
    fn random_directory_content() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/random_directory");
        assert_eq!(vec!["./test/data/random_directory/file1.txt"], index.search("word1").unwrap());
        assert_eq!(vec!["./test/data/random_directory/file1.txt"], index.search("word2").unwrap());
        assert_eq!(vec!["./test/data/random_directory/file1.txt"], index.search("word3").unwrap());
        assert_eq!(None, index.search("word4"));
        assert_eq!(None, index.search("word5"));
    }

    #[test]
    fn several_matches() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/several_matches_directory");
        let expected = vec!["./test/data/several_matches_directory/file1.txt"];
        assert_eq!(expected, index.search("word2").unwrap());
        let expected = vec!["./test/data/several_matches_directory/file1.txt", "./test/data/several_matches_directory/file2.txt"];
        assert_eq!(index.search("word1").unwrap(), expected);
        assert_eq!(index.search("word3").unwrap(), expected);
    }

    #[test]
    fn multi_keywords_search() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/several_matches_directory");
        let expected = vec!["./test/data/several_matches_directory/file1.txt"];
        assert_eq!(expected, index.search("word1 word2").unwrap());
    }

    #[test]
    fn clean_keywords_before_search() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/simple_directory");
        let expected = vec!["./test/data/simple_directory/file1.txt"];
        assert_eq!(index.search("(word1) Word2, word3?").unwrap(), expected);
    }

    #[test]
    fn restore_index() {
        let index = FsIndex::restore("./test/data/simple_dump.json");
        let expected = vec!["./test/data/simple_directory/file1.txt"];
        assert_eq!(index.search("(word1) Word2, word3?").unwrap(), expected);
    }

    #[test]
    #[should_panic(expected="Unable to read dump file ./test/data/image_file.png")]
    fn restore_wrong_file() {
        FsIndex::restore("./test/data/image_file.png");
    }

    #[test]
    #[should_panic(expected="File not found ./test/data/foobar")]
    fn restore_unknown_file() {
        FsIndex::restore("./test/data/foobar");
    }

    #[test]
    fn dump_index() {
        let mut index = FsIndex::new(0.1);
        index.ingest("./test/data/simple_content.txt");
        let mut dest_file = std::env::temp_dir();
        dest_file.push("bloom_dump.json");
        index.dump(dest_file.as_path().to_str().unwrap());
        let expected = "{\"error_rate\":0.1,\"bloom_filters\":{\"./test/data/simple_content.txt\":{\"key_size\":4,\"bitfield\":[248,242,8],\"bitfield_size\":20}}}\n";
        let actual = fs::read_to_string(&dest_file).unwrap();
        assert_eq!(actual, expected);
        fs::remove_file(dest_file).unwrap();
    }

    #[test]
    #[should_panic(expected="Impossible to create dump file ./test/data")]
    fn dump_in_directory() {
        let mut index = FsIndex::new(0.01);
        index.ingest("./test/data/simple_content.txt");
        index.dump("./test/data");
    }
}

