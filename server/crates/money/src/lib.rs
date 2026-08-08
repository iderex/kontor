// SPDX-License-Identifier: AGPL-3.0-only
//! Money, dates and moments, with the rules of
//! docs/decisions/0005-money-and-time.md expressed as types.
//!
//! Three things are refused here rather than reviewed for later: adding two
//! amounts in different currencies, holding an amount as a floating point
//! number, and converting between currencies without recording the rate and
//! when it was taken. None of the three has an operator visible symptom until a
//! report is wrong six months later, which is why each one is a type rather
//! than a rule somebody remembers.

use std::fmt;

use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{Timestamp, Unit, Zoned};

/// A currency, with the exponent that says how many minor units make one major
/// unit.
///
/// The set is small on purpose. A currency this instance does not know is
/// refused at the edge rather than defaulted to two decimal places, because two
/// is wrong for the zero exponent currencies and being wrong quietly is what
/// this module exists against. Adding one is a line here and a migration for
/// any stored code, and #22 is where the argument for a full ISO 4217 table
/// belongs if the small set stops being enough.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Currency {
    Eur,
    Usd,
    Gbp,
    Chf,
    Sek,
    /// Zero exponent. One yen is one minor unit and there is no subdivision.
    Jpy,
    /// Zero exponent, and the second one, so nothing here can treat the zero
    /// case as a single special currency.
    Isk,
}

impl Currency {
    /// The ISO 4217 alphabetic code.
    pub fn code(self) -> &'static str {
        match self {
            Currency::Eur => "EUR",
            Currency::Usd => "USD",
            Currency::Gbp => "GBP",
            Currency::Chf => "CHF",
            Currency::Sek => "SEK",
            Currency::Jpy => "JPY",
            Currency::Isk => "ISK",
        }
    }

    /// How many decimal places the minor unit sits at. Zero means the amount
    /// has no fractional part at all.
    pub fn exponent(self) -> u32 {
        match self {
            Currency::Jpy | Currency::Isk => 0,
            _ => 2,
        }
    }

    /// The number of minor units in one major unit.
    pub fn minor_units_per_major(self) -> i64 {
        10_i64.pow(self.exponent())
    }

    pub fn from_code(code: &str) -> Result<Self, MoneyError> {
        match code {
            "EUR" => Ok(Currency::Eur),
            "USD" => Ok(Currency::Usd),
            "GBP" => Ok(Currency::Gbp),
            "CHF" => Ok(Currency::Chf),
            "SEK" => Ok(Currency::Sek),
            "JPY" => Ok(Currency::Jpy),
            "ISK" => Ok(Currency::Isk),
            other => Err(MoneyError::UnknownCurrency(other.to_owned())),
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// What can go wrong, named rather than collapsed into one error.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MoneyError {
    /// Two amounts in different currencies were combined without a rate.
    CurrencyMismatch {
        left: Currency,
        right: Currency,
    },
    /// A sum left the range of the integer holding it.
    Overflow,
    /// A rate was applied to an amount it was not quoted for.
    RateDoesNotApply {
        rate_from: Currency,
        amount: Currency,
    },
    /// A rate of zero or below, which no conversion can use.
    RateNotPositive,
    UnknownCurrency(String),
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::CurrencyMismatch { left, right } => write!(
                f,
                "cannot combine {left} and {right} without a conversion rate"
            ),
            MoneyError::Overflow => {
                f.write_str("the amount left the range of the minor unit counter")
            }
            MoneyError::RateDoesNotApply { rate_from, amount } => {
                write!(
                    f,
                    "a rate quoted from {rate_from} does not apply to {amount}"
                )
            }
            MoneyError::RateNotPositive => f.write_str("a conversion rate must be above zero"),
            MoneyError::UnknownCurrency(code) => {
                write!(f, "{code} is not a currency this instance knows")
            }
        }
    }
}

impl std::error::Error for MoneyError {}

/// An amount of one currency, counted in minor units.
///
/// There is no constructor taking a floating point number and no accessor
/// returning one. A binary float cannot hold 0.1, so a total built from floats
/// depends on the order the rows came back in, and two runs of one report
/// disagree in the last place for no reason a reader can see.
///
/// There is no `Add` implementation either, and that absence is the rule. The
/// only way to combine two amounts is [`Money::try_add`], which refuses two
/// currencies, or [`Total`], which records how a conversion was made. A report
/// that wants a mixed total has to say how it made one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Money {
    minor: i64,
    currency: Currency,
}

