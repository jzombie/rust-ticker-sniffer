fn main() {
    // This will call the build-utils crate for custom build-time processing
    if let Err(e) = ticker_sniffer_build_utils::run_build_utils() {
        eprintln!("Error running build utils: {}", e);
        std::process::exit(1);
    }
}
