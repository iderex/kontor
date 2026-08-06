//! The properties of docs/decisions/0005-money-and-time.md, checked over every
//! value of a bounded domain rather than over a sample of one.
//!
//! Exhaustive where the domain allows it: every pair of currencies, every day
//! in a four year window, every quarter in that window. Where the domain is an
//! integer the values are a fixed list including the awkward ones, because a
//! random draw over i64 lands on nothing interesting almost every time.
//!
//! The window is 2024 through 2027 inclusive. It contains two leap days, four
//! spring transitions and four autumn ones, so no case below is a case somebody
//! had to remember to write.

use jiff::civil::{Date, date};
use jiff::tz::TimeZone;
use kontor_money::{
    Currency, Money, MoneyError, Quarter, Rate, Total, hours_between, start_of_day,
};

const CURRENCIES: [Currency; 7] = [
    Currency::Eur,
    Currency::Usd,
    Currency::Gbp,
    Currency::Chf,
    Currency::Sek,
    Currency::Jpy,
    Currency::Isk,
];

/// What each currency's exponent is, stated here rather than read out of the
/// crate. A test that asks the code what the answer should be and then checks
/// the code against it passes whatever the code says: the first version of the
/// test below did exactly that, and switching JPY to two decimal places left it
/// green.
const EXPECTED_EXPONENT: [(Currency, u32); 7] = [
    (Currency::Eur, 2),
    (Currency::Usd, 2),
    (Currency::Gbp, 2),
    (Currency::Chf, 2),
    (Currency::Sek, 2),
    (Currency::Jpy, 0),
    (Currency::Isk, 0),
];

const AMOUNTS: [i64; 9] = [0, 1, -1, 99, 100, -100, 123_456_789, i64::MAX, i64::MIN];

const FIRST_YEAR: i16 = 2024;
const LAST_YEAR: i16 = 2027;

fn every_day() -> impl Iterator<Item = Date> {
    let mut day = date(FIRST_YEAR, 1, 1);
    let end = date(LAST_YEAR + 1, 1, 1);
    std::iter::from_fn(move || {
        if day >= end {
            return None;
        }
        let current = day;
        day = day
            .tomorrow()
            .expect("a day inside the window has a tomorrow");
        Some(current)
    })
}

fn every_quarter() -> impl Iterator<Item = Quarter> {
    (FIRST_YEAR..=LAST_YEAR).flat_map(|year| (1..=4).map(move |quarter| Quarter { year, quarter }))
}

/// Adding two amounts is possible exactly when they are the same currency, for
/// every pair of currencies this instance knows. This is the property the
/// absence of an `Add` implementation exists to force, and the one that makes a
/// mixed total impossible to produce by accident.
#[test]
fn addition_is_refused_across_currencies_and_allowed_within_one() {
    for left in CURRENCIES {
        for right in CURRENCIES {
            let a = Money::from_minor(100, left);
            let b = Money::from_minor(100, right);
            match a.try_add(b) {
                Ok(sum) => {
                    assert_eq!(left, right, "{left} and {right} were added");
                    assert_eq!(sum.minor_units(), 200);
                    assert_eq!(sum.currency(), left);
                }
                Err(MoneyError::CurrencyMismatch { .. }) => {
                    assert_ne!(left, right, "{left} could not be added to itself");
                }
                Err(other) => panic!("{left} plus {right} failed with {other}"),
            }
        }
    }
}

/// Addition within one currency is commutative, and it never wraps. The two
/// extreme values are in the list precisely so that the overflow branch is
/// reached rather than assumed.
#[test]
fn addition_is_commutative_and_never_wraps() {
    for currency in CURRENCIES {
        for left in AMOUNTS {
            for right in AMOUNTS {
                let a = Money::from_minor(left, currency);
                let b = Money::from_minor(right, currency);
                match (a.try_add(b), b.try_add(a)) {
                    (Ok(one), Ok(other)) => {
                        assert_eq!(one, other, "{left} + {right} depends on the order");
                        assert_eq!(
                            one.minor_units(),
                            left.checked_add(right).expect("it did not overflow"),
                        );
                    }
                    (Err(MoneyError::Overflow), Err(MoneyError::Overflow)) => {
                        assert!(left.checked_add(right).is_none());
                    }
                    (one, other) => panic!("{left} + {right} gave {one:?} and {other:?}"),
                }
            }
        }
    }
}

