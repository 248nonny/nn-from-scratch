
mod math;

use math::*;
use rand::rng;
use rand_distr::{Normal, Distribution};

pub struct NNData {
    pub data: Vec<u8>,
    pub label: usize,
}

#[derive(Debug)]
pub struct NeuralNet {
    weights: Vec<Matrix>,
    learning_rate: f32,
}

impl NeuralNet {
    /// net_structure is a Vector (so array, slice, or Vec) which describes the number of layers
    /// of neurons as well as how many neurons there are in a layer.
    ///
    /// The first value in net_structure is the dimension of the input values;
    /// The second value is the number of neurons in the first layer; ...(etc)...;
    /// the last value is the number of neurons in the output layer.
    pub fn new() -> Self {

        // the structure of the network for the MNIST dataset; input layer
        // can take in an image, middle layer for processing, output layer has
        // one node per possible label.
        let net_structure = vec![28 * 28, 100, 10];

        let mut weights: Vec<Matrix> = Vec::new();

        let num_layers = net_structure.size();

        weights.reserve_exact(num_layers);

        let mut net_structure_iter = net_structure.elements();

        // need at least input and output, so this should work.
        let mut n = net_structure_iter.next().unwrap();
        let mut m = net_structure_iter.next().unwrap();

        loop {
            weights.push(Matrix::new(m, n));
            
            n = m;

            if let Some(next) = net_structure_iter.next() {
                m = next;
            } else {
                break;
            }
        }

        //println!("{:?}", weights);


        NeuralNet {
            weights,
            learning_rate: 0.06,
        }
    }

    pub fn set_learning_rate(&mut self, rate: f32) {
        self.learning_rate = rate;
    }

    pub fn populate_random_weights(&mut self) {
        let mut rng = rng();
        for matrix in &mut self.weights {
            let bound = (matrix.m() as f32).sqrt();
            let normal = Normal::new(0.0, 1.0/bound).unwrap();
            matrix.apply_fn(|x| *x = normal.sample(&mut rng));
        }
    }

    // feeds value through neural network, returns output at each layer.
    pub fn nn_process_forward<V>(&self, input: V) -> Vec<Vec<F>>
    where V: Vector<F> {
        //println!("feeding a value into the network: {:?}",input);
        let mut values: Vec<Vec<F>> = Vec::new();
        values.reserve_exact(self.weights.len() + 1);

        values.push(input.elements().collect());

        for layer in &self.weights {
            //println!("\n\nThis layer has {} nodes.", layer.n());
            values.push((layer * values.last().unwrap()).iter().map(|&x| sigmoid(x)).collect());
        }

        values
    }

    pub fn image_to_prediction<V>(&self, input: V) -> Vec<F> 
    where V: Vector<F> {
        self.nn_process_forward(input).last().unwrap().clone()
    }

    // For stochastic gradient descent, uses one data point at a time.
    // Returns summed error.
    pub fn train_one(&mut self, data_point: &NNData) -> f32 {
        // convert data to float inputs, shifting and scaling slightly:
        let input_data: Vec<f32> = scale_and_normalize_data(&data_point.data);

        // create target value:
        let mut target: Vec<f32> = vec![0.01;10];
        target[data_point.label] = 0.99;
        let target = target; // no longer mutable!


        // feed data through network:
        let neuron_values = self.nn_process_forward(input_data);
        let output = &neuron_values.last().unwrap();

        //println!("\n\ntarget: {:?}\nactual: {:?}", target, output);

        // get output error:
        let mut error = vec![0.0;10];
        for i in 0..10 {
            error[i] = output[i] - target[i];
        }

        let scalar_error: f32 = error.iter().fold(0.0, |sum, x| sum + x.abs());

        // iterate through layers, backpropagating the error and then
        // adjusting weights according to the gradient and learning rate:
        for count in 0..self.weights.len() { 
            let layer = self.weights.len() - 1 - count; // iterate in reverse order.

            let layer_weight_matrix = &mut (self.weights[layer]); // get layer matrix.


            //println!("layer weight matrix dims: m = {}, n = {}",layer_weight_matrix.m(),layer_weight_matrix.n());
            //println!("neuron layers: {}", neuron_values.len());
            //println!("error length: {}", error.len());
            // compute gradient, update terms.
            for i in 0..layer_weight_matrix.m() {

                let row = layer_weight_matrix.get_mut_row_slice(i);

                let sigmoid_value = sigmoid(dot(row, &neuron_values[layer]).unwrap());
                                                                                // no -1 because neuron_values
                                                                                // includes input layer; we
                                                                                // are using the previous
                                                                                // layer's neuron values for
                                                                                // this.

                // adjust weight based on gradient.
                for j in 0..row.len() {
                    row[j] += - self.learning_rate * 2.0 * error[i] * sigmoid_value * (1.0 - sigmoid_value) * neuron_values[layer][j]
                }
            }

            error = &layer_weight_matrix.to_transpose() * &error; // backpropagation baby!
        }

        scalar_error

    }
}

pub fn scale_and_normalize_data (data: &Vec<u8>) -> Vec<f32> {
    data.iter().map(|x| *x as f32 / 255.0 * 0.98 + 0.01).collect()
}
