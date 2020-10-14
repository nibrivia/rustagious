#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! A crate to model contagion

use rand::distributions::{Bernoulli, Distribution};
use rand_distr::LogNormal;
use std::cmp::{max, min};

type Time = u64;

// old
//const SYMPTOMATIC_MU: f64 = 1.621;
//const SYMPTOMATIC_SIGMA: f64 = 0.418;

// meta https://bmjopen.bmj.com/content/bmjopen/10/8/e039652.full.pdf
const SYMPTOMATIC_MU: f64 = 1.63;
const SYMPTOMATIC_SIGMA: f64 = 0.5;

/// Struct representing an individual and keeping track of associated state
#[derive(Debug)]
pub struct Person {
    /// Person's name for easy referencing
    pub name: String,

    infection: Option<Infection>,
    tested_positive: Option<Time>,
}

/// Infection data
#[derive(Debug, Clone, Copy)]
pub struct Infection {
    date: Time,

    testable_date: Time,
    contagious_date: Time,
    recovery_date: Time,

    symptomatic_date: Option<Time>,
    // Infection's original source
    //pub source: String,
}

impl Person {
    /// Creates a new person
    pub fn new(name: String) -> Person {
        Person {
            name,
            infection: None,
            tested_positive: None,
        }
    }

    /// Exposes a person to a source on a given date
    pub fn expose(self: &mut Self, date: Time /*source: String*/) {
        // already infected, let's not do this again...
        if self.infection.is_some() {
            return;
        }

        let mut rng = rand::thread_rng();

        // Symptomatic date, everything is computed in reference to that
        // Contagious period is 2, at least one day of incubation
        let log_normal = LogNormal::new(SYMPTOMATIC_MU, SYMPTOMATIC_SIGMA).unwrap();
        let symptomatic_date: Time = date + log_normal.sample(&mut rng).round() as Time;
        let testable_date = max(date + 1, symptomatic_date - 2);
        let contagious_date = max(date + 1, symptomatic_date - 2);
        let recovery_date = symptomatic_date + 10;

        // Do we show symptoms
        let d = Bernoulli::new(0.6).unwrap();
        let v = d.sample(&mut rng);
        let symptomatic_date = if v { Some(symptomatic_date) } else { None };

        self.infection = Some(Infection {
            date,
            testable_date,
            contagious_date,
            symptomatic_date,
            recovery_date,
            //source,
        });
    }

    /// True if the infection is done/has never happened
    pub fn has_recovered(self: &mut Self, date: Time) -> bool {
        if let Some(infection) = &self.infection {
            date > infection.recovery_date
        } else {
            true
        }
    }

    /// Runs a test on a person
    pub fn test(self: &mut Self, date: Time, delay: Time) {
        if let Some(infection) = &self.infection {
            if infection.testable_date <= date {
                self.tested_positive = Some(date + delay);
            }
        }
    }

    /// Interacts two people
    pub fn interact(self: &mut Self, date: Time, other: &mut Self) {
        if other.is_contagious(date) {
            self.expose(
                date,
                //other.get_infection().as_ref().unwrap().source.to_string(),
            );
        }

        if self.is_contagious(date) {
            other.expose(date); //, self.infection.as_ref().unwrap().source.to_string());
        }
    }

    /// Is this person able to infect others?
    pub fn is_contagious(self: &Self, date: Time) -> bool {
        if let Some(infection) = &self.infection {
            return infection.contagious_date <= date && date <= infection.recovery_date;
        }
        false
    }

    /// Is this person in a state where they should be isolating?
    pub fn is_isolating(self: &Self, date: Time) -> bool {
        if let Some(tested_date) = self.tested_positive {
            if tested_date <= date {
                return true;
            }
        }
        if let Some(infection) = &self.infection {
            if let Some(symptomatic_date) = infection.symptomatic_date {
                //println!("Symptomatic and isolating!");
                return symptomatic_date <= date;
            }
        }
        false
    }

    /// Returns a reference to the current infection status
    pub fn get_infection(self: &Self) -> &Option<Infection> {
        &self.infection
    }

