#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! A crate to model contagion

type Time = u64;

/// Struct representing an individual and keeping track of associated state
#[derive(Debug)]
pub struct Person {
    /// Person's name for easy referencing
    pub name: String,

    infection: Option<Infection>,
    tested_positive: bool,
}

/// Infection data
#[derive(Debug)]
pub struct Infection {
    date: Time,
    source: String,
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
    pub fn expose(self: &mut Self, date: Time, source: String, _p: f64) {
        // TODO use probabistic input
        self.infection = Some(Infection { date, source });
    }

    /// Runs a test on a person
    pub fn test(self: &mut Self) {
        self.tested_positive = self.infection.is_some();
    }

    /// Interacts two people
    pub fn interact(self: &mut Self, date: Time, other: &mut Self) {
        if other.is_contagious() {
            self.expose(date, "other".to_string(), 1.0); // TODO name
        }

        if self.is_contagious() {
            other.expose(date, "self".to_string(), 1.0); // TODO name
        }
    }

    /// Is this person able to infect others?
    pub fn is_contagious(self: &Self) -> bool {
        self.was_sick() // TODO
    }

    /// Is this person in a state where they should be isolating?
    pub fn is_isolating(self: &Self) -> bool {
        self.tested_positive
    }

    /// Has this person *ever* been infected?
    pub fn was_sick(self: &Self) -> bool {
        self.infection.is_some()
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
        me.expose(3, "MIT".to_string(), 1.0);

        // Don't know better yet, should *not* be isolating...
        assert!(!me.is_isolating());
        assert!(me.was_sick()); // ...but am sick (hidden state)

        // Get tested
        me.test();

        // Really should be isolating
        assert!(me.is_isolating());
        assert!(me.was_sick());
    }

    #[test]
    fn interaction() {
        let mut healthy_me = Person::new("Olivia Healthy".to_string());
        let mut sick_me = Person::new("Olivia Sick".to_string());
        assert!(!healthy_me.was_sick());
        assert!(!sick_me.was_sick());

        // Sick me is sick
        sick_me.expose(1, "MIT".to_string(), 1.0);
        assert!(sick_me.was_sick());

        // Interact
        sick_me.interact(3, &mut healthy_me);
        assert!(sick_me.was_sick()); // should still be sick
        assert!(healthy_me.was_sick()); // should *also* be sick
    }
}
