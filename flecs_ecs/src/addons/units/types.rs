use std::ops::Deref;
use std::ptr::addr_of;

use crate::core::*;
use crate::sys;
use flecs_ecs_sys::*; //for all the units statics

macro_rules! create_pre_registered_extern_component {
    ($struct_name:ident, $static_id:ident) => {
        create_pre_registered_extern_component!($struct_name, $static_id, "");
    };
    ($struct_name:ident, $static_id:ident, $doc:tt) => {
        #[derive(Debug, Default)]
        #[allow(clippy::empty_docs)]
        #[doc = $doc]
        pub struct $struct_name;

        impl From<$struct_name> for flecs_ecs::core::Entity {
            #[inline]
            fn from(_view: $struct_name) -> Self {
                flecs_ecs::core::Entity(unsafe { $static_id })
            }
        }

        impl Deref for $struct_name {
            type Target = u64;
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                unsafe { &*addr_of!($static_id) }
            }
        }

        impl PartialEq<u64> for $struct_name {
            #[inline]
            fn eq(&self, other: &u64) -> bool {
                unsafe { $static_id == *other }
            }
        }

        impl PartialEq<$struct_name> for u64 {
            #[inline]
            fn eq(&self, _other: &$struct_name) -> bool {
                *self == unsafe { $static_id }
            }
        }

        impl ComponentInfo for $struct_name {
            const IS_GENERIC: bool = false;
            const IS_ENUM: bool = false;
            const IS_TAG: bool = true;
            const IMPLS_CLONE: bool = false;
            const IMPLS_DEFAULT: bool = false;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
            type TagType =
                flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsATag;
        }

        impl TagComponent for $struct_name {}

        impl ComponentType<Struct> for $struct_name {}

        impl ComponentId for $struct_name {
            type UnderlyingType = $struct_name;
            type UnderlyingEnumType = NoneEnum;

            fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
            ) -> sys::ecs_entity_t {
                unsafe { $static_id }
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl WorldProvider<'a>,
                _name: &str,
            ) -> sys::ecs_entity_t {
                unsafe { $static_id }
            }

            fn is_registered_with_world<'a>(_: impl WorldProvider<'a>) -> bool {
                true
            }

            fn id<'a>(_world: impl WorldProvider<'a>) -> sys::ecs_id_t {
                unsafe { $static_id }
            }

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: std::sync::atomic::AtomicU32 =
                    std::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }
        }
    };
}

// Prefixes
create_pre_registered_extern_component!(Prefixes, EcsUnitPrefixes, "Parent scope for prefixes.");

// Unit counts
create_pre_registered_extern_component!(Yocto, EcsYocto, "Yocto unit prefix.");
create_pre_registered_extern_component!(Zepto, EcsZepto, "Zepto unit prefix.");
create_pre_registered_extern_component!(Atto, EcsAtto, "Atto unit prefix.");
create_pre_registered_extern_component!(Femto, EcsFemto, "Femto unit prefix.");
create_pre_registered_extern_component!(Pico, EcsPico, "Pico unit prefix.");
create_pre_registered_extern_component!(Nano, EcsNano, "Nano unit prefix.");
create_pre_registered_extern_component!(Micro, EcsMicro, "Micro unit prefix.");
create_pre_registered_extern_component!(Milli, EcsMilli, "Milli unit prefix.");
create_pre_registered_extern_component!(Centi, EcsCenti, "Centi unit prefix.");
create_pre_registered_extern_component!(Deci, EcsDeci, "Deci unit prefix.");
create_pre_registered_extern_component!(Deca, EcsDeca, "Deca unit prefix.");
create_pre_registered_extern_component!(Hecto, EcsHecto, "Hecto unit prefix.");
create_pre_registered_extern_component!(Kilo, EcsKilo, "Kilo unit prefix.");
create_pre_registered_extern_component!(Mega, EcsMega, "Mega unit prefix.");
create_pre_registered_extern_component!(Giga, EcsGiga, "Giga unit prefix.");
create_pre_registered_extern_component!(Tera, EcsTera, "Tera unit prefix.");
create_pre_registered_extern_component!(Peta, EcsPeta, "Peta unit prefix.");
create_pre_registered_extern_component!(Exa, EcsExa, "Exa unit prefix.");
create_pre_registered_extern_component!(Zetta, EcsZetta, "Zetta unit prefix.");
create_pre_registered_extern_component!(Yotta, EcsYotta, "Yotta unit prefix.");
create_pre_registered_extern_component!(Kibi, EcsKibi, "Kibi unit prefix.");
create_pre_registered_extern_component!(Mebi, EcsMebi, "Mebi unit prefix.");
create_pre_registered_extern_component!(Gibi, EcsGibi, "Gibi unit prefix.");
create_pre_registered_extern_component!(Tebi, EcsTebi, "Tebi unit prefix.");
create_pre_registered_extern_component!(Pebi, EcsPebi, "Pebi unit prefix.");
create_pre_registered_extern_component!(Exbi, EcsExbi, "Exbi unit prefix.");
create_pre_registered_extern_component!(Zebi, EcsZebi, "Zebi unit prefix.");
create_pre_registered_extern_component!(Yobi, EcsYobi, "Yobi unit prefix.");

