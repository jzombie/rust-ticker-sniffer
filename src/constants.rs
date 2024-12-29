use crate::models::DocumentCompanyNameExtractorConfig;

pub const DEFAULT_COMPANY_NAME_EXTRACTOR_CONFIG: DocumentCompanyNameExtractorConfig =
    DocumentCompanyNameExtractorConfig {
        min_text_doc_token_sim_threshold: 0.99,
        continuity_reward: 1.5,
        confidence_score_duplicate_threshold: 2,
        low_confidence_penalty_factor: 0.5,
        min_confidence_level_threshold: 0.5,
    };
