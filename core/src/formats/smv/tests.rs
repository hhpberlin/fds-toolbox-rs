// use ariadne::Report;

use super::Simulation;

#[test]
fn parses_successfully() {
    let input = include_str!("../../../../demo-house/DemoHaus2.smv");
    let sim = Simulation::parse(input);
    // Report::build
    let _sim = sim
        .map_err(|x| miette::Report::new(x).with_source_code(input))
        .unwrap();
}