// Quantities
create_pre_registered_extern_component!(Duration, EcsDuration, "Duration quantity.");
create_pre_registered_extern_component!(Time, EcsTime, "Time quantity.");
create_pre_registered_extern_component!(Mass, EcsMass, "Mass quantity.");
create_pre_registered_extern_component!(
    ElectricCurrent,
    EcsElectricCurrent,
    "Electric current quantity."
);
create_pre_registered_extern_component!(
    LuminousIntensity,
    EcsLuminousIntensity,
    "Luminous intensity quantity."
);
create_pre_registered_extern_component!(Force, EcsForce, "Force quantity.");
create_pre_registered_extern_component!(Amount, EcsAmount, "Amount quantity.");
create_pre_registered_extern_component!(Length, EcsLength, "Length quantity.");
create_pre_registered_extern_component!(Pressure, EcsPressure, "Pressure quantity.");
create_pre_registered_extern_component!(Speed, EcsSpeed, "Speed quantity.");
create_pre_registered_extern_component!(Temperature, EcsTemperature, "Temperature quantity.");
create_pre_registered_extern_component!(Data, EcsData, "Data quantity.");
create_pre_registered_extern_component!(DataRate, EcsDataRate, "Data rate quantity.");
create_pre_registered_extern_component!(Angle, EcsAngle, "Angle quantity.");
create_pre_registered_extern_component!(Frequency, EcsFrequency, "Frequency quantity.");
create_pre_registered_extern_component!(Uri, EcsUri, "URI quantity.");
create_pre_registered_extern_component!(Color, EcsColor, "Color quantity.");

// Durations
pub mod duration {
    use super::*;
    create_pre_registered_extern_component!(
        PicoSeconds,
        EcsPicoSeconds,
        "PicoSeconds duration unit."
    );
    create_pre_registered_extern_component!(
        NanoSeconds,
        EcsNanoSeconds,
        "NanoSeconds duration unit."
    );
    create_pre_registered_extern_component!(
        MicroSeconds,
        EcsMicroSeconds,
        "MicroSeconds duration unit."
    );
    create_pre_registered_extern_component!(
        MilliSeconds,
        EcsMilliSeconds,
        "MilliSeconds duration unit."
    );
    create_pre_registered_extern_component!(Seconds, EcsSeconds, "Seconds duration unit.");
    create_pre_registered_extern_component!(Minutes, EcsMinutes, "Minutes duration unit.");
    create_pre_registered_extern_component!(Hours, EcsHours, "Hours duration unit.");
    create_pre_registered_extern_component!(Days, EcsDays, "Days duration unit.");
}

// Angles
pub mod angle {
    use super::*;
    create_pre_registered_extern_component!(Radians, EcsRadians, "Radians angle unit.");
    create_pre_registered_extern_component!(Degrees, EcsDegrees, "Degrees angle unit.");
}

// Times
pub mod time {
    use super::*;
    create_pre_registered_extern_component!(Date, EcsDate, "Date unit.");
}

// Masses
pub mod mass {
    use super::*;
    create_pre_registered_extern_component!(Grams, EcsGrams, "Grams unit.");
    create_pre_registered_extern_component!(KiloGrams, EcsKiloGrams, "KiloGrams unit.");
}

// Electric Currents
pub mod electric_current {
    use super::*;
    create_pre_registered_extern_component!(Ampere, EcsAmpere, "Ampere unit.");
}

// Amounts
pub mod amount {
    use super::*;
    create_pre_registered_extern_component!(Mole, EcsMole, "Mole unit.");
}

// Luminous Intensities
pub mod luminous_intensity {
    use super::*;
    create_pre_registered_extern_component!(Candela, EcsCandela, "Candela unit.");
}

// Forces
pub mod force {
    use super::*;
    create_pre_registered_extern_component!(Newton, EcsNewton, "Newton unit.");
}

// Lengths
pub mod length {
    use super::*;
    create_pre_registered_extern_component!(Meters, EcsMeters, "Meters unit.");
    create_pre_registered_extern_component!(PicoMeters, EcsPicoMeters, "PicoMeters unit.");
    create_pre_registered_extern_component!(NanoMeters, EcsNanoMeters, "NanoMeters unit.");
    create_pre_registered_extern_component!(MicroMeters, EcsMicroMeters, "MicroMeters unit.");
    create_pre_registered_extern_component!(MilliMeters, EcsMilliMeters, "MilliMeters unit.");
    create_pre_registered_extern_component!(CentiMeters, EcsCentiMeters, "CentiMeters unit.");
    create_pre_registered_extern_component!(KiloMeters, EcsKiloMeters, "KiloMeters unit.");
    create_pre_registered_extern_component!(Miles, EcsMiles, "Miles unit.");
    create_pre_registered_extern_component!(Pixels, EcsPixels, "Pixels unit.");
}

// Pressure
pub mod pressure {
    use super::*;
    create_pre_registered_extern_component!(Pascal, EcsPascal, "Pascal unit.");
    create_pre_registered_extern_component!(Bar, EcsBar, "Bar unit.");
}

