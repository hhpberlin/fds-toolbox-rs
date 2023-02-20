use super::Simulation;

#[test]
fn parses_successfully() {
    let input = include_str!("../../../../demo-house/DemoHaus2.smv");
    let sim = Simulation::parse(input.lines());
    assert!(sim.is_ok());
}