    /// Returns the number of days that this person was carrying the virus unaware
    pub fn days_unaware(self: &Self, date: Time) -> u64 {
        if let Some(infection) = &self.infection {
            if min(infection.recovery_date, date) > infection.contagious_date {
                min(infection.recovery_date, date) - infection.contagious_date
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Has this person *ever* been infected?
    pub fn was_sick(self: &Self, date: Time) -> bool {
        //println!("{} {:?}", self.name, self.infection);
        if let Some(infection) = &self.infection {
            return infection.date <= date;
        }
        false
    }

    /// Prints a health summary to stdout
    pub fn health_summary(self: &Self) {
        if let Some(infection) = &self.infection {
            println!(
                "{} infected on {} and is isolating? {}",
                self.name,
                infection.date,
                //infection.source,
                self.is_isolating(99999) // TODO
            );
        } else {
            println!("{} healthy", self.name);
        }
    }
}

/// Phase type
#[derive(Debug, PartialEq, Eq)]
pub enum Phase {
    /// B sees A
    A,

    /// B is isolating
    Isolate,

    /// B sees C
    C,
}

/// Returns custom phase function
pub fn gen_phase_fn(a: u64, ac: u64, c: u64, ca: u64, offset: u64) -> Box<dyn Fn(Time) -> Phase> {
    let cycle_len = a + ac + c + ca;
    Box::new(move |day| {
        let cycle_day = (day + offset) % cycle_len;
        if cycle_day < a {
            Phase::A
        } else if cycle_day < a + ac {
            Phase::Isolate
        } else if cycle_day < a + ac + c {
            Phase::C
        } else {
            Phase::Isolate
        }
    })
}

/// An example phase function
pub fn phase(day: u64) -> Phase {
    let cycle_day = day % (6 * 7);
    if cycle_day <= 15 {
        Phase::A
    } else if cycle_day >= 21 && cycle_day <= 36 {
        Phase::C
    } else {
        Phase::Isolate
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn single_infected_tested_isolating() {
        let mut me = Person::new("Olivia".to_string());
        assert_eq!(me.name, "Olivia".to_string());

        // Get sick
        me.expose(2); //, "MIT".to_string());

        // Don't know better yet, should *not* be isolating...
        assert!(!me.is_isolating(2));
        assert!(me.was_sick(4)); // ...but am sick (hidden state)

        // Get tested, no delay
        me.test(12, 0);

        // Really should be isolating
        assert!(me.is_isolating(12));
        assert!(me.was_sick(12));
    }

    #[test]
    fn interaction_abc_future() {
        let mut a = Person::new("Olivia A".to_string());
        let mut b = Person::new("Olivia B".to_string());
        let mut c = Person::new("Olivia C".to_string());
        assert!(!a.was_sick(0));
        assert!(!b.was_sick(0));
        assert!(!c.was_sick(0));

        // Sick me gets sick
        b.expose(10); //, "MIT".to_string());
        assert!(b.was_sick(10));

        // Nobody's sick, but out of order
        b.interact(1, &mut a);
        assert!(!a.was_sick(1));
        assert!(!b.was_sick(1));
        assert!(!c.was_sick(1));

        // Interact
        b.interact(19, &mut c);
        assert!(!a.was_sick(19)); // should not be sick
        assert!(b.was_sick(19)); // should *also* be sick
        assert!(c.was_sick(19)); // should *also* be sick
    }

    #[test]
    fn interaction() {
        let mut healthy_me = Person::new("Olivia Healthy".to_string());
        let mut sick_me = Person::new("Olivia Sick".to_string());
        assert!(!healthy_me.was_sick(0));
        assert!(!sick_me.was_sick(0));

        // Sick me is sick
        sick_me.expose(0); //, "MIT".to_string());
        assert!(sick_me.was_sick(2));

        // Interact
        sick_me.interact(10, &mut healthy_me);
        assert!(sick_me.was_sick(11)); // should still be sick
        assert!(healthy_me.was_sick(11)); // should *also* be sick
    }

    #[test]
    fn phase_test() {
        assert_eq!(phase(0), Phase::A, "0 with A");
        assert_eq!(phase(1), Phase::A, "1 with A");
        assert_eq!(phase(15), Phase::A, "15 with A");

        assert_eq!(phase(16), Phase::Isolate, "16 isolating");
        assert_eq!(phase(17), Phase::Isolate, "17 isolating");
        assert_eq!(phase(18), Phase::Isolate, "18 isolating");
        assert_eq!(phase(19), Phase::Isolate, "19 isolating");
        assert_eq!(phase(20), Phase::Isolate, "20 isolating");

        assert_eq!(phase(21), Phase::C, "21 C");
        assert_eq!(phase(30), Phase::C, "30 C");
        assert_eq!(phase(36), Phase::C, "36 C");

        assert_eq!(phase(37), Phase::Isolate, "37 isolating");
        assert_eq!(phase(38), Phase::Isolate, "38 isolating");
        assert_eq!(phase(39), Phase::Isolate, "39 isolating");
        assert_eq!(phase(40), Phase::Isolate, "40 isolating");
        assert_eq!(phase(41), Phase::Isolate, "41 isolating");

        assert_eq!(phase(42), Phase::A, "42 with A");
    }

    #[test]
    fn infectious_causality() {
        for _ in 0..5000 {
            let mut me = Person::new("Olivia".to_string());
            me.expose(100);
            me.test(100, 0);
            assert!(!me.was_sick(99));
            assert!(!me.is_isolating(100));
        }
    }

    #[test]
    fn delay_testing() {
        for _ in 0..5_000 {
            let mut me = Person::new("Olivia".to_string());
            me.expose(100);

            let infection = me.get_infection().unwrap();
            let t = infection.testable_date;

            if let Some(s) = infection.symptomatic_date {
                me.test(t, 5);
                assert!(t <= s);

                // no symptoms, or test result
                for d in t..s {
                    assert!(!me.is_isolating(d));
                }

                // should be isolating when we have symptoms
                assert!(me.is_isolating(s));

                // should be isolating when we get results back
                assert!(me.is_isolating(t + 5));
            } else {
                // testing is too early
                me.test(t - 1, 2);
                assert!(!me.is_isolating(t + 1));

                me.test(t, 5);
                assert!(!me.is_isolating(t)); // should not be isolating yet
                assert!(!me.is_isolating(t + 1));
                assert!(!me.is_isolating(t + 2));
                assert!(!me.is_isolating(t + 3));
                assert!(!me.is_isolating(t + 4));
                assert!(me.is_isolating(t + 5)); // got results, is isolating
            }
        }
    }

    #[test]
    fn symptomatic_distribution_quantiles() {
        let mut sympt_dist = HashMap::new();
        let mut n_tot = 0;

        // Get 10k samples where symptoms are shown
        while n_tot < 10_000 {
            let mut me = Person::new("Olivia".to_string());
            me.expose(100);

            let infection = me.get_infection().unwrap();

            // only use symptomatic cases
            if let Some(s) = infection.symptomatic_date {
                n_tot += 1;
                let incubation_days = s - 100;
                let cur = sympt_dist
                    .get(&incubation_days)
                    .or_else(|| Some(&0))
                    .unwrap()
                    + 1;
                sympt_dist.insert(incubation_days, cur);
            }
        }

        // compute the cumulative distribution
        let mut cum_dist = Vec::new();
        let mut prev = 0;
        for d in 0..100 {
            let n = sympt_dist.get(&d).or_else(|| Some(&0)).unwrap();
            cum_dist.push((prev + n) as f64 / (n_tot as f64));
            prev += n;
        }
        assert!(prev == n_tot, "Expected prev {} == n_tot {}", prev, n_tot);

        let percentiles: Vec<(f64, f64)> = vec![
            (02.5, 1.92),
            (05., 2.24),
            (10., 2.69),
            (25., 3.64),
            (50., 5.10),
            (75., 7.15),
            (90., 9.69),
            (95., 11.60),
            (97.5, 13.60),
        ];

        // because we're dealing with integer days, make sure that the percentiles fall on the
        // right day
        for (p, expected) in percentiles {
            let above = expected.round() as usize;
            let below = above - 1;
            assert!(
                cum_dist[below] < (p / 100.) + 0.01,
                "Expected cum_dist[{}] {:.3} to be <{}",
                below,
                cum_dist[below],
                p / 100.
            );
            assert!(
                cum_dist[above] > (p / 100.) - 0.01,
                "Expected cum_dist[{}] {:.3} to be >{}",
                below,
                cum_dist[below],
                p / 100.
            );
        }
    }

    #[test]
    fn gen_phase_fn_test() {
        let phase_fn = gen_phase_fn(16, 5, 16, 5, 0);
        for day in 0..100 {
            assert_eq!(
                phase_fn(day),
                phase(day),
                "Day {}: gen says {:?}, should be {:?}",
                day,
                phase_fn(day),
                phase(day),
            );
        }
    }

    #[test]
    fn gen_phase_fn_test_hardcoded() {
        let alt_fn = gen_phase_fn(1, 0, 1, 0, 0);
        for day in 0..100 {
            if day % 2 == 0 {
                assert_eq!(
                    alt_fn(day),
                    Phase::A,
                    "Day {}: gen says {:?} should be {:?}",
                    day,
                    alt_fn(day),
                    Phase::A
                );
            } else {
                assert_eq!(
                    alt_fn(day),
                    Phase::C,
                    "Day {}: gen says {:?} should be {:?}",
                    day,
                    alt_fn(day),
                    Phase::C
                );
            }
        }

        let none_fn_1 = gen_phase_fn(0, 12, 0, 0, 0);
        let none_fn_2 = gen_phase_fn(0, 0, 0, 1, 0);
        let none_fn_3 = gen_phase_fn(0, 2, 0, 6, 0);
        for day in 0..100 {
            assert_eq!(
                none_fn_1(day),
                Phase::Isolate,
                "Day {}: gen_1 {:?} should be isolated",
                day,
                alt_fn(day),
            );
            assert_eq!(
                none_fn_2(day),
                Phase::Isolate,
                "Day {}: gen_2 {:?} should be isolated",
                day,
                alt_fn(day),
            );
            assert_eq!(
                none_fn_3(day),
                Phase::Isolate,
                "Day {}: gen_3 {:?} should be isolated",
                day,
                alt_fn(day),
            );
        }

        let always_a = gen_phase_fn(5, 0, 0, 0, 0);
        let always_c = gen_phase_fn(0, 0, 9, 0, 0);
        for day in 0..100 {
            assert_eq!(always_a(day), Phase::A);
            assert_eq!(always_c(day), Phase::C);
        }
    }
}
