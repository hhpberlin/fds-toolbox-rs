// use ariadne::Report;

use super::Simulation;

#[test]
fn parses_successfully() {
    let input = include_str!("../../../../demo-house/DemoHaus2.smv");
    let sim = Simulation::parse(input.lines());
    // Report::build
    let sim = sim.unwrap();
}
