pub trait Shuffle {
    fn shuffle(&mut self, seed: u64, nr_of_iterations: usize);
}

impl<T> Shuffle for Vec<T>
where
    T: Clone,
{
    fn shuffle(&mut self, seed: u64, nr_of_iteration: usize) {
        let mut state = seed;
        let nr_of_chunks = 5;
        let chunk_size = self.len() / nr_of_chunks;
        let mut buffer: Vec<T> = self.clone();

        for _ in 0..nr_of_iteration {
            let mut groups: Vec<&mut [T]> = self.chunks_mut(chunk_size).collect();
            for i in 0..groups.len() {
                //LGC algo (https://en.wikipedia.org/wiki/Linear_congruential_generator)
                state = state.wrapping_mul(134775813).wrapping_add(1) % 4294967296;
                let j = (state as usize) % groups.len();
                groups.swap(i, j);
            }

            let mut index = 0usize;
            for group in groups {
                let size = group.len();
                buffer[index..index + size].swap_with_slice(group);
                index += size
            }
            self.swap_with_slice(buffer.as_mut_slice());

            //Guard against small arrays
            let rotate = 1 % self.len();
            self.rotate_left(rotate);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shuffle_10_vec() {
        let mut input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        input.shuffle(123, 1000);
        assert_eq!(input.len(), 10)
    }
    #[test]
    fn shuffle_9_vec() {
        let mut input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        input.shuffle(123, 1000);
        assert_eq!(input.len(), 9)
    }
}
