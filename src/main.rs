#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! ???

use rustagious::Person;
use std::collections::HashMap;

type Res = (bool, bool, bool);

fn main() {
    let res = run_n(100);
    let mut tot = 0;
    for (k, v) in &res {
        println!("{:?}: {:?}", v, k);
        tot += v;
    }
    println!("{} runs total", tot);
    println!("done");
}

fn run_n(n: u64) -> HashMap<Res, u64> {
    let mut res = HashMap::new();

    for day in 0..6 * 7 {
        for source in 0..3 {
            for _ in 0..n {
                let run = run_trial(day, source);
                let cur = res.get(&run).or_else(|| Some(&0)).unwrap() + 1;
                res.insert(run, cur);
            }
        }
    }
    res
}

enum Phase {
    A,
    Isolate,
    C,
}

fn phase(day: u64) -> Phase {
    let cycle_day = day % 6 * 7;
    if cycle_day <= 15 {
        Phase::A
    } else if cycle_day >= 21 && cycle_day <= 36 {
        Phase::C
    } else {
        Phase::Isolate
    }
}

/// Runs a single experiment
fn run_trial(moment: u64, who: u64) -> Res {
    let mut a = Person::new("A".to_string());
    let mut b = Person::new("B".to_string());
    let mut c = Person::new("C".to_string());

    let source = "alpha".to_string();
    match who {
        0 => a.expose(moment, source),
        1 => {
            b.expose(moment, source);
        }
        2 => c.expose(moment, source),
        _ => unreachable!(),
    }

    for day in 1..300 {
        if a.is_isolating(day) || b.is_isolating(day) || c.is_isolating(day) {
            break;
        }

        match day % 7 {
            1 | 4 => a.test(day),
            _ => {}
        }

        match phase(day) {
            Phase::A => b.interact(day, &mut a),
            Phase::C => b.interact(day, &mut c),
            Phase::Isolate => {}
        }
    }

    //a.health_summary();
    //b.health_summary();
    //c.health_summary();

    //println!("A: {:?}", a.get_infection());
    //println!("B: {:?}", b.get_infection());
    //println!("C: {:?}", c.get_infection());

    (a.was_sick(100), b.was_sick(100), c.was_sick(100))
}
