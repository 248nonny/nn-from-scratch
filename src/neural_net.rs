
mod math;

use math::*;
use rand::{rngs::ThreadRng, rng, Rng};
use rand_distr::{Normal, Distribution};

struct NNData<T, U, const N: usize> {
    data: [T; N],
    label: U,
}

#[derive(Debug)]
pub struct NeuralNet {
    weights: Vec<Matrix>,
    rng: ThreadRng,
}

impl NeuralNet {
    /// net_structure is a Vector (so array, slice, or Vec) which describes the number of layers
    /// of neurons as well as how many neurons there are in a layer.
    ///
    /// The first value in net_structure is the dimension of the input values;
    /// The second value is the number of neurons in the first layer; ...(etc)...;
    /// the last value is the number of neurons in the output layer.
    pub fn new<T>(net_structure: T) -> Self
    where T: Vector<usize> {

        let mut weights: Vec<Matrix> = Vec::new();

        let num_layers = net_structure.size();

        weights.reserve_exact(num_layers);

        let mut net_structure_iter = net_structure.elements();

        // need at least input and output, so this should work.
        let mut m = net_structure_iter.next().unwrap();
        let mut n = net_structure_iter.next().unwrap();

        loop {
            weights.push(Matrix::new(m, n));
            
            m = n;

            if let Some(next) = net_structure_iter.next() {
                n = next;
            } else {
                break;
            }
        }

        //println!("{:?}", weights);


        NeuralNet {
            weights,
            rng: rng(),
        }
    }

    pub fn populate_random_weights(&mut self) {
        for matrix in &mut self.weights {
            let bound = (matrix.m() as f32).sqrt();
            let normal = Normal::new(0.0, 1.0/bound).unwrap();
            matrix.apply_fn(|x| *x = normal.sample(&mut self.rng));
        }
    }

    pub fn nn_process_forward<V,T>(&self, input: V) -> Vec<F>
    where V: Vector<F> {
        let mut current_value: Vec<F> = input.elements().collect();
        for layer in &self.weights {
            current_value = (layer * &current_value).iter().map(|x| sigmoid(*x)).collect();
        }

        current_value
    }
}
