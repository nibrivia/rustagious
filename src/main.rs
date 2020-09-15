#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

//! ???

use rustagious::{phase, Person, Phase};
use std::collections::HashMap;

type Res = (String, String, String);

fn main() {
    let res = run_n(100_000);
    let mut ord = Vec::new();

    //let mut tot = 0;
    for (k, v) in res {
        ord.push((v, k));
        //tot += v;
    }
    ord.sort();

    for (v, k) in &ord {
        println!(
            "{:.0} {} {} {}",
            //(500. * (*v as f64)) / tot as f64,
            v,
            k.0,
            k.1,
            k.2,
            //k.3,
            //k.4
        );
    }
    //printerr!("done");
}

fn run_n(n: u64) -> HashMap<Res, u64> {
    let mut res = HashMap::new();

    for day in 0..6 * 7 {
        for source in 1..4 {
            for _ in 0..n {
                let run = run_trial(day, source);
                let cur = res.get(&run).or_else(|| Some(&0)).unwrap() + 1;
                res.insert(run, cur);
            }
        }
    }
    res
}

/// Runs a single experiment
fn run_trial(moment: u64, who: u64) -> Res {
    //let mut z = Person::new("0".to_string());
    let mut a = Person::new("A".to_string());
    let mut b = Person::new("B".to_string());
    let mut c = Person::new("C".to_string());
    //let mut d = Person::new("D".to_string());

    match who {
        //0 => z.expose(moment, format!("Z.{:}", moment)),
        1 => a.expose(moment, format!("A.{:}", moment)),
        2 => b.expose(moment, format!("B.{:}", moment)),
        3 => c.expose(moment, format!("C.{:}", moment)),
        //4 => d.expose(moment, format!("D.{:}", moment)),
        _ => unreachable!(),
    }

    for day in 0..300 {
        if
        //z.is_isolating(day) ||
        a.is_isolating(day) || b.is_isolating(day) || c.is_isolating(day)
        //|| d.is_isolating(day)
        {
            break;
        }

        match day % 7 {
            2 | 5 => {
                a.test(day);
            }
            3 | 6 => {
                //z.test(day);
            }
            _ => {}
        }

        //c.interact(day, &mut d);
        match phase(day) {
            Phase::A => b.interact(day, &mut a),
            Phase::C => {
                // a.interact(day, &mut z);
                b.interact(day, &mut c);
            }
            Phase::Isolate => {}
        }
    }

    //a.health_summary();
    //b.health_summary();
    //c.health_summary();

    //println!("A: {:?}", a.get_infection());
    //println!("B: {:?}", b.get_infection());
    //println!("C: {:?}", c.get_infection());

    (
        //get_source(&z),
        get_source(&a),
        get_source(&b),
        get_source(&c),
        //get_source(&d),
    )
}

fn get_source(p: &Person) -> String {
    if let Some(infection) = p.get_infection() {
        infection.source.to_string()
    } else {
        "-.00".to_string()
    }
}
