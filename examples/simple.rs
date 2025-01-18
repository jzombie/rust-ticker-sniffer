use ticker_sniffer::extract_tickers_from_text;

fn main() {
    env_logger::init();

    let text = "Berkshire Hathaway and Apple";

    let results = extract_tickers_from_text(&text).unwrap();

    println!("Extracted Tickers for the given text \"{}\"", text);
    for (ticker_symbol, frequency) in results {
        println!("{}: {:.2}", ticker_symbol, frequency);
    }
}
