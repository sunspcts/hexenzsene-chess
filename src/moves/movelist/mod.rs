mod pick;
mod score;
use super::Move;

// Implementing this as an array so it'll be stack allocated. Profiling showed a LOT of malloc calls in the movegen phase.
#[derive(Clone, Copy, Debug)]
pub struct MoveList {
    moves: [Move; 256],
    scores: [i16; 256],
    len: u8, // pointer essentially
}

impl MoveList {
    // write to pointer location, and increment pointer.
    pub fn push(&mut self, mv: Move) {
        self.moves[self.len as usize] = mv;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // doesn't zero out anything, just resets the pointer to zero.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    // based on the implementation for Vec, doesn't drop elements but simply moves them behind the pointer.
    // Since the pointer can't be manually incremented without overwriting whatever it sits at, the moved values are unreachable!
    // https://doc.rust-lang.org/src/alloc/vec/mod.rs.html#2478-2480

    // I don't actually use this anymore, but it's good to keep around.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Move) -> bool,
    {
        let original_len = self.len as usize;

        //index of position we swap into, always less than or equal to read.
        let mut write = 0;

        for read in 0..original_len {
            if f(&self.moves[read]) {
                // bool function we passed in
                if read != write {
                    //if it's in the right spot, we dont even need to swap them!
                    self.swap(read, write);
                }
                write += 1;
            }
        }

        self.len = write as u8; // the last element we kept is at index write, so that's the new length.
    }

    #[inline]
    pub fn swap(&mut self, i: usize, j: usize) {
        self.moves.swap(i, j);
        self.scores.swap(i, j);
    }
}

impl Default for MoveList {
    fn default() -> Self {
        MoveList {
            moves: [Move::new_from_raw(0); 256],
            scores: [0; 256],
            len: 0,
        }
    }
}

impl std::ops::Deref for MoveList {
    type Target = [Move];

    fn deref(&self) -> &Self::Target {
        &self.moves[..self.len as usize]
    }
}

impl std::ops::DerefMut for MoveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.moves[..self.len as usize]
    }
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = std::iter::Take<std::array::IntoIter<Move, 256>>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves.into_iter().take(self.len as usize)
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut MoveList {
    type Item = &'a mut Move;
    type IntoIter = std::slice::IterMut<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
