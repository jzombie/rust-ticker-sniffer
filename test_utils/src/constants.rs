use std::path::PathBuf;
use std::sync::LazyLock;

macro_rules! define_path {
    ($($segment:expr),*) => {{
        PathBuf::from_iter(vec![$($segment),*])
    }};
}

// Lazy-initialized static variables
pub static TEST_FILES_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| define_path!("tests", "test_files"));

pub static TEST_RUNNER_VALIDATION_FILES_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| define_path!("tests", "test_runner_validation_files"));

pub static COMPANY_SYMBOL_FILE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| define_path!("data", "company_symbol_list.csv"));