/// A currency with no minor unit has no fractional part anywhere: not in what
/// it accepts, not in what it stores, and not in what it prints.
#[test]
fn a_currency_with_no_minor_unit_has_no_fractional_part() {
    for (currency, expected_exponent) in EXPECTED_EXPONENT {
        assert_eq!(
            currency.exponent(),
            expected_exponent,
            "{currency} does not have the exponent it is supposed to"
        );
        let printed = Money::from_minor(12_345, currency).to_string();
        if expected_exponent == 0 {
            assert_eq!(currency.minor_units_per_major(), 1);
            assert!(
                !printed.contains('.'),
                "{currency} printed a decimal point in {printed}"
            );
            // One unit of the currency is one minor unit, and asking for a
            // fraction of it is refused rather than rounded away.
            assert_eq!(
                Money::from_major_minor(7, 0, currency)
                    .expect("a whole amount")
                    .minor_units(),
                7
            );
            assert_eq!(
                Money::from_major_minor(7, 1, currency),
                Err(MoneyError::Overflow)
            );
        } else {
            let places = printed
                .split_once('.')
                .expect("a currency with a minor unit prints one")
                .1
                .split_whitespace()
                .next()
                .expect("the digits come before the code")
                .len();
            assert_eq!(places, expected_exponent as usize);
            assert_eq!(
                Money::from_major_minor(7, 5, currency)
                    .expect("a fraction")
                    .minor_units(),
                7 * currency.minor_units_per_major() + 5
            );
        }
    }
}

/// Every day in the window lands in exactly one quarter, and lands inside that
/// quarter's own range. Written this way round on purpose: it checks the two
/// functions against each other rather than against a table somebody typed.
#[test]
fn every_day_lands_in_exactly_one_quarter() {
    for day in every_day() {
        let quarter = Quarter::of(day);
        assert!(
            quarter.first_day() <= day && day < quarter.first_day_after(),
            "{day} was placed in {quarter:?}, which does not contain it"
        );
        let matches = every_quarter()
            .filter(|candidate| candidate.first_day() <= day && day < candidate.first_day_after())
            .count();
        assert_eq!(matches, 1, "{day} is in {matches} quarters");
    }
}

/// The boundary itself. The last day of a period is in that period and the next
/// day is in the next one, which is the off by one that puts a deal in the
/// wrong quarter and is invisible in every total afterwards.
#[test]
fn the_last_day_of_a_period_is_in_it_and_the_next_day_is_not() {
    for quarter in every_quarter() {
        let first_after = quarter.first_day_after();
        let last = first_after.yesterday().expect("a quarter has a last day");
        assert_eq!(Quarter::of(last), quarter, "{last} left its own quarter");
        assert_ne!(
            Quarter::of(first_after),
            quarter,
            "{first_after} stayed in the quarter before it"
        );
        assert_eq!(Quarter::of(quarter.first_day()), quarter);
    }
}

/// A leap day is a day like any other, and the day that does not exist is
/// refused rather than rolled into March.
#[test]
fn a_leap_day_is_a_day_and_a_missing_one_is_refused() {
    let leap = date(2024, 2, 29);
    assert_eq!(
        Quarter::of(leap),
        Quarter {
            year: 2024,
            quarter: 1
        }
    );
    assert_eq!(
        leap.tomorrow().expect("a leap day has a tomorrow"),
        date(2024, 3, 1)
    );
    assert!(Date::new(2027, 2, 29).is_err(), "2027-02-29 was accepted");

    // Every February in the window, so the count is derived rather than typed.
    for year in FIRST_YEAR..=LAST_YEAR {
        let days = every_day()
            .filter(|day| day.year() == year && day.month() == 2)
            .count();
        let leap_year = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        assert_eq!(days, if leap_year { 29 } else { 28 }, "February {year}");
    }
}

