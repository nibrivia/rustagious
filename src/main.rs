#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! ???

use rayon::prelude::*;
use rustagious::{gen_phase_fn, Person, Phase};
use std::collections::HashMap;

//type Res = (String, u64, String, u64, String, u64);
type Res = (u64, u64);

fn main() {
    println!("a, ac, c, ca, offset, b_test, n, tot_days_unaware, n_infected");
    let n = 10_000;
    //for (a, ac, c, ca) in gen_phases() {
    #[allow(clippy::print_literal)]
    gen_phases()
        .par_iter()
        .map(move |phase_desc| {
            let (a, ac, c, ca, offset) = *phase_desc;
            let cycle_len = a + ac + c + ca;

            let phase_fn = &gen_phase_fn(a, ac, c, ca, offset);
            for d in 0..cycle_len {
                // isolate always, we keep that one
                if a == 0 && c == 0 {
                    continue;
                }

                // if it's the weekend
                if d % 7 == 5 || d % 7 == 6 {
                    // no isolation on the weekend...
                    if phase_fn(d) == Phase::Isolate {
                        return;
                    }
                }
            }

            let outcomes = run_n(n, None, cycle_len, phase_fn);
            for (res, n) in outcomes {
                println!(
                    "{}, {}, {}, {}, {}, {}, {}, {}, {}",
                    a, ac, c, ca, offset, "NA", n, res.0, res.1
                );
            }

            /*
            for b_test in 0..cycle_len {
                let outcomes = run_n(n, Some(b_test), cycle_len, phase_fn);
                for (res, n) in outcomes {
                    println!(
                        "{}, {}, {}, {}, {}, {}, {}, {}, {}",
                        a, ac, c, ca, offset, b_test, n, res.0, res.1
                    );
                }
            }
            */
        })
        .collect::<()>();
}

fn gen_phases() -> Vec<(u64, u64, u64, u64, u64)> {
    let mut phases = Vec::new();

    phases.push((1, 0, 1, 0, 0)); // alternating
    for offset in 0..7 {
        phases.push((16, 5, 16, 5, offset)); // current and variations
    }

    phases.push((14, 0, 0, 0, 0)); // always A-B
    phases.push((0, 0, 14, 0, 0)); // always C-B

    for duration in (28..29).step_by(14) {
        phases.push((0, duration / 2, 0, duration / 2, 0)); // isolating completly

        for tot_isolation_days in (0..11).step_by(2) {
            let phase_duration = (duration - tot_isolation_days) / 2;
            if phase_duration < 5 {
                continue;
            }
            for ac in 0..(tot_isolation_days + 1) {
                // no isolating on weekends
                if ac > 5 || tot_isolation_days - ac > 5 {
                    continue;
                }
                for offset in 0..7 {
                    phases.push((
                        phase_duration,
                        ac,
                        phase_duration,
                        tot_isolation_days - ac,
                        offset,
                    ));
                }
            }
        }
    }
    phases
}

fn run_n(
    n: u64,
    b_test: Option<u64>,
    cycle_len: u64,
    phase_fn: &dyn Fn(u64) -> Phase,
) -> HashMap<Res, u64> {
    let mut res = HashMap::new();

    for day in 0..cycle_len {
        for source in 1..4 {
            for _ in 0..n {
                let run = run_trial(day, source, b_test, cycle_len, phase_fn);
                let cur = res.get(&run).or_else(|| Some(&0)).unwrap() + 1;
                res.insert(run, cur);
            }
        }
    }
    res
}

/// Runs a single experiment
fn run_trial(
    moment: u64,
    who: u64,
    b_test: Option<u64>,
    cycle_len: u64,
    phase_fn: &dyn Fn(u64) -> Phase,
) -> Res {
    let mut a = Person::new("".to_string());
    let mut b = Person::new("".to_string());
    let mut c = Person::new("".to_string());

    match who {
        1 => a.expose(moment), //, format!("A.{:}", moment)),
        2 => b.expose(moment), //, format!("B.{:}", moment)),
        3 => c.expose(moment), //, format!("C.{:}", moment)),
        _ => unreachable!(),
    }

    let mut max_day = 0;
    for day in moment..300 {
        max_day = day;

        // someone is isolating, stop sim
        if a.is_isolating(day) || b.is_isolating(day) || c.is_isolating(day) {
            break;
        }

        // everyone has recovered, stop
        if a.has_recovered(day) && b.has_recovered(day) && c.has_recovered(day) {
            break;
        }

        match day % 7 {
            1 | 4 => {
                a.test(day, 1);
            }
            0 | 3 => {}
            _ => {}
        }

        if b_test.is_some() && day % cycle_len == b_test.unwrap() {
            b.test(day, 2);
        }

        match phase_fn(day) {
            Phase::A => b.interact(day, &mut a),
            Phase::C => {
                // a.interact(day, &mut z);
                b.interact(day, &mut c);
            }
            Phase::Isolate => {}
        }
    }

    (
        a.days_unaware(max_day) + b.days_unaware(max_day) + c.days_unaware(max_day),
        a.was_sick(max_day) as u64 + b.was_sick(max_day) as u64 + c.was_sick(max_day) as u64,
    )
}

/*
fn get_source(p: &Person) -> String {
    if let Some(infection) = p.get_infection() {
        infection.source.to_string()
    } else {
        "-.00".to_string()
    }
}
*/
