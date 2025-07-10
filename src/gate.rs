use crate::node::Node;

pub struct Gate {
    is_open: bool
}
impl Gate {
    pub fn new() -> Self {
        Self {
            is_open: false
        }
    }
}
impl<const C: usize, const N: usize> Node<C, N> for Gate {
    #[inline(always)]
    fn process(&mut self, inputs: &[crate::buffer::Frame<C, N>], output: &mut crate::buffer::Frame<C, N>) {
        if let Some(input) = inputs.get(0) {
            if self.is_open {
                *output = *input;
            }
            else {
                for buf in output.iter_mut() {
                    buf.fill(0.0);
                }
            }
        }
    }
}
