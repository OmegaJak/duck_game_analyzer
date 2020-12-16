pub trait IterExt<T>: Iterator<Item = T> {
	fn filter_count<P: Fn(&Self::Item) -> bool>(self, predicate: P) -> usize where Self: Sized
	{
		// Took most of this structure from std src Iterator::count
		let add1 = |count: usize, item: T| {
			if predicate(&item) {
				return std::ops::Add::add(count, 1)
			}

			count
		};
		
		self.fold(0, add1)
	}
}

impl<K, T: Iterator<Item = K>> IterExt<K> for T {
}