// Speed
pub mod speed {
    use super::*;
    create_pre_registered_extern_component!(
        MetersPerSecond,
        EcsMetersPerSecond,
        "MetersPerSecond unit."
    );
    create_pre_registered_extern_component!(
        KiloMetersPerSecond,
        EcsKiloMetersPerSecond,
        "KiloMetersPerSecond unit."
    );
    create_pre_registered_extern_component!(
        KiloMetersPerHour,
        EcsKiloMetersPerHour,
        "KiloMetersPerHour unit."
    );
    create_pre_registered_extern_component!(MilesPerHour, EcsMilesPerHour, "MilesPerHour unit.");
}

// Temperature
pub mod temperature {
    use super::*;
    create_pre_registered_extern_component!(Kelvin, EcsKelvin, "Kelvin unit.");
    create_pre_registered_extern_component!(Celsius, EcsCelsius, "Celsius unit.");
    create_pre_registered_extern_component!(Fahrenheit, EcsFahrenheit, "Fahrenheit unit.");
}

// Data
pub mod data {
    use super::*;
    create_pre_registered_extern_component!(Bits, EcsBits, "Bits unit.");
    create_pre_registered_extern_component!(KiloBits, EcsKiloBits, "KiloBits unit.");
    create_pre_registered_extern_component!(MegaBits, EcsMegaBits, "MegaBits unit.");
    create_pre_registered_extern_component!(GigaBits, EcsGigaBits, "GigaBits unit.");
    create_pre_registered_extern_component!(Bytes, EcsBytes, "Bytes unit.");
    create_pre_registered_extern_component!(KiloBytes, EcsKiloBytes, "KiloBytes unit.");
    create_pre_registered_extern_component!(MegaBytes, EcsMegaBytes, "MegaBytes unit.");
    create_pre_registered_extern_component!(GigaBytes, EcsGigaBytes, "GigaBytes unit.");
    create_pre_registered_extern_component!(KibiBytes, EcsKibiBytes, "KibiBytes unit.");
    create_pre_registered_extern_component!(MebiBytes, EcsMebiBytes, "MebiBytes unit.");
    create_pre_registered_extern_component!(GibiBytes, EcsGibiBytes, "GibiBytes unit.");
}

// DataRates
pub mod datarate {
    use super::*;
    create_pre_registered_extern_component!(BitsPerSecond, EcsBitsPerSecond, "BitsPerSecond unit.");
    create_pre_registered_extern_component!(
        KiloBitsPerSecond,
        EcsKiloBitsPerSecond,
        "KiloBitsPerSecond unit."
    );
    create_pre_registered_extern_component!(
        MegaBitsPerSecond,
        EcsMegaBitsPerSecond,
        "MegaBitsPerSecond unit."
    );
    create_pre_registered_extern_component!(
        GigaBitsPerSecond,
        EcsGigaBitsPerSecond,
        "GigaBitsPerSecond unit."
    );
    create_pre_registered_extern_component!(
        BytesPerSecond,
        EcsBytesPerSecond,
        "BytesPerSecond unit."
    );
    create_pre_registered_extern_component!(
        KiloBytesPerSecond,
        EcsKiloBytesPerSecond,
        "KiloBytesPerSecond unit."
    );
    create_pre_registered_extern_component!(
        MegaBytesPerSecond,
        EcsMegaBytesPerSecond,
        "MegaBytesPerSecond unit."
    );
    create_pre_registered_extern_component!(
        GigaBytesPerSecond,
        EcsGigaBytesPerSecond,
        "GigaBytesPerSecond unit."
    );
}

// Frequency
pub mod frequency {
    use super::*;
    create_pre_registered_extern_component!(Hertz, EcsHertz, "Hertz unit.");
    create_pre_registered_extern_component!(KiloHertz, EcsKiloHertz, "KiloHertz unit.");
    create_pre_registered_extern_component!(MegaHertz, EcsMegaHertz, "MegaHertz unit.");
    create_pre_registered_extern_component!(GigaHertz, EcsGigaHertz, "GigaHertz unit.");
}

// URI
pub mod uri {
    use super::*;
    create_pre_registered_extern_component!(Hyperlink, EcsUriHyperlink, "UriHyperlink unit.");
    create_pre_registered_extern_component!(Image, EcsUriImage, "UriImage unit.");
    create_pre_registered_extern_component!(File, EcsUriFile, "UriFile unit.");
}

// Color
pub mod color {
    use super::*;
    create_pre_registered_extern_component!(Rgb, EcsColorRgb, "Rgb color unit.");
    create_pre_registered_extern_component!(Hsl, EcsColorHsl, "Hsl color unit.");
    create_pre_registered_extern_component!(Css, EcsColorCss, "Css color unit.");
}

// Others
create_pre_registered_extern_component!(Percentage, EcsPercentage, "Percentage unit.");
create_pre_registered_extern_component!(Bel, EcsBel, "Bel unit.");
create_pre_registered_extern_component!(DeciBel, EcsDeciBel, "DeciBel unit.");