/// A daylight saving change inside a reporting period changes the length of the
/// period, and the length is what a report divides by. In a zone that moves its
/// clocks, the first quarter of a year is an hour short and the fourth is an
/// hour long; in a zone that does not, and in UTC, every quarter is exactly
/// twenty four hours a day.
#[test]
fn a_daylight_saving_change_inside_a_period_changes_its_length() {
    let berlin = TimeZone::get("Europe/Berlin").expect("the zone is in the database");
    let tokyo = TimeZone::get("Asia/Tokyo").expect("the zone is in the database");
    let utc = TimeZone::UTC;

    for quarter in every_quarter() {
        let days = every_day()
            .filter(|day| Quarter::of(*day) == quarter)
            .count() as f64;

        for zone in [&tokyo, &utc] {
            let (start, end) = quarter.moment_range(zone);
            assert_eq!(
                hours_between(start, end),
                days * 24.0,
                "{quarter:?} in a zone with no transition"
            );
        }

        let (start, end) = quarter.moment_range(&berlin);
        let hours = hours_between(start, end);
        let expected = match quarter.quarter {
            1 => days * 24.0 - 1.0,
            4 => days * 24.0 + 1.0,
            _ => days * 24.0,
        };
        assert_eq!(hours, expected, "{quarter:?} in Europe/Berlin");
    }
}

/// The first moment of a day is the first moment that exists, not the reading
/// of a clock that never showed it. Checked over every day in the window in a
/// zone that moves its clocks, so the transition days are included without
/// being named.
#[test]
fn every_day_has_a_first_moment_and_the_days_are_in_order() {
    let berlin = TimeZone::get("Europe/Berlin").expect("the zone is in the database");
    let mut previous = None;
    for day in every_day() {
        let moment = start_of_day(day, &berlin);
        if let Some(previous) = previous {
            assert!(
                moment > previous,
                "{day} did not start after the day before"
            );
        }
        previous = Some(moment);
        assert_eq!(
            kontor_money::day_of(moment, &berlin),
            day,
            "the first moment of {day} is not on {day}"
        );
    }
}

/// A conversion carries its rate and the moment the rate was taken, and a rate
/// quoted for one currency cannot be applied to another.
#[test]
fn a_conversion_carries_its_rate_and_refuses_the_wrong_currency() {
    let taken_at = "2026-01-15T09:00:00Z".parse().expect("a timestamp");
    let rate = Rate::new(Currency::Usd, Currency::Eur, 920_000, taken_at).expect("a positive rate");

    let converted = rate
        .apply(Money::from_minor(10_000, Currency::Usd))
        .expect("the rate applies");
    assert_eq!(converted.amount(), Money::from_minor(9_200, Currency::Eur));
    assert_eq!(converted.source(), Money::from_minor(10_000, Currency::Usd));
    assert_eq!(converted.rate().taken_at(), taken_at);

    for currency in CURRENCIES {
        if currency == Currency::Usd {
            continue;
        }
        assert!(
            matches!(
                rate.apply(Money::from_minor(1, currency)),
                Err(MoneyError::RateDoesNotApply { .. })
            ),
            "a rate from USD applied to {currency}"
        );
    }

    assert_eq!(
        Rate::new(Currency::Usd, Currency::Eur, 0, taken_at),
        Err(MoneyError::RateNotPositive)
    );
}

/// A total in one currency cannot absorb another currency without a rate, and a
/// total that used one says so and can produce it.
#[test]
fn a_total_across_currencies_carries_every_rate_it_used() {
    let taken_at = "2026-01-15T09:00:00Z".parse().expect("a timestamp");
    let rate = Rate::new(Currency::Usd, Currency::Eur, 920_000, taken_at).expect("a positive rate");

    let mut total = Total::zero(Currency::Eur);
    total
        .add(Money::from_minor(5_000, Currency::Eur))
        .expect("same currency");
    assert!(!total.is_converted());
    assert!(total.rates().is_empty());

    assert!(matches!(
        total.add(Money::from_minor(5_000, Currency::Usd)),
        Err(MoneyError::CurrencyMismatch { .. })
    ));

    total
        .add_converted(
            rate.apply(Money::from_minor(10_000, Currency::Usd))
                .expect("applies"),
        )
        .expect("the converted amount is in the total's currency");

    assert_eq!(total.amount(), Money::from_minor(14_200, Currency::Eur));
    assert!(total.is_converted());
    assert_eq!(total.rates().len(), 1);
    assert_eq!(total.rates()[0].taken_at(), taken_at);
}
