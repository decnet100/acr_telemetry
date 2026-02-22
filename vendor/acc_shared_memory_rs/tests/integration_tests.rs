use acc_shared_memory_rs::{ACCSharedMemory, ACCError};

#[test]
fn test_library_initialization() {
    // This test will fail if ACC is not running, which is expected
    match ACCSharedMemory::new() {
        Ok(_) => {
            // ACC is running - great!
            println!("ACC is running, shared memory available");
        }
        Err(ACCError::SharedMemoryNotAvailable) => {
            // Expected when ACC is not running
            println!("ACC is not running (expected in CI/testing)");
        }
        Err(e) => {
            panic!("Unexpected error initializing ACC shared memory: {}", e);
        }
    }
}

#[test]
fn test_enum_conversions() {
    use acc_shared_memory_rs::enums::*;

    // Test AccStatus
    assert_eq!(AccStatus::try_from(0).unwrap(), AccStatus::Off);
    assert_eq!(AccStatus::try_from(1).unwrap(), AccStatus::Replay);
    assert_eq!(AccStatus::try_from(2).unwrap(), AccStatus::Live);
    assert_eq!(AccStatus::try_from(3).unwrap(), AccStatus::Pause);
    assert!(AccStatus::try_from(999).is_err());

    // Test AccSessionType
    assert_eq!(AccSessionType::try_from(0).unwrap(), AccSessionType::Practice);
    assert_eq!(AccSessionType::try_from(1).unwrap(), AccSessionType::Qualifying);
    assert_eq!(AccSessionType::try_from(2).unwrap(), AccSessionType::Race);

    // Test AccPenaltyType (with fallback)
    assert_eq!(AccPenaltyType::try_from(0).unwrap(), AccPenaltyType::None);
    assert_eq!(AccPenaltyType::try_from(1).unwrap(), AccPenaltyType::DriveThoughCutting);
    assert_eq!(AccPenaltyType::try_from(999).unwrap(), AccPenaltyType::Unknown); // Fallback

    // Test AccRainIntensity
    assert_eq!(AccRainIntensity::try_from(0).unwrap(), AccRainIntensity::NoRain);
    assert_eq!(AccRainIntensity::try_from(3).unwrap(), AccRainIntensity::MediumRain);
    assert!(AccRainIntensity::try_from(999).is_err());
}

#[test]
fn test_data_types() {
    use acc_shared_memory_rs::datatypes::*;

    // Test Vector3f
    let v = Vector3f::new(1.0, 2.0, 3.0);
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
    assert!((v.magnitude() - 3.741_657_5).abs() < 0.001);

    let v2 = Vector3f::from([4.0, 5.0, 6.0]);
    assert_eq!(v.dot(&v2), 32.0);

    // Test Wheels
    let wheels = Wheels::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(wheels.average(), 2.5);
    assert_eq!(wheels.front_average(), 1.5);
    assert_eq!(wheels.rear_average(), 3.5);

    // Test CarDamage
    let damage = CarDamage::new(0.1, 0.2, 0.3, 0.4, 0.5);
    assert_eq!(damage.total_damage(), 1.5);
    assert_eq!(damage.max_damage(), 0.5);
    assert!(damage.has_damage());

    let no_damage = CarDamage::none();
    assert!(!no_damage.has_damage());

    // Test ContactPoint
    let points = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0], [10.0, 11.0, 12.0]];
    let contact = ContactPoint::from_nested_array(points);
    assert_eq!(contact.front_left.x, 1.0);
    assert_eq!(contact.rear_right.z, 12.0);
}

#[test]
fn test_enum_methods() {
    use acc_shared_memory_rs::enums::*;

    // Test AccStatus methods
    assert!(AccStatus::Live.is_active());
    assert!(AccStatus::Replay.is_active());
    assert!(!AccStatus::Off.is_active());
    assert!(!AccStatus::Pause.is_active());

    assert!(AccStatus::Live.is_live());
    assert!(!AccStatus::Replay.is_live());

    // Test AccRainIntensity methods
    assert!(!AccRainIntensity::NoRain.is_wet());
    assert!(AccRainIntensity::LightRain.is_wet());
    assert!(AccRainIntensity::HeavyRain.requires_wet_tyres());
    assert!(!AccRainIntensity::Drizzle.requires_wet_tyres());

    // Test grip levels
    assert_eq!(AccRainIntensity::NoRain.grip_level(), 1.0);
    assert!(AccRainIntensity::HeavyRain.grip_level() < 0.5);

    // Test AccTrackGripStatus methods
    assert!(!AccTrackGripStatus::Optimum.is_wet());
    assert!(AccTrackGripStatus::Wet.is_wet());
    assert!(AccTrackGripStatus::Green.is_slippery());
    assert!(!AccTrackGripStatus::Optimum.is_slippery());

    // Test AccPenaltyType methods
    assert!(AccPenaltyType::DisqualifiedCutting.is_disqualification());
    assert!(!AccPenaltyType::DriveThoughCutting.is_disqualification());
    assert!(AccPenaltyType::StopAndGo10Cutting.is_cutting_penalty());
    assert!(AccPenaltyType::StopAndGo20PitSpeeding.is_pit_speeding_penalty());

    // Test AccFlagType methods
    assert!(AccFlagType::YellowFlag.requires_caution());
    assert!(!AccFlagType::GreenFlag.requires_caution());
    assert!(AccFlagType::BlueFlag.is_racing_flag());
    assert!(!AccFlagType::PenaltyFlag.is_racing_flag());
}

#[cfg(feature = "serde")]
#[test]
fn test_serde_serialization() {
    use acc_shared_memory_rs::datatypes::*;
    use acc_shared_memory_rs::enums::*;

    // Test Vector3f serialization
    let v = Vector3f::new(1.0, 2.0, 3.0);
    let json = serde_json::to_string(&v).unwrap();
    let v2: Vector3f = serde_json::from_str(&json).unwrap();
    assert_eq!(v, v2);

    // Test enum serialization
    let status = AccStatus::Live;
    let json = serde_json::to_string(&status).unwrap();
    let status2: AccStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(status, status2);

    // Test Wheels serialization
    let wheels = Wheels::new(1.0, 2.0, 3.0, 4.0);
    let json = serde_json::to_string(&wheels).unwrap();
    let wheels2: Wheels = serde_json::from_str(&json).unwrap();
    assert_eq!(wheels, wheels2);
}