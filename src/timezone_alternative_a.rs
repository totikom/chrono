#![allow(dead_code, unreachable_pub)]

use crate::offset::FixedOffset;
use crate::Days;
use crate::Months;
use crate::NaiveDateTime;
use crate::ParseResult;
use crate::TimeDelta;
use crate::Utc;
use core::fmt::{Debug, Display};
use std::error::Error;

pub struct DateTime<Tz: TimeZone> {
    datetime: NaiveDateTime,
    zone: Tz,
}

impl<Tz> DateTime<Tz>
where
    Tz: TimeZone,
{
    fn naive_utc(&self) -> NaiveDateTime {
        self.datetime
    }
    fn fixed(&self) -> DateTime<FixedOffset> {
        DateTime { zone: self.zone.offset(), datetime: self.datetime }
    }
}

impl<Tz> DateTime<Tz>
where
    Tz: TimeZone + Clone,
{
    // keep this out of the `TimeZone` trait to avoid object safety problems
    fn parse_from_str_tz(
        _input: &str,
        _format: &str,
        _timezone: &Tz,
    ) -> ParseResult<Result<DateTime<Tz>, InvalidLocalTimeInfoTz<Tz>>> {
        todo!()
    }
    fn add_months(&self, months: Months) -> DateTime<Tz> {
        let new_datetime = self.naive_utc() + months;
        new_datetime.and_timezone_2(&self.zone)
    }
    fn sub_months(&self, months: Months) -> DateTime<Tz> {
        let new_datetime = self.naive_utc() - months;
        new_datetime.and_timezone_2(&self.zone)
    }
    fn add_days(&self, days: Days) -> DateTime<Tz> {
        let new_datetime = self.naive_utc() + days;
        new_datetime.and_timezone_2(&self.zone)
    }
    fn sub_days(&self, days: Days) -> DateTime<Tz> {
        let new_datetime = self.naive_utc() - days;
        new_datetime.and_timezone_2(&self.zone)
    }
    fn add(&self, duration: TimeDelta) -> DateTime<Tz> {
        let new_datetime = self.naive_utc() + duration;
        new_datetime.and_timezone_2(&self.zone)
    }
    fn sub(&self, duration: TimeDelta) -> DateTime<Tz> {
        let new_datetime = self.naive_utc() - duration;
        new_datetime.and_timezone_2(&self.zone)
    }
}

impl NaiveDateTime {
    fn and_local_timezone_2<Tz>(
        self,
        timezone: &Tz,
    ) -> Result<DateTime<Tz>, InvalidLocalTimeInfoTz<Tz>>
    where
        Tz: TimeZone + Clone,
    {
        match timezone.offset_at_local(self) {
            Ok(offset) => {
                let mut zone = timezone.clone();
                zone.update_offset_at_local(self).map_err(|e| e.and_tz(timezone.clone()))?;
                Ok(DateTime { datetime: self - offset, zone })
            }
            Err(e) => Err(InvalidLocalTimeInfoTz {
                local: e.local,
                transition: e.transition,
                tz: timezone.clone(),
            }),
        }
    }

    fn and_timezone_2<Tz>(self, timezone: &Tz) -> DateTime<Tz>
    where
        Tz: TimeZone + Clone,
    {
        let mut zone = timezone.clone();
        zone.update_offset_at(self);
        DateTime { datetime: self, zone }
    }
}

impl<Tz> Clone for DateTime<Tz>
where
    Tz: TimeZone + Clone,
{
    fn clone(&self) -> Self {
        DateTime { datetime: self.datetime, zone: self.zone.clone() }
    }
}

impl<Tz> Copy for DateTime<Tz> where Tz: TimeZone + Copy {}

// a given transition time, similar format to tzinfo,
// including the Utc timestamp of the transition,
// the offset prior to the transition, and the offset
// after the transition
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UtcTransition {
    at: NaiveDateTime,
    from: FixedOffset,
    to: FixedOffset,
}

// a transition but where the NaiveDateTime's represent
// a local time rather than a Utc time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalTransition {
    transition_start: (NaiveDateTime, FixedOffset),
    transition_end: (NaiveDateTime, FixedOffset),
}

impl UtcTransition {
    fn current_offset(&self) -> FixedOffset {
        self.to
    }
    fn local_start(&self) -> NaiveDateTime {
        self.at + self.from
    }
    fn local_end(&self) -> NaiveDateTime {
        self.at + self.to
    }
    fn local(&self) -> LocalTransition {
        LocalTransition {
            transition_start: (self.local_start(), self.from),
            transition_end: (self.local_end(), self.to),
        }
    }
}

// this structure is returned when asking for the transitions
// immediately prior to and after a given Utc or Local time.
// when asking with a given local time, the before and after
// will occasionally be equal
pub struct ClosestTransitions {
    before: UtcTransition,
    after: UtcTransition,
}

// a replacement for the Err part of a LocalResult.
// this allows us to use a regular std::result::Result
// and pass this in the Err branch
//
// this should also contain enough of the original data
// such that it is possible to implement helper methods \
// to, for example, get a "good enough" conversion from a
// local time where the local timestamp is invalid or ambiguous
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidLocalTimeInfo {
    local: NaiveDateTime,
    transition: UtcTransition,
}

