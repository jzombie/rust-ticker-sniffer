use ticker_sniffer::tokenize;

#[cfg(test)]
mod tokenizer_tests {
    use super::*;

    #[test]
    fn test_tokenize_with_single_spaces() {
        let text = "This is a test string";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test", "string"]);
    }

    #[test]
    fn test_tokenize_with_multiple_spaces() {
        let text = "This  is   a test    string";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test", "string"]);
    }

    #[test]
    fn test_tokenize_with_tabs() {
        let text = "This\tis\ta\ttest\tstring";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test", "string"]);
    }

    #[test]
    fn test_tokenize_with_line_breaks() {
        let text = "This\nis\na\ntest\nstring";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test", "string"]);
    }

    #[test]
    fn test_tokenize_with_mixed_whitespace() {
        let text = "This  is\n   a test\tstring\n\nwith   mixed   whitespace";
        let tokens = tokenize(text);
        assert_eq!(
            tokens,
            vec![
                "This",
                "is",
                "a",
                "test",
                "string",
                "with",
                "mixed",
                "whitespace"
            ]
        );
    }

    #[test]
    fn test_tokenize_empty_string() {
        let text = "";
        let tokens = tokenize(text);
        assert_eq!(tokens, Vec::<&str>::new());
    }

    #[test]
    fn test_tokenize_with_newline_and_tabs() {
        let text = "This\n\tis\t\na  test\n\t  string";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test", "string"]);
    }

    #[test]
    fn test_tokenize_removing_end_puncutation() {
        let text = "This\n\tis\t\na  test\n\t  string!!";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["This", "is", "a", "test", "string"]);
    }

    #[test]
    fn test_tokenize_maintining_mid_word_symbols() {
        let text = "BRK.A and BRK-B are both valid!";
        let tokens = tokenize(text);
        assert_eq!(
            tokens,
            vec!["BRK.A", "and", "BRK-B", "are", "both", "valid"]
        );
    }
}
