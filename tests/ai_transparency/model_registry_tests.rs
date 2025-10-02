use bear_ai_llm::ai_transparency::ModelRegistry;
use tempfile::tempdir;

#[test]
fn test_resolve_llama_models() {
    let registry = ModelRegistry::new();

    // Test Llama 2 models
    assert_eq!(
        registry.resolve_model_id("llama-2-7b-chat.Q4_K_M.gguf"),
        Some("meta-llama/Llama-2-7b-chat-hf".to_string())
    );

    assert_eq!(
        registry.resolve_model_id("llama-2-13b-chat.Q5_K_S.gguf"),
        Some("meta-llama/Llama-2-13b-chat-hf".to_string())
    );

    // Test Llama 3 models
    assert_eq!(
        registry.resolve_model_id("llama-3-8b-instruct.Q4_K_M.gguf"),
        Some("meta-llama/Meta-Llama-3-8B-Instruct".to_string())
    );
}

#[test]
fn test_resolve_mistral_models() {
    let registry = ModelRegistry::new();

    assert_eq!(
        registry.resolve_model_id("mistral-7b-instruct-v0.2.Q4_K_M.gguf"),
        Some("mistralai/Mistral-7B-Instruct-v0.2".to_string())
    );

    assert_eq!(
        registry.resolve_model_id("mixtral-8x7b-instruct.Q5_K_S.gguf"),
        Some("mistralai/Mixtral-8x7B-Instruct-v0.1".to_string())
    );
}

#[test]
fn test_resolve_phi_models() {
    let registry = ModelRegistry::new();

    assert_eq!(
        registry.resolve_model_id("phi-2.Q4_K_M.gguf"),
        Some("microsoft/phi-2".to_string())
    );

    assert_eq!(
        registry.resolve_model_id("phi-3-mini.Q4_K_M.gguf"),
        Some("microsoft/Phi-3-mini-4k-instruct".to_string())
    );
}

#[test]
fn test_resolve_gemma_models() {
    let registry = ModelRegistry::new();

    assert_eq!(
        registry.resolve_model_id("gemma-2b.Q4_K_M.gguf"),
        Some("google/gemma-2b".to_string())
    );

    assert_eq!(
        registry.resolve_model_id("gemma-7b.Q4_K_M.gguf"),
        Some("google/gemma-7b".to_string())
    );
}

#[test]
fn test_custom_mappings() {
    let mut registry = ModelRegistry::new();

    registry.add_custom_mapping(
        "my-custom-model.gguf".to_string(),
        "user/custom-model".to_string()
    );

    assert_eq!(
        registry.resolve_model_id("my-custom-model.gguf"),
        Some("user/custom-model".to_string())
    );

    // Custom mapping should override built-in
    registry.add_custom_mapping(
        "llama-2-7b-chat.Q4_K_M.gguf".to_string(),
        "user/my-llama".to_string()
    );

    assert_eq!(
        registry.resolve_model_id("llama-2-7b-chat.Q4_K_M.gguf"),
        Some("user/my-llama".to_string())
    );
}

#[test]
fn test_remove_custom_mapping() {
    let mut registry = ModelRegistry::new();

    registry.add_custom_mapping(
        "test-model.gguf".to_string(),
        "user/test".to_string()
    );

    assert!(registry.remove_custom_mapping("test-model.gguf"));
    assert_eq!(registry.resolve_model_id("test-model.gguf"), None);

    // Removing non-existent mapping should return false
    assert!(!registry.remove_custom_mapping("non-existent.gguf"));
}

#[test]
fn test_extract_model_name() {
    let registry = ModelRegistry::new();

    assert_eq!(
        registry.extract_model_name("llama-2-7b-chat.Q4_K_M.gguf"),
        "Llama 2 7b Chat"
    );

    assert_eq!(
        registry.extract_model_name("mistral_7b_instruct.gguf"),
        "Mistral 7b Instruct"
    );

    assert_eq!(
        registry.extract_model_name("phi-3-mini.gguf"),
        "Phi 3 Mini"
    );
}

#[test]
fn test_unknown_model() {
    let registry = ModelRegistry::new();

    assert_eq!(
        registry.resolve_model_id("completely-unknown-model.gguf"),
        None
    );
}

#[test]
fn test_save_and_load_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Create registry with custom mapping
    let mut registry1 = ModelRegistry::new();
    registry1.add_custom_mapping(
        "test1.gguf".to_string(),
        "user/test1".to_string()
    );
    registry1.add_custom_mapping(
        "test2.gguf".to_string(),
        "user/test2".to_string()
    );

    // Save to file
    registry1.save_to_file(config_path.clone()).unwrap();

    // Load from file
    let registry2 = ModelRegistry::load_from_file(config_path).unwrap();

    assert_eq!(
        registry2.resolve_model_id("test1.gguf"),
        Some("user/test1".to_string())
    );
    assert_eq!(
        registry2.resolve_model_id("test2.gguf"),
        Some("user/test2".to_string())
    );
}

#[test]
fn test_get_all_mappings() {
    let mut registry = ModelRegistry::new();

    registry.add_custom_mapping(
        "custom1.gguf".to_string(),
        "user/custom1".to_string()
    );

    let mappings = registry.get_all_mappings();

    // Should include both built-in and custom mappings
    assert!(mappings.len() > 1);

    // Check custom mapping is included
    assert!(mappings.iter().any(|(_, id)| id == "user/custom1"));

    // Check built-in mappings are included
    assert!(mappings.iter().any(|(_, id)| id.contains("llama")));
}

#[test]
fn test_filename_normalization() {
    let registry = ModelRegistry::new();

    // Test with underscores vs hyphens
    assert_eq!(
        registry.resolve_model_id("llama_2_7b_chat.Q4_K_M.gguf"),
        Some("meta-llama/Llama-2-7b-chat-hf".to_string())
    );

    // Test with .bin extension
    assert_eq!(
        registry.resolve_model_id("llama-2-7b-chat.Q4_K_M.bin"),
        Some("meta-llama/Llama-2-7b-chat-hf".to_string())
    );

    // Test case insensitivity
    assert_eq!(
        registry.resolve_model_id("LLAMA-2-7B-CHAT.Q4_K_M.GGUF"),
        Some("meta-llama/Llama-2-7b-chat-hf".to_string())
    );
}
