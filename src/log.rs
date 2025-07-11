use assert_no_alloc::permit_alloc;

use crate::node::{Node, Bang};

pub struct Log<const N: usize, const C: usize> {

}
impl<const N: usize, const C: usize> Log <N, C> {
}
impl<const N: usize, const C: usize>  Node<N,C> for Log<N, C>{
    fn handle_bang(&mut self, inputs: &[Bang], _: &mut Bang) {
        if let Some(bang) = inputs.get(0){
            match bang {
                Bang::Empty => (),
                _ => println!("bang!")
            }
            
        }
    }
}