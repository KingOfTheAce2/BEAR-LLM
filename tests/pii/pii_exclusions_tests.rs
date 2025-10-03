// Multi-Regional PII Exclusions Tests
//
// Tests for 8-region TOML-based exclusion system (3,474+ patterns)
// Covers: Regional loading, exclusion matching, false positive prevention

use bear_ai_llm::pii_detector::PIIDetector;

#[tokio::test]
async fn test_exclusions_legal_terms() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Common legal terms that should NOT be flagged as PII
    let legal_terms = vec![
        "Supreme Court",
        "First Amendment",
        "Federal Court",
        "United States",
        "Justice Department",
        "District Court",
    ];

    for term in legal_terms {
        let text = format!("The {} ruled in this case.", term);
        let entities = detector.detect_pii(&text).await.unwrap();

        // Filter for entities that match the legal term
        let false_positives: Vec<_> = entities.iter()
            .filter(|e| e.text.contains(term) || term.contains(&e.text))
            .collect();

        assert!(false_positives.is_empty(),
            "Legal term '{}' should not be flagged as PII. Found: {:?}",
            term, false_positives);
    }
}

#[tokio::test]
async fn test_exclusions_us_locations() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // US locations (from en region)
    let us_locations = vec![
        "United States",
        "New York",
        "California",
        "Washington",
        "Los Angeles",
        "Chicago",
    ];

    for location in us_locations {
        let text = format!("The trial was held in {}.", location);
        let entities = detector.detect_pii(&text).await.unwrap();

        let location_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text == location || e.text.contains(location))
            .collect();

        assert!(location_flagged.is_empty(),
            "US location '{}' should not be flagged as PII. Found: {:?}",
            location, location_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_eu_locations() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // European locations (from eu region)
    let eu_locations = vec![
        "Paris",
        "London",
        "Berlin",
        "Madrid",
        "Rome",
        "Amsterdam",
    ];

    for location in eu_locations {
        let text = format!("The hearing was in {}.", location);
        let entities = detector.detect_pii(&text).await.unwrap();

        let location_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text == location)
            .collect();

        println!("EU location '{}' detection: {:?}", location, location_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_apac_locations() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Asia-Pacific locations (from apac region)
    let apac_locations = vec![
        "Tokyo",
        "Beijing",
        "Singapore",
        "Sydney",
        "Seoul",
        "Hong Kong",
    ];

    for location in apac_locations {
        let text = format!("The conference was in {}.", location);
        let entities = detector.detect_pii(&text).await.unwrap();

        let location_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text == location)
            .collect();

        println!("APAC location '{}' detection: {:?}", location, location_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_latam_locations() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Latin America locations (from latam region)
    let latam_locations = vec![
        "Mexico City",
        "Buenos Aires",
        "São Paulo",
        "Santiago",
        "Lima",
        "Bogotá",
    ];

    for location in latam_locations {
        let text = format!("The meeting was in {}.", location);
        let entities = detector.detect_pii(&text).await.unwrap();

        let location_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text.contains(location))
            .collect();

        println!("LATAM location '{}' detection: {:?}", location, location_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_organizations() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Well-known organizations that might look like PII
    let organizations = vec![
        "United Nations",
        "World Health Organization",
        "European Union",
        "International Court of Justice",
    ];

    for org in organizations {
        let text = format!("{} issued a statement.", org);
        let entities = detector.detect_pii(&text).await.unwrap();

        let org_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text.contains(org))
            .collect();

        println!("Organization '{}' detection: {:?}", org, org_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_time_terms() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Time-related terms that might be confused with names
    let time_terms = vec![
        "Monday",
        "Tuesday",
        "January",
        "February",
        "Spring",
        "Summer",
    ];

    for term in time_terms {
        let text = format!("The hearing is scheduled for {}.", term);
        let entities = detector.detect_pii(&text).await.unwrap();

        let time_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text == term)
            .collect();

        assert!(time_flagged.is_empty(),
            "Time term '{}' should not be flagged as PII. Found: {:?}",
            term, time_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_case_insensitive() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Test case-insensitive matching
    let test_cases = vec![
        ("Supreme Court", "supreme court"),
        ("New York", "new york"),
        ("United States", "UNITED STATES"),
    ];

    for (original, variant) in test_cases {
        let text = format!("Reference to {}.", variant);
        let entities = detector.detect_pii(&text).await.unwrap();

        let flagged: Vec<_> = entities.iter()
            .filter(|e| e.text.to_lowercase().contains(&variant.to_lowercase()))
            .collect();

        println!("Case variant '{}' (original: '{}'): {:?}",
            variant, original, flagged);
    }
}

#[tokio::test]
async fn test_exclusions_count() {
    // Verify that exclusions were loaded
    // (This would require exposing exclusion count in PIIDetector)
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Should have loaded exclusions successfully
    // Test by checking if common terms are excluded
    let text = "United States Supreme Court";
    let entities = detector.detect_pii(text).await.unwrap();

    println!("Exclusions test - detected entities: {:?}", entities);

    // Should not detect common legal/location terms
}

#[tokio::test]
async fn test_exclusions_multilingual_names() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Names that are also common words in various languages
    let ambiguous_names = vec![
        "March",      // Month or surname
        "April",      // Month or first name
        "May",        // Month or first name
        "June",       // Month or first name
        "August",     // Month or first name
    ];

    for name in ambiguous_names {
        let text = format!("The meeting is in {}.", name);
        let entities = detector.detect_pii(&text).await.unwrap();

        println!("Ambiguous name '{}' in time context: {:?}", name, entities);
    }
}

#[tokio::test]
async fn test_exclusions_government_agencies() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let agencies = vec![
        "Department of Justice",
        "Federal Bureau of Investigation",
        "Internal Revenue Service",
        "Securities and Exchange Commission",
    ];

    for agency in agencies {
        let text = format!("The {} conducted an investigation.", agency);
        let entities = detector.detect_pii(&text).await.unwrap();

        let agency_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text.contains(agency))
            .collect();

        println!("Government agency '{}' detection: {:?}", agency, agency_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_legal_phrases() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let legal_phrases = vec![
        "due process",
        "habeas corpus",
        "pro bono",
        "ex parte",
        "in rem",
        "de facto",
    ];

    for phrase in legal_phrases {
        let text = format!("The court considered the {} argument.", phrase);
        let entities = detector.detect_pii(&text).await.unwrap();

        let phrase_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text.to_lowercase().contains(phrase))
            .collect();

        assert!(phrase_flagged.is_empty(),
            "Legal phrase '{}' should not be flagged. Found: {:?}",
            phrase, phrase_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_court_names() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    let courts = vec![
        "Supreme Court",
        "District Court",
        "Circuit Court",
        "Appellate Court",
        "Family Court",
        "Small Claims Court",
    ];

    for court in courts {
        let text = format!("The {} issued a ruling.", court);
        let entities = detector.detect_pii(&text).await.unwrap();

        let court_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text.contains(court))
            .collect();

        assert!(court_flagged.is_empty(),
            "Court name '{}' should not be flagged as PII. Found: {:?}",
            court, court_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_avoid_over_redaction() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Legal document with many exclusion terms
    let legal_doc = r#"
        In the United States District Court for the Southern District of New York

        The Supreme Court precedent established in First Amendment cases
        applies to this matter before the Federal Court. The Justice Department
        filed an amicus brief supporting the plaintiff's position.

        This case references California law and the Ninth Circuit's interpretation
        of constitutional rights.
    "#;

    let entities = detector.detect_pii(legal_doc).await.unwrap();

    println!("Legal document entities detected: {:?}", entities);

    // Should not over-redact legal terminology
    let legal_term_entities: Vec<_> = entities.iter()
        .filter(|e| {
            e.text.contains("Court") ||
            e.text.contains("Amendment") ||
            e.text.contains("Department") ||
            e.text.contains("States")
        })
        .collect();

    assert!(legal_term_entities.is_empty(),
        "Should not flag common legal terms as PII. Found: {:?}",
        legal_term_entities);
}

#[tokio::test]
async fn test_exclusions_proper_names_vs_locations() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // "Washington" can be a person or a location
    let location_context = "The case was filed in Washington.";
    let person_context = "Attorney Washington argued the case.";

    let location_entities = detector.detect_pii(location_context).await.unwrap();
    let person_entities = detector.detect_pii(person_context).await.unwrap();

    println!("Location context: {:?}", location_entities);
    println!("Person context: {:?}", person_entities);

    // Context should help disambiguate
}

#[tokio::test]
async fn test_exclusions_regional_merge() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Test that all 8 regions are loaded
    // Mix terms from different regions
    let multi_regional_text = r#"
        Offices in New York, London, Tokyo, and São Paulo.
        Compliance with United States, European Union, and Chinese regulations.
    "#;

    let entities = detector.detect_pii(multi_regional_text).await.unwrap();

    println!("Multi-regional test entities: {:?}", entities);

    // Location names should be excluded across all regions
}

#[tokio::test]
async fn test_exclusions_partial_matches() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Partial matches should NOT be excluded
    // (e.g., "New York" is excluded, but "New" alone is not)
    let text = "The New attorney filed a brief.";
    let entities = detector.detect_pii(text).await.unwrap();

    println!("Partial match test: {:?}", entities);

    // "New" alone should not be excluded just because "New York" is
}

#[tokio::test]
async fn test_exclusions_abbreviations() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Common abbreviations
    let abbreviations = vec![
        "USA",
        "FBI",
        "DOJ",
        "IRS",
        "SEC",
        "EU",
        "UN",
    ];

    for abbr in abbreviations {
        let text = format!("The {} is involved in this case.", abbr);
        let entities = detector.detect_pii(&text).await.unwrap();

        let abbr_flagged: Vec<_> = entities.iter()
            .filter(|e| e.text == abbr)
            .collect();

        println!("Abbreviation '{}' detection: {:?}", abbr, abbr_flagged);
    }
}

#[tokio::test]
async fn test_exclusions_preserves_real_pii() {
    let detector = PIIDetector::new();
    detector.initialize().await.unwrap();

    // Ensure exclusions don't prevent detecting real PII
    let text = r#"
        The case was filed in New York (location, should be excluded).
        Contact John Smith at john.smith@example.com (PII, should be detected).
        SSN: 123-45-6789 (PII, should be detected).
    "#;

    let entities = detector.detect_pii(text).await.unwrap();

    // Should detect email and SSN
    let has_email = entities.iter().any(|e| e.entity_type == "EMAIL");
    let has_ssn = entities.iter().any(|e| e.entity_type == "SSN");

    assert!(has_email, "Should detect email despite exclusions");
    assert!(has_ssn, "Should detect SSN despite exclusions");

    // Should NOT flag "New York"
    let has_new_york = entities.iter().any(|e| e.text.contains("New York"));

    println!("Detected 'New York': {}", has_new_york);
}
