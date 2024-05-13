//! Generic types and units for working with time.
//!
//! This is a re-export of parts of the [fugit] crate.

pub use fugit::HertzU32 as Hertz;
pub use fugit::HoursDurationU32 as HoursDuration;
pub use fugit::KilohertzU32 as Kilohertz;
pub use fugit::MegahertzU32 as Megahertz;
pub use fugit::MicrosDurationU32 as MicrosDuration;
pub use fugit::MillisDurationU32 as MillisDuration;
pub use fugit::NanosDurationU32 as NanosDuration;
pub use fugit::SecsDurationU32 as SecsDuration;
pub use fugit::TimerDurationU32 as TimerDuration;
pub use fugit::TimerInstantU32 as TimerInstant;
pub use fugit::TimerRateU32 as TimerRate;

pub use fugit::ExtU32 as DurationExtU32;
pub use fugit::ExtU32Ceil as DurationExtU32Ceil;
pub use fugit::RateExtU32;