impl Money {
    /// An amount given directly in minor units, which is how it is stored and
    /// how it arrives from the database.
    pub fn from_minor(minor: i64, currency: Currency) -> Self {
        Money { minor, currency }
    }

    /// An amount given in major units and minor units, which is how a person
    /// types one. The minor part of a zero exponent currency must be zero, and
    /// a caller passing one is refused rather than rounded.
    pub fn from_major_minor(
        major: i64,
        minor: i64,
        currency: Currency,
    ) -> Result<Self, MoneyError> {
        let scale = currency.minor_units_per_major();
        if minor.abs() >= scale && scale > 1 {
            return Err(MoneyError::Overflow);
        }
        if scale == 1 && minor != 0 {
            return Err(MoneyError::Overflow);
        }
        let scaled = major.checked_mul(scale).ok_or(MoneyError::Overflow)?;
        let total = if major < 0 {
            scaled.checked_sub(minor.abs())
        } else {
            scaled.checked_add(minor.abs())
        }
        .ok_or(MoneyError::Overflow)?;
        Ok(Money {
            minor: total,
            currency,
        })
    }

    pub fn zero(currency: Currency) -> Self {
        Money { minor: 0, currency }
    }

    pub fn minor_units(self) -> i64 {
        self.minor
    }

    pub fn currency(self) -> Currency {
        self.currency
    }

    /// Addition, refusing two currencies. This is the only addition there is.
    pub fn try_add(self, other: Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }
        Ok(Money {
            minor: self
                .minor
                .checked_add(other.minor)
                .ok_or(MoneyError::Overflow)?,
            currency: self.currency,
        })
    }

    pub fn try_sub(self, other: Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            });
        }
        Ok(Money {
            minor: self
                .minor
                .checked_sub(other.minor)
                .ok_or(MoneyError::Overflow)?,
            currency: self.currency,
        })
    }
}

impl fmt::Display for Money {
    /// Written with exactly as many decimal places as the currency has, so a
    /// zero exponent currency never appears with a fractional part somebody
    /// then tries to enter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scale = self.currency.minor_units_per_major();
        let sign = if self.minor < 0 { "-" } else { "" };
        let magnitude = self.minor.unsigned_abs();
        let major = magnitude / scale as u64;
        if self.currency.exponent() == 0 {
            write!(f, "{sign}{major} {}", self.currency)
        } else {
            let minor = magnitude % scale as u64;
            write!(
                f,
                "{sign}{major}.{minor:0width$} {}",
                self.currency,
                width = self.currency.exponent() as usize
            )
        }
    }
}

/// A conversion rate, and the moment it was taken.
///
/// The moment is not optional and there is no constructor without it. A rate
/// without one cannot be checked later and cannot be told apart from today's
/// rate applied to last year's deal, which is the specific thing a report may
/// not do silently.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rate {
    from: Currency,
    to: Currency,
    /// Minor units of `to` per one million minor units of `from`, which keeps
    /// the rate an integer. A float here would put the defect this module
    /// refuses back in through the conversion path.
    per_million: i64,
    taken_at: Timestamp,
}

impl Rate {
    pub fn new(
        from: Currency,
        to: Currency,
        per_million: i64,
        taken_at: Timestamp,
    ) -> Result<Self, MoneyError> {
        if per_million <= 0 {
            return Err(MoneyError::RateNotPositive);
        }
        Ok(Rate {
            from,
            to,
            per_million,
            taken_at,
        })
    }

    pub fn from(self) -> Currency {
        self.from
    }

    pub fn to(self) -> Currency {
        self.to
    }

    pub fn taken_at(self) -> Timestamp {
        self.taken_at
    }

    /// Applies the rate, carrying itself into the result.
    pub fn apply(self, amount: Money) -> Result<Converted, MoneyError> {
        if amount.currency() != self.from {
            return Err(MoneyError::RateDoesNotApply {
                rate_from: self.from,
                amount: amount.currency(),
            });
        }
        let scaled = (amount.minor_units() as i128) * (self.per_million as i128);
        let converted = scaled / 1_000_000;
        let converted = i64::try_from(converted).map_err(|_| MoneyError::Overflow)?;
        Ok(Converted {
            amount: Money::from_minor(converted, self.to),
            source: amount,
            rate: self,
        })
    }
}

/// An amount that came out of a conversion, with the amount it came from and
/// the rate that made it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Converted {
    amount: Money,
    source: Money,
    rate: Rate,
}

impl Converted {
    pub fn amount(self) -> Money {
        self.amount
    }

    pub fn source(self) -> Money {
        self.source
    }

    pub fn rate(self) -> Rate {
        self.rate
    }
}

