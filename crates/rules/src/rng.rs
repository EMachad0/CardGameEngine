//! SplitMix64, a tiny deterministic PRNG.
//!
//! Same seed, same sequence, on every machine and every run, so a replay from
//! the same seed reproduces every random outcome.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// An integer in `0..n`. Panics if `n == 0`.
    /// Modulo biases it slightly toward low values, which doesn't matter for small `n`.
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "Rng::below(0)");
        (self.next_u64() % n as u64) as usize
    }

    /// One of `values`, at random. With none, `None`, and the stream doesn't move.
    pub(crate) fn choose<'v, T>(&mut self, values: &'v [T]) -> Option<&'v T> {
        (!values.is_empty()).then(|| &values[self.below(values.len())])
    }

    /// Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn matches_reference_splitmix64_output() {
        // First output of the reference SplitMix64 for seed 0.
        assert_eq!(Rng::new(0).next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn choosing_from_nothing_gives_nothing() {
        assert_eq!(Rng::new(3).choose::<u32>(&[]), None);
    }

    #[test]
    fn choosing_from_nothing_leaves_the_stream_as_it_was() {
        let mut rng = Rng::new(3);

        rng.choose::<u32>(&[]);

        assert_eq!(rng, Rng::new(3));
    }

    #[test]
    fn a_choice_is_one_of_the_values() {
        let values = [10, 20, 30];
        for seed in 0..100 {
            let chosen = Rng::new(seed).choose(&values).copied();

            assert!(
                chosen.is_some_and(|value| values.contains(&value)),
                "seed {seed} chose {chosen:?}",
            );
        }
    }

    #[test]
    fn a_choice_takes_one_draw_from_the_stream() {
        for seed in 0..100 {
            let mut chooser = Rng::new(seed);
            let mut drawer = Rng::new(seed);

            chooser.choose(&[10, 20, 30]);
            drawer.next_u64();

            assert_eq!(chooser, drawer, "seed {seed}");
        }
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut v: Vec<u32> = (0..20).collect();
        Rng::new(7).shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>());
    }
}
