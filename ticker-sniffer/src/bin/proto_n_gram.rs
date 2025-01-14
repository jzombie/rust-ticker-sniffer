#![allow(dead_code, unused_imports, unused_variables)]

#[path = "../../test_utils/lib.rs"]
use ticker_sniffer::extract_tickers_from_text;

use log::{debug, error, info, log_enabled, Level};

use std::collections::HashMap;

fn main() {
    env_logger::init();

    // let mut matches = HashSet::new();

    // TODO: Commit to tokenizer tests; expect: Tokens: ["WELL", "IPHONE", "DEVELOPMENT", "EBAY", "DEVELOPMENT", "WALMART", "WALMARTS"]
    // let tokens = tokenize(
    //     &"Well okay iPhone turtle develoPment e-Bay  Deve-\nlopment Wal-mart's at it again!",
    // );

    // TODO: Commit to tokenizer tests; expect: Tokens: ["ECOMMERCE", "AMAZONCOM", "AMAZON", "INC", "AMZN", "QUICK", "QUOTEAMZN", "FREE", "REPORT", "DOW", "JONES", "INDUSTRIAL", "AVERAGE", "WALGREENS", "BOOTS", "ALLIANCE", "WBA", "QUICK", "QUOTEWBA", "FREE", "REPORT", "FEB", "THE", "THE", "AMAZON", "DOW"]
    // let tokens = tokenize(
    //     &"E-commerce giant Amazon.com Inc. (AMZN Quick QuoteAMZN - Free Report) joined the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator Walgreens Boots Alliance (WBA Quick QuoteWBA - Free Report) on Feb 26. The reshuffle reflects the ongoing shift in economic power from traditional brick-and-mortar retail to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks a significant milestone in the recognition of the e-commerce giant's influence and its role in the broader market.",
    // );

    // let query = "The stuff IT dreams are made of, but you know, Agilent is interesting.";

    // let query = "Ashford";
    // let query = "Power REIT";
    // let query = "REIT Hospitality Apple stuff";
    // let query = "Arbor";
    // let query = "Arbor Realty";
    // let query = "Berkshire Hathaway is not Apple, but owns Apple, of course, which is not Apple Hospitality REIT.";
    // let query = "Apple";

    // let query = "Apple";
    // let query = "Berkshire";

    // TODO: Ensure that queries can't be performed in reverse
    // let query = "Hathaway Berkshire Hospitality Hathaway Berkshire Hathaway Apple Apple Hospitality REIT INC REIT 222";
    // let query = "Apple Apple Hospitality REIT Apple INC";
    // let query = "Apple Apple INC Apple Hospitality REIT";
    // let query = "Berkshire Hathaway Berkshire Hathaway";
    // let query = "Hathaway";
    // let query = "Hospitality Apple";

    // let query = "Apple Walmart Hospitality REIT";
    // let query = "Alphabet";
    // let query = "Amazon";
    // let query = "SPDR Dow Jones Industrial Average";
    // let query = "Dow Jones Industrial, and well, Apple Computers.";
    // let query = "The article mentions Dow Chemical Company as one of the key players in the recycled plastic market, indicating their involvement and potential to benefit from the market growth.";

    // TODO: Locate source
    // TODO: [add test] Figure out why APLE is showing up instead of AAPL
    // TODO: [add test] Check for `EDOW` (First Trust Dow 30 Equal Weight ETF) in the results
    let query = r#"E-commerce giant Amazon.com Inc. (AMZN Quick QuoteAMZN - Free Report) joined the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator Walgreens Boots Alliance (WBA Quick QuoteWBA - Free Report) on Feb 26. The reshuffle reflects the ongoing shift in economic power from traditional brick-and-mortar retail to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks a significant milestone in the recognition of the e-commerce giant's influence and its role in the broader market.
        The shift was prompted by Walmart's (WMT Quick QuoteWMT - Free Report) decision to execute a 3-to-1 stock split, which has reduced its stock's weighting in the index. The Dow is a price-weighted index. So, stocks that fetch higher prices are given more weight. Amazon's addition has increased consumer retail exposure within the index, alongside enhancing the representation of various other business sectors that Amazon engages in, including cloud computing, digital streaming and artificial intelligence, among others (read: Walmart Soars on Earnings, Dividend & Vizio Deal: ETFs to Buy).
        Amazon took the 17th position in the index, while Walmart's weighting dropped to 26 from 17. UnitedHealth Group remained the most heavily weighted stock in the index. Amazon's entry into the Dow Jones is not just a symbolic change but a reflection of the evolving priorities and dynamics within the investment world. It signals a broader recognition of the value and impact of technology and e-commerce sectors, encouraging investors to perhaps rethink their investment approaches in light of these trends.

        While the Dow Jones is making new record highs, its performance is lagging behind the S&P and Nasdaq over the past year. The underperformance is due to the lack of exposure in tech stocks and the “Magnificent Seven” companies in particular. The Dow includes two of the Magnificent Seven — Apple (AAPL Quick QuoteAAPL - Free Report) and Microsoft (MSFT Quick QuoteMSFT - Free Report) . Amazon will be the third. As such, the addition of Amazon will help Dow Jones catch up with the S&P 500 gains. The shares of the commerce giant have surged more than 80% over the past year (read: ETFs to Tap on Amazon's Strong Q4 Earnings).

        Given this, investors seeking to tap the potential strength in the Dow Jones trend could consider SPDR Dow Jones Industrial Average ETF (DIA Quick QuoteDIA - Free Report) , iShares Dow Jones U.S. ETF (IYY Quick QuoteIYY - Free Report) , Invesco Dow Jones Industrial Average Dividend ETF (DJD Quick QuoteDJD - Free Report) and First Trust Dow 30 Equal Weight ETF (EDOW Quick QuoteEDOW - Free Report) .

        ETFs to Tap
        SPDR Dow Jones Industrial Average ETF (DIA Quick QuoteDIA - Free Report)
        SPDR Dow Jones Industrial Average ETF is one of the largest and most popular ETFs in the large-cap space, with AUM of $33.1 billion and an average daily volume of 3.8 million shares. It tracks the Dow Jones Industrial Average Index, holding 30 stocks in its basket with each making up for less than 9% share. Financials (21.7%), information technology (19.5%), healthcare (18.5%), consumer discretionary (15.9%) and industrials (14.613.7%) and are the top five sectors (read: Will Dow Jones ETFs Rule in 2024?).

        SPDR Dow Jones Industrial Average ETF charges 16 bps in annual fees and has a Zacks ETF Rank #1 (Strong Buy) with a Medium risk outlook.

        iShares Dow Jones U.S. ETF (IYY Quick QuoteIYY - Free Report)

        iShares Dow Jones U.S. ETF tracks the Dow Jones U.S. Index, holding 1077 stocks in its basket, with none accounting for more than 6.4% of the assets. Information technology takes the largest share at 29%, while financials, healthcare and consumer discretionary round off the next spots with double-digit exposure each.

        iShares Dow Jones U.S. ETF has amassed $1.9 billion in its asset base while trading in an average daily volume of 36,000 shares. It charges 20 bps in annual fees and has a Zacks ETF Rank #3 (Hold) with a Medium risk outlook.

        Invesco Dow Jones Industrial Average Dividend ETF (DJD Quick QuoteDJD - Free Report)

        Invesco Dow Jones Industrial Average Dividend ETF offers exposure to dividend-paying companies included in the Dow Jones Industrial Average by their 12-month dividend yield over the prior 12 months. It holds 27 stocks in its basket, with none accounting for more than 12% of the assets.

        Invesco Dow Jones Industrial Average Dividend ETF has been able to manage assets worth $294.4 million while trading in a volume of 56,000 shares a day on average. It charges 7 bps in annual fees and has a Zacks ETF Rank #3.

        First Trust Dow 30 Equal Weight ETF (EDOW Quick QuoteEDOW - Free Report)

        First Trust Dow 30 Equal Weight ETF offers equal-weight exposure to all the 30 components of the Dow Jones Industrial Average by tracking the Dow Jones Industrial Average Equal Weight Index.

        First Trust Dow 30 Equal Weight ETF has accumulated $249.1 million in its asset base and trades in an average daily volume of 58,000 shares. It charges 50 bps in annual fees.

        Want key ETF info delivered straight to your inbox?
    Zacks’ free Fund Newsletter will brief you on top news and analysis, as well as top-performing ETFs, each week.
        "#;

    let query =
        "A Google search and Apple Hospitality REIT, and then there is the REIT symbol and AAPL, now I will say it again, Apple is a good company, and they team up with Google occasionally though Apple and Google are competitors. A Google search confirms...";

    // let query = "NVDA, GOOG, GOOGL, A, AAPL and AMZN";

    // let query = "First Trust Dow 30 Equal Weight ETF";
    // Note: This is a subset of the previous, filtered to the lines which were are causing EDOW to not be represented in the result
    // let query = r#"

    //         While the Dow Jones is making new record highs, its performance is lagging behind the S&P and Nasdaq over the past year. The underperformance is due to the lack of exposure in tech stocks and the “Magnificent Seven” companies in particular. The Dow includes two of the Magnificent Seven — Apple (AAPL Quick QuoteAAPL - Free Report) and Microsoft (MSFT Quick QuoteMSFT - Free Report) . Amazon will be the third. As such, the addition of Amazon will help Dow Jones catch up with the S&P 500 gains. The shares of the commerce giant have surged more than 80% over the past year (read: ETFs to Tap on Amazon's Strong Q4 Earnings).

    //         Given this, investors seeking to tap the potential strength in the Dow Jones trend could consider SPDR Dow Jones Industrial Average ETF (DIA Quick QuoteDIA - Free Report) , iShares Dow Jones U.S. ETF (IYY Quick QuoteIYY - Free Report) , Invesco Dow Jones Industrial Average Dividend ETF (DJD Quick QuoteDJD - Free Report) and First Trust Dow 30 Equal Weight ETF (EDOW Quick QuoteEDOW - Free Report) .

    //         First Trust Dow 30 Equal Weight ETF (EDOW Quick QuoteEDOW - Free Report)

    //         First Trust Dow 30 Equal Weight ETF offers equal-weight exposure to all the 30 components of the Dow Jones Industrial Average by tracking the Dow Jones Industrial Average Equal Weight Index.

    //     First Trust Dow 30 Equal Weight ETF has accumulated $249.1 million in its asset base and trades in an average daily volume of 58,000 shares. It charges 50 bps in annual fees.

    //         Want key ETF info delivered straight to your inbox?
    //     Zacks’ free Fund Newsletter will brief you on top news and analysis, as well as top-performing ETFs, each week.
    // "#;

    // let query = "Dow Jones Industrial Average";
    // let query = "Berkshire Hathaway";

    // TODO: This includes a lot of repeated "Capital" entries, with only initial 0 window indexes.
    // This type of pattern should be filtered out so it effectively removes them entirely.
    // let query = r#"
    // Amazon
    // has for years counted on millions of third-party sellers to provide the bulk of the inventory that consumers buy. But keeping track of their finances has long been a challenge for outside merchants, particularly smaller mom-and-pop shops.

    // Amazon said Monday that it’s partnering with Intuit
    // to bring the software company’s online accounting tools to its vast network of sellers in mid-2025. Intuit QuickBooks will be available on Amazon Seller Central, the hub sellers use to manage their Amazon businesses, the companies said. Eligible sellers will also have access to loans through QuickBooks Capital.

    // “Together with Intuit, we’re working to equip our selling partners with additional financial tools and access to capital to help them scale efficiently,” Dharmesh Mehta, Amazon’s vice president of worldwide selling partner services, said in the joint release.

    // The companies said sellers will see a real-time view of the financial health of their business, getting a clear picture of profitability, cash flow and tax estimates.

    // While the Intuit integration isn’t expected to go live until the middle of next year, the announcement comes as sellers ramp up their businesses for the holiday season, the busiest time of the year for most retailers.

    // Representatives from both companies declined to provide specific terms of the agreement, including how revenue will be shared.

    // The marketplace is a critical part of Amazon’s retail strategy. In addition to accounting for about 60% of products sold, Amazon generates fees from providing fulfillment and shipping services as well as by offering customer support to sellers and charging them to advertise on the site.

    // In the third quarter, seller services revenue increased 10% to $37.9 billion, accounting for 24% of total revenue, a number that’s steadily increased in recent years. Amazon CEO Andy Jassy said on the earnings call that ”[third-party] demand is still strong and unit volumes are strong.”

    // Amazon shares are up almost 50% this year, climbing to a fresh record Friday, and topping the Nasdaq’s 31% gain for the year. Meanwhile, Intuit has underperformed the broader tech index, with its stock up less than 4% in 2024.

    // Intuit shares dropped 5% on Nov. 19 after The Washington Post reported that President-elect Donald Trump’s government efficiency team is considering creating a free tax-filing app. They fell almost 6% three days later after the company issued a revenue forecast for the current quarter that trailed analysts’ estimates due to some sales being delayed.

    // QuickBooks, which is particularly popular as an all-in-one accounting, expense management and payroll tool for small businesses, has been one of Intuit’s key drivers for growth. The company said in November that its QuickBooks Online Accounting segment expanded by 21% in the latest quarter, while total revenue increased 10% to $3.28 billion.

    // Intuit has been adding generative artificial intelligence tools into QuickBooks and other small business services, such as its Mailchimp email marketing offering, to provide more automated insights for users.

    // “You can imagine, as we look ahead, our goal is to create a done-for-you experience across the entire platform, across Mailchimp and QuickBooks and all of the services,” Intuit CEO Sasan Goodarzi said on the fiscal first-quarter earnings call.

    // Goodarzi said in Monday’s release that the company is bringing its “AI-driven expert platform to help sellers boost their revenue and profitability, save time, and grow with confidence.”
    // "#;

    // TODO: [add test; specifically for company name extractor] Ensure DIA gets extracted using company name
    // let query = r#"
    //     Given this, investors seeking to tap the potential strength in the Dow Jones trend could consider SPDR Dow Jones Industrial Average ETF (DIA Quick QuoteDIA - Free Report) , iShares Dow Jones U.S. ETF (IYY Quick QuoteIYY - Free Report) , Invesco Dow Jones Industrial Average Dividend ETF (DJD Quick QuoteDJD - Free Report) and First Trust Dow 30 Equal Weight ETF (EDOW Quick QuoteEDOW - Free Report) .
    // "#;

    // TODO: [add test] This should only return Apple
    // let query = "Apple Apple Inc Hospitality";

    // let query = "E-commerce giant Amazon.com Inc. (AMZN Quick QuoteAMZN - Free Report) joined the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator Walgreens Boots Alliance (WBA Quick QuoteWBA - Free Report) on Feb 26. The reshuffle reflects the ongoing shift in economic power from traditional brick-and-mortar retail to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks a significant milestone in the recognition of the e-commerce giant's influence and its role in the broader market.";

    // let query = r#"
    // Invesco Dow Jones Industrial Average Dividend ETF (DJD Quick QuoteDJD - Free Report)

    //     Invesco Dow Jones Industrial Average Dividend ETF offers exposure to dividend-paying companies included in the Dow Jones Industrial Average by their 12-month dividend yield over the prior 12 months. It holds 27 stocks in its basket, with none accounting for more than 12% of the assets.

    //     Invesco Dow Jones Industrial Average Dividend ETF has been able to manage assets worth $294.4 million while trading in a volume of 56,000 shares a day on average. It charges 7 bps in annual fees and has a Zacks ETF Rank #3.
    // "#;

    //     let query = r#"
    //     Palantir
    // : Cramer praised the software company’s management and said the company’s third quarter, which sent shares surging 20%, was one of the best of the year. He also said he liked the company’s defense business, noting Palantir’s work for the Pentagon.
    // Axon
    // : Axon largely makes equipment for law enforcement, and Cramer suggested it is poised to do well as a GOP trifecta in Washington likely means police will receive more funding. The company also reported better-than-expected quarterly results, and it has new software that includes artificial intelligence, he added.
    // Tesla
    // : Tesla CEO Elon Musk is a close ally of President-elect Donald Trump, and Cramer said his loyalty will probably lead to rewards for the company and its automated vehicle business.

    //     "#;

    // let query = " Invesco Dow Jones Industrial Average Dividend ETF ";

    // let query = r#"SPDR Dow Jones Industrial Average ETF Invesco Dow Jones Industrial Average Dividend ETF iShares Dow Jones U.S. ETF"#;

    // let query = "Apple Walmart Berkshire Hathaway Apple Wal-mart Berkshire Hathaway Dow Jones Industrial Average";

    // let query = "Apple Hospitality REIT";

    // let query = "E-commerce giant Amazon.com Inc. (AMZN Quick QuoteAMZN - Free Report) joined the blue-chip index, Dow Jones Industrial Average, replacing drugstore operator Walgreens Boots Alliance (WBA Quick QuoteWBA - Free Report) on Feb 26. The reshuffle reflects the ongoing shift in economic power from traditional brick-and-mortar retail to e-commerce and technology-driven companies. The inclusion of Amazon in the Dow marks a significant milestone in the recognition of the e-commerce giant's influence and its role in the broader market.";
    // let query = "The DOW Jones Industrial";
    // let query = "Agilent";
    // let query = "NVDA";
    // let query = "Google";
    // let query = "J.P. Morgan Exchange-Traded Fund Trust - JPMorgan BetaBuilders Emerging Markets Equity ETF";
    let query = r#"
    Intel
    on Thursday announced the appointment of two new directors with significant semiconductor manufacturing experience as the company ramps up the search process for ousted CEO Pat Gelsinger’s replacement.

    Former ASML
    CEO Eric Meurice and Microchip Technology
    interim CEO Steve Sanghi will join Intel’s board effective immediately, the company said. Their appointments mean that Intel’s board once again has directors with semiconductor experience, fixing a vacuum left by the departure of Cadence Design Systems Chairman Lip-Bu Tan in August.

    Intel declined to comment on what committees the two new directors would join and the nature of the search process. 

    Intel’s search for new directors predated Gelsinger’s firing, according to people familiar with the matter. The company’s board had been interviewing semiconductor executives for several weeks, said the people, who requested anonymity to discuss confidential information freely.

    Meurice ran ASML, which manufactures some of the most advanced chipmaking machines, for eight years. ASML’s share price quintupled during his tenure, Intel said.

    Sanghi rejoined Microchip as interim CEO in November, after serving as CEO from 1991 to 2016. Sanghi was previously an executive at Intel.

    “Eric and Steve are highly respected leaders in the semiconductor industry whose deep technical expertise, executive experience and operational rigor make them great additions to the Intel board,” interim Executive Chairman Frank Yeary said.

    Sanghi and Meurice join Intel’s board at a critical juncture. The company ousted Gelsinger over the weekend and has since been assembling a short list of replacements with the help of an executive search firm. Intel’s market cap sits firmly below $100 billion, and the company is still in the middle of an intense cost-cutting drive.

    Intel CFO David Zinsner and product chief MJ Holthaus currently serve as interim co-CEOs.
    "#;

    let query = "Eric";

    let query = "A";
    let query = "BRK-B";

    let results = extract_tickers_from_text(&query).unwrap();

    println!("Extracted Tickers:");
    for (ticker_symbol, frequency) in results {
        println!("{}: {:.2}", ticker_symbol, frequency);
    }
}
