// use ariadne::Report;
use miette::{Diagnostic, SourceSpan};

use super::{Simulation, err::Error};

#[test]
fn parses_successfully() {
    let input = include_str!("../../../../demo-house/DemoHaus2.smv");
    let sim = Simulation::parse(input);
    // Report::build
    let sim = sim.map_err(|x| miette::Report::new(x).with_source_code(input)).unwrap();
}
