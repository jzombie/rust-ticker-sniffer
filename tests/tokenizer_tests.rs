use ticker_sniffer::Tokenizer;

#[cfg(test)]
mod text_doc_tokenizer_tests {
    use super::*;

    #[test]
    fn test_non_case_sensitive() {
        let tokenizer = Tokenizer::text_doc_parser(false);

        let text = "these are all lowercase tokens";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["LOWERCASE", "TOKENS"]);
    }

    #[test]
    fn test_case_sensitive() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "these are all lowercase tokens";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_tokenize_ignores_stop_words() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "This Is A Stop Word Laden Phrase With Microsoft, Google, and Apple";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(
            tokens,
            vec![
                "STOP",
                "WORD",
                "LADEN",
                "PHRASE",
                "MICROSOFT",
                "GOOGLE",
                "APPLE"
            ]
        );
    }

    #[test]
    fn test_tokenize_with_single_spaces() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "This is a test string that Amazon, Meta, and Tesla did not generate";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["AMAZON", "META", "TESLA"]);
    }

    #[test]
    fn test_tokenize_with_multiple_spaces() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text =
            "This    is   a    test   string that       Netflix    and Nvidia     did not generate";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["NETFLIX", "NVIDIA"]);
    }

    #[test]
    fn test_tokenize_with_tabs() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "This\tis\ta\ttest\tstring\tthat\tFord\tand\tDisney\tdid\tnot\tgenerate";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["FORD", "DISNEY"]);
    }

    #[test]
    fn test_tokenize_with_line_breaks() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "This\nis\na\ntest\nstring\nwith\nApple\nand\nMicrosoft";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["APPLE", "MICROSOFT"]);
    }

    #[test]
    fn test_tokenize_with_mixed_whitespace() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "This  is\n   a test\tstring\n\nwith   mixed   whitespace \n\tGoogle...\n\n\t and Facebook did not generate.";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["GOOGLE", "FACEBOOK"]);
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, Vec::<&str>::new());
    }

    #[test]
    fn test_tokenize_with_newline_and_tabs() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "This\n\tis\t\na  test\n\t  string\n\tthat\n\tBoeing\n\tand\n\tCaterpillar\n\tdid not generate";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["BOEING", "CATERPILLAR"]);
    }

    #[test]
    fn test_tokenize_with_varied_company_names() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "Amazon, Meta, Tesla, IBM, Oracle, Adobe, Shopify, and Salesforce";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(
            tokens,
            vec![
                "AMAZON",
                "META",
                "TESLA",
                "IBM",
                "ORACLE",
                "ADOBE",
                "SHOPIFY",
                "SALESFORCE"
            ]
        );
    }

    #[test]
    fn test_tokenize_with_punctuation() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "Alphabet, Inc. (Google), Tesla's products, and Amazon's dominance.";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["ALPHABET", "INC", "GOOGLE", "TESLA", "AMAZON"]);
    }

    #[test]
    fn test_tokenize_maintining_mid_word_symbols() {
        let tokenizer = Tokenizer::text_doc_parser(true);

        let text = "BRK.A and BRK-B are both valid!";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["BRKA", "BRKB"]);
    }
}

#[cfg(test)]
mod ticker_symbol_tokenizer_tests {
    use super::*;

    #[test]
    fn test_ticker_tokenize_with_uppercase_symbols() {
        let tokenizer = Tokenizer::ticker_symbol_parser();

        let text = "AAPL MSFT TSLA";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["AAPL", "MSFT", "TSLA"]);
    }

    #[test]
    fn test_ticker_tokenize_ignores_lowercase_words() {
        let tokenizer = Tokenizer::ticker_symbol_parser();

        let text = "aapl msft tsla";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, Vec::<&str>::new());
    }

    #[test]
    fn test_ticker_tokenize_with_mixed_case() {
        let tokenizer = Tokenizer::ticker_symbol_parser();

        let text = "AAPL msft TSLA Goog";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["AAPL", "TSLA"]);
    }

    #[test]
    fn test_ticker_tokenize_with_numbers() {
        let tokenizer = Tokenizer::ticker_symbol_parser();

        let text = "BRK.A BRK-B 12345";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["BRKA", "BRKB"]);
    }

    #[test]
    fn test_ticker_tokenize_with_stop_words() {
        let tokenizer = Tokenizer::ticker_symbol_parser();

        let text = "The AAPL and MSFT stocks are rising.";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["AAPL", "MSFT"]);
    }

    #[test]
    fn test_ticker_tokenize_with_punctuation() {
        let tokenizer = Tokenizer::ticker_symbol_parser();

        let text = "AAPL, MSFT; TSLA. BRK-A!";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, vec!["AAPL", "MSFT", "TSLA", "BRKA"]);
    }

    #[test]
    fn test_ticker_tokenize_empty_string() {
        let tokenizer = Tokenizer::ticker_symbol_parser();

        let text = "";
        let tokens = tokenizer.tokenize(text);
        assert_eq!(tokens, Vec::<&str>::new());
    }
}
