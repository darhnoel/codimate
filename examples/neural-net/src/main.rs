use codimate_example_neural_net::{
    explain, neural_net_algorithm, neural_net_motion, neural_net_view, NeuralNet, NeuralNetTiming,
};

fn main() {
    explain("Neural Net")
        .state(NeuralNet::new())
        .view(neural_net_view)
        .algorithm(neural_net_algorithm)
        .motion(neural_net_motion)
        .timing(NeuralNetTiming::default())
        .render("results/neural-net.mp4");
}
