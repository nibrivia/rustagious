#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! A crate to model contagion

use rand::distributions::{Bernoulli, Distribution};

type Time = u64;

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
        if self.infection.is_some() {
            return;
        }
        let d = Bernoulli::new(0.6).unwrap();
        let v = d.sample(&mut rand::thread_rng());
        let symptomatic_date = if v { Some(date + 5) } else { None };

        self.infection = Some(Infection {
            date,
            testable_date: date + 2,   // TODO
            contagious_date: date + 3, // TODO
            symptomatic_date,
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
            return infection.contagious_date <= date;
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
                self.is_isolating(99999)
            );
        } else {
            println!("{} healthy", self.name);
        }
    }
}

/// Returns the odds of a person being contagious `t` after infection
pub fn p_contagious(_t: Time) -> f64 {
    // TODO
    1.0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_infected_tested_isolating() {
        let mut me = Person::new("Olivia".to_string());
        assert_eq!(me.name, "Olivia".to_string());

        // Get sick
        me.expose(3, "MIT".to_string());

        // Don't know better yet, should *not* be isolating...
        assert!(!me.is_isolating(4));
        assert!(me.was_sick(4)); // ...but am sick (hidden state)

        // Get tested
        me.test(5);

        // Really should be isolating
        assert!(me.is_isolating(6));
        assert!(me.was_sick(6));
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
        sick_me.interact(3, &mut healthy_me);
        assert!(sick_me.was_sick(4)); // should still be sick
        assert!(healthy_me.was_sick(4)); // should *also* be sick
    }
}
