use crate::models::CompanyTokenProcessorConfig;

pub const DEFAULT_COMPANY_TOKEN_PROCESSOR_CONFIG: &CompanyTokenProcessorConfig =
    &CompanyTokenProcessorConfig {
        threshold_ratio_exact_matches: 0.50,
        threshold_min_company_token_coverage: 0.60,
    };
