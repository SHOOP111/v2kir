use aetherfall::model::RiftEvent;
use std::collections::HashSet;

#[test]
fn every_rift_event_has_a_unique_title_and_description() {
    let titles: HashSet<_> = RiftEvent::ALL.iter().map(|event| event.title()).collect();
    assert_eq!(titles.len(), RiftEvent::ALL.len());

    for event in RiftEvent::ALL {
        assert!(!event.title().trim().is_empty());
        assert!(!event.description().trim().is_empty());
    }
}

#[test]
fn rift_event_catalog_contains_all_four_combat_modifiers() {
    assert!(RiftEvent::ALL.contains(&RiftEvent::BloodMoon));
    assert!(RiftEvent::ALL.contains(&RiftEvent::Overcharge));
    assert!(RiftEvent::ALL.contains(&RiftEvent::TimeSnare));
    assert!(RiftEvent::ALL.contains(&RiftEvent::FortuneFlux));
}