/// A total in one currency, which remembers every conversion that went into it.
///
/// A total built only from amounts already in its currency has an empty rate
/// list, and that is how a reader tells a converted total from a plain one. The
/// list is not a log for debugging: a report showing a converted total is
/// required to show these rates and their moments beside it.
#[derive(Clone, Debug)]
pub struct Total {
    amount: Money,
    rates: Vec<Rate>,
}

impl Total {
    pub fn zero(currency: Currency) -> Self {
        Total {
            amount: Money::zero(currency),
            rates: Vec::new(),
        }
    }

    /// Adds an amount that is already in the total's currency.
    pub fn add(&mut self, amount: Money) -> Result<(), MoneyError> {
        self.amount = self.amount.try_add(amount)?;
        Ok(())
    }

    /// Adds an amount that had to be converted, keeping the rate.
    pub fn add_converted(&mut self, converted: Converted) -> Result<(), MoneyError> {
        self.amount = self.amount.try_add(converted.amount())?;
        self.rates.push(converted.rate());
        Ok(())
    }

    pub fn amount(&self) -> Money {
        self.amount
    }

    /// Every rate used, in the order they were applied. Empty means nothing was
    /// converted.
    pub fn rates(&self) -> &[Rate] {
        &self.rates
    }

    pub fn is_converted(&self) -> bool {
        !self.rates.is_empty()
    }
}

/// A day with no time and no zone, which is what a close date is.
///
/// A close date is the day a deal closes and it is the same day for everybody
/// looking at it. Storing it as a moment forces a zone on it, and then the last
/// day of a quarter lands in two different quarters for two people. Stored as
/// `date`.
pub type CloseDate = Date;

/// A moment on the clock, which is what a change log entry is. Stored as
/// `timestamptz`, which PostgreSQL holds as an instant rather than as a local
/// reading.
pub type Moment = Timestamp;

/// A calendar quarter of a year.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Quarter {
    pub year: i16,
    /// 1 to 4.
    pub quarter: u8,
}

impl Quarter {
    /// The quarter a close date lands in.
    ///
    /// No zone is taken and none is needed, which is the whole point of holding
    /// a close date as a date.
    pub fn of(date: CloseDate) -> Quarter {
        Quarter {
            year: date.year(),
            quarter: ((date.month() as u8 - 1) / 3) + 1,
        }
    }

    /// The first day of the quarter.
    pub fn first_day(self) -> CloseDate {
        Date::new(self.year, ((self.quarter - 1) * 3 + 1) as i8, 1)
            .expect("a quarter's first day is a valid date")
    }

    /// The day after the quarter's last day, which is the exclusive end used
    /// everywhere a range is needed.
    pub fn first_day_after(self) -> CloseDate {
        if self.quarter == 4 {
            Quarter {
                year: self.year + 1,
                quarter: 1,
            }
            .first_day()
        } else {
            Quarter {
                year: self.year,
                quarter: self.quarter + 1,
            }
            .first_day()
        }
    }

    /// The half open range of moments this quarter covers in a given zone.
    ///
    /// This is the one place a zone enters, and it enters because the question
    /// is about moments rather than about days. The instance zone decides, and
    /// where a transition falls inside the range the range is shorter or longer
    /// by the transition rather than by a fixed number of hours.
    pub fn moment_range(self, zone: &TimeZone) -> (Moment, Moment) {
        (
            start_of_day(self.first_day(), zone),
            start_of_day(self.first_day_after(), zone),
        )
    }
}

/// The first moment of a day in a zone.
///
/// A day does not always start at midnight. Where a zone moves its clocks
/// forward at midnight, the local time 00:00 does not exist that day, and jiff
/// resolves it to the first moment that does rather than to a time nobody's
/// clock showed.
pub fn start_of_day(date: CloseDate, zone: &TimeZone) -> Moment {
    date.to_zoned(zone.clone())
        .expect("a civil date has a first moment in every zone")
        .timestamp()
}

/// The number of hours between two moments, which is what a period's length
/// actually is once a daylight saving transition is inside it.
pub fn hours_between(start: Moment, end: Moment) -> f64 {
    let span = end - start;
    span.total(Unit::Hour)
        .expect("an hour total of a timestamp span is always computable")
}

/// The calendar day a moment falls on in a zone.
///
/// Used where a change log entry has to be placed in a reporting period. Two
/// entries one second apart can land on different days, and which day either
/// lands on is a fact about the zone rather than about the entry.
pub fn day_of(moment: Moment, zone: &TimeZone) -> CloseDate {
    Zoned::new(moment, zone.clone()).date()
}
