pub struct HeatmapPlot {

}

pub fn heatmap<Message>(data: Array2<f64>) -> impl Widget<Message> {

}



/// FizzBuzz implementation that is generic over the number of iterations.
pub fn fizzbuzz<N: Unsigned>(n: N) -> impl Iterator<Item = String> {
    (0..n.to_usize())
        .map(|i| {
            let i = i + 1;
            if i % 15 == 0 {
                "FizzBuzz".to_string()
            } else if i % 3 == 0 {
                "Fizz".to_string()
            } else if i % 5 == 0 {
                "Buzz".to_string()
            } else {
                i.to_string()
            }
        })
}