impl Display for InvalidLocalTimeInfo {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl Error for InvalidLocalTimeInfo {}

impl InvalidLocalTimeInfo {
    fn and_tz<Tz>(&self, tz: Tz) -> InvalidLocalTimeInfoTz<Tz> {
        InvalidLocalTimeInfoTz { local: self.local, transition: self.transition.clone(), tz }
    }
}

// as above, but with the TimeZone parameter. This exists because some API's
// can't return this version due to object safety, but it is still nice to
// have where possible.
pub struct InvalidLocalTimeInfoTz<Tz> {
    local: NaiveDateTime,
    transition: UtcTransition,
    tz: Tz,
}

impl<Tz> Clone for InvalidLocalTimeInfoTz<Tz>
where
    Tz: Clone,
{
    fn clone(&self) -> Self {
        InvalidLocalTimeInfoTz {
            local: self.local,
            transition: self.transition.clone(),
            tz: self.tz.clone(),
        }
    }
}

impl<Tz> Debug for InvalidLocalTimeInfoTz<Tz> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl<Tz> Display for InvalidLocalTimeInfoTz<Tz> {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl<Tz> Error for InvalidLocalTimeInfoTz<Tz> {}

// here the TimeZoneA should be something small or static, like an empty enum variant or an empty struct.
// potentially all the methods here should be fallible
//
// the implementor of TimeZoneA will usually store its current offset internally (if dynamic) or make it available as a
// const if static.
//
// where the TimeZoneA implemention handles daylight savings or other timezone that needs more data than just an offset
// it might store a `String` or enum variant which enables the `%Z` formatting, extracted via the `.name()` method.
//
// we move the `datetime_from_str` to the `DateTime<Tz>` impl
// we have to avoid `from_local_datetime` and `from_utc_datetime` here
// and point users towards `and_local_timezone()` and `.and_timezone()` instead.
// because there is no way to force the `TimeZoneA` to implement `Clone` but still keep object safety.
// for all practical purposes all `TimeZoneA` implementors should probably implement at least `Clone` and likely `Copy` as well.
pub trait TimeZone {
    // this could have a default implementation if there was a `from_fixed_offset` method
    // in the trait, but that would be problematic for object safety, so instead
    // the implemention is left to the user.
    #[cfg(feature = "clock")]
    fn offset_now(&self) -> FixedOffset {
        self.offset_at(Utc::now().naive_utc())
    }

    fn offset(&self) -> FixedOffset;

    fn offset_at_local(&self, local: NaiveDateTime) -> Result<FixedOffset, InvalidLocalTimeInfo> {
        match self.closest_transitions_from_local(local) {
            None => Ok(self.offset()),
            Some(ClosestTransitions { before, after }) if before == after => {
                Err(InvalidLocalTimeInfo { local, transition: before })
            }
            Some(ClosestTransitions { before, .. }) => Ok(before.to),
        }
    }

    fn offset_at(&self, utc: NaiveDateTime) -> FixedOffset {
        if let Some(ClosestTransitions { before, .. }) = self.closest_transitions(utc) {
            before.current_offset()
        } else {
            self.offset()
        }
    }

    // this is needed as we can't construct a new `TimeZoneA` and still be object safe
    fn update_offset_at_local(
        &mut self,
        _local: NaiveDateTime,
    ) -> Result<(), InvalidLocalTimeInfo> {
        Ok(())
    }

    fn update_offset_at(&mut self, _utc: NaiveDateTime) {}

    // potentially the `_transitions` functions should take a `local: bool` parameter
    // as it would be incorrect to implement one but leave the other with the default impl

    // this is not hugely useful as it will just be the
    // previous and next transitions, but it might be nice
    // to expose this in public API what is currently just in `tzinfo`.
    fn closest_transitions(&self, _utc: NaiveDateTime) -> Option<ClosestTransitions> {
        None
    }

    // if the local timestamp is valid, then these transitions will each be different
    // if the local timestamp is either ambiguous or invalid, then both fields of the
    // tuple will be the same
    fn closest_transitions_from_local(&self, _local: NaiveDateTime) -> Option<ClosestTransitions> {
        None
    }

    // to be used in %Z formatting
    fn name(&self) -> Option<&str> {
        None
    }
}

impl TimeZone for Box<dyn TimeZone> {
    fn offset_now(&self) -> FixedOffset {
        self.as_ref().offset_now()
    }

    fn offset(&self) -> FixedOffset {
        self.as_ref().offset()
    }

    fn offset_at_local(&self, local: NaiveDateTime) -> Result<FixedOffset, InvalidLocalTimeInfo>
    where
        Self: Sized,
    {
        self.as_ref().offset_at_local(local)
    }

    fn offset_at(&self, utc: NaiveDateTime) -> FixedOffset {
        self.as_ref().offset_at(utc)
    }

    fn update_offset_at_local(&mut self, local: NaiveDateTime) -> Result<(), InvalidLocalTimeInfo> {
        self.as_mut().update_offset_at_local(local)
    }

    fn update_offset_at(&mut self, utc: NaiveDateTime) {
        self.as_mut().update_offset_at(utc)
    }

    fn closest_transitions(&self, utc: NaiveDateTime) -> Option<ClosestTransitions> {
        self.as_ref().closest_transitions(utc)
    }

    fn closest_transitions_from_local(&self, local: NaiveDateTime) -> Option<ClosestTransitions> {
        self.as_ref().closest_transitions_from_local(local)
    }

    fn name(&self) -> Option<&str> {
        self.as_ref().name()
    }
}

impl TimeZone for FixedOffset {
    fn offset(&self) -> FixedOffset {
        crate::offset::Offset::fix(self)
    }
}