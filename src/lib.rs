#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! A crate to model contagion

use rand::distributions::{Bernoulli, Distribution};
use rand_distr::LogNormal;

type Time = u64;

const SYMPTOMATIC_MEAN: f64 = 5.058146;
const SYMPTOMATIC_SD: f64 = 1.518921;

/// Struct representing an individual and keeping track of associated state
#[derive(Debug)]
pub struct Person {
    /// Person's name for easy referencing
    pub name: String,

    infection: Option<Infection>,
    tested_positive: bool, // TODO tested date
}

/// Infection data
#[derive(Debug)]
pub struct Infection {
    date: Time,

    testable_date: Time,
    contagious_date: Time,
    recovery_date: Time,

    symptomatic_date: Option<Time>,

    // TODO hacky
    /// Infection's original source
    pub source: String,
}

impl Person {
    /// Creates a new person
    pub fn new(name: String) -> Person {
        Person {
            name,
            infection: None,
            tested_positive: false,
        }
    }

    /// Exposes a person to a source on a given date
    pub fn expose(self: &mut Self, date: Time, source: String) {
        // already infected, let's not do this again...
        if self.infection.is_some() {
            return;
        }

        let mut rng = rand::thread_rng();

        // Symptomatic date, everything is computed in reference to that
        // mean 2, standard deviation 3
        let log_normal = LogNormal::new(SYMPTOMATIC_MEAN.ln(), SYMPTOMATIC_SD.ln()).unwrap();
        let symptomatic_date: Time = date + log_normal.sample(&mut rng).round() as Time;
        let testable_date = symptomatic_date - 2;
        let contagious_date = symptomatic_date - 2;
        let recovery_date = symptomatic_date + 14;

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
            source,
        });
    }

    /// Runs a test on a person
    pub fn test(self: &mut Self, date: Time) {
        if let Some(infection) = &self.infection {
            self.tested_positive = infection.testable_date <= date;
        }
    }

    /// Interacts two people
    pub fn interact(self: &mut Self, date: Time, other: &mut Self) {
        if other.is_contagious(date) {
            self.expose(
                date,
                other.get_infection().as_ref().unwrap().source.to_string(),
            );
        }

        if self.is_contagious(date) {
            other.expose(date, self.infection.as_ref().unwrap().source.to_string());
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
        if self.tested_positive {
            return true;
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

    /// Has this person *ever* been infected?
    pub fn was_sick(self: &Self, date: Time) -> bool {
        println!("{} {:?}", self.name, self.infection);
        if let Some(infection) = &self.infection {
            return infection.date <= date;
        }
        false
    }

    /// Prints a health summary to stdout
    pub fn health_summary(self: &Self) {
        if let Some(infection) = &self.infection {
            println!(
                "{} infected on {} by {} and is isolating? {}",
                self.name,
                infection.date,
                infection.source,
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

    #[test]
    fn single_infected_tested_isolating() {
        let mut me = Person::new("Olivia".to_string());
        assert_eq!(me.name, "Olivia".to_string());

        // Get sick
        me.expose(2, "MIT".to_string());

        // Don't know better yet, should *not* be isolating...
        assert!(!me.is_isolating(2));
        assert!(me.was_sick(4)); // ...but am sick (hidden state)

        // Get tested
        me.test(12);

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
        b.expose(10, "MIT".to_string());
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
        sick_me.expose(0, "MIT".to_string());
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
}
