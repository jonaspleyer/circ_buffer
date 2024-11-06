//! A const-sized Ring-Buffer datastructure.
//!
//! The crate is `no_std`.
//! It uses elements from the standard library for testing purposes but does not rely on them for
//! internal implementation details.
//!
//! # Example
//!
//! ```
//! use circ_buffer::RingBuffer;
//!
//! let mut ring_buffer = RingBuffer::<_, 5>::new();
//! ring_buffer.push("Aurea prima");
//! ring_buffer.push("sata est");
//! ring_buffer.push("aetas, quae");
//! ring_buffer.push("vindice nullo");
//! ring_buffer.push("sponte sua,");
//! ring_buffer.push("sine lege fidem");
//! ring_buffer.push("rectumque colebat.");
//!
//! assert_eq!(ring_buffer[0], "aetas, quae");
//! assert_eq!(ring_buffer[1], "vindice nullo");
//! assert_eq!(ring_buffer[2], "sponte sua,");
//! assert_eq!(ring_buffer[3], "sine lege fidem");
//! assert_eq!(ring_buffer[4], "rectumque colebat.");
//! ```
//!
//! # Features
//! - [serde](https://serde.rs/) allows for deserialization of the RingBuffer

#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A ring Buffer with constant size.
/// Makes use of a fixed-size array internally.
/// ```
/// # use circ_buffer::*;
/// let mut circ_buffer = RingBuffer::<i64, 4>::default();
///
/// // These entries will be made into free space in the buffer.
/// circ_buffer.push(1_i64);
/// circ_buffer.push(2_i64);
/// circ_buffer.push(3_i64);
/// circ_buffer.push(4_i64);
///
/// // Now it will start truncating initial entries.
/// circ_buffer.push(5_i64);
/// circ_buffer.push(6_i64);
/// circ_buffer.push(7_i64);
///
/// let mut elements = circ_buffer.iter();
/// assert_eq!(elements.next(), Some(&4));
/// assert_eq!(elements.next(), Some(&5));
/// assert_eq!(elements.next(), Some(&6));
/// assert_eq!(elements.next(), Some(&7));
/// ```
#[derive(Clone, Debug)]
pub struct RingBuffer<T, const N: usize>(ItemStorage<T, N>);

/// Iterator of the [RingBuffer] struct.
///
/// This iterator does not necessarily contain `N` elements.
/// It depends on how many entries have been added previously.
///
/// ```
/// # use circ_buffer::*;
/// let mut circ_buffer = RingBuffer::<usize, 4>::default();
/// circ_buffer.push(1);
/// circ_buffer.push(33);
/// let elements = circ_buffer.iter().collect::<Vec<_>>();
/// assert_eq!(elements.len(), 2);
/// assert_eq!(elements[0], &1);
/// assert_eq!(elements[1], &33);
///
/// circ_buffer.push(4);
/// circ_buffer.push(5);
/// circ_buffer.push(6);
/// let elements = circ_buffer.iter().collect::<Vec<_>>();
/// assert_eq!(elements.len(), 4);
/// assert_eq!(elements, vec![&33, &4, &5, &6]);
/// ```
pub struct RingBufferIter<T, const N: usize>(ItemStorage<T, N>);

#[derive(Debug)]
struct ItemStorage<T, const N: usize> {
    items: [core::mem::MaybeUninit<T>; N],
    size: usize,
    first: usize,
}

impl<T, const N: usize> Iterator for RingBufferIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.0.size == 0 {
            return None;
        }
        let index = self.0.first;
        self.0.first = (self.0.first + 1) % N;
        self.0.size -= 1;
        Some(unsafe { self.0.items[index].assume_init_read() })
    }
}

/// Produced by the [iter](RingBuffer::iter) method of the [RingBuffer].
///
/// This iterates over references of the [RingBuffer].
pub struct RingBufferIterRef<'a, T, const N: usize> {
    items: &'a [core::mem::MaybeUninit<T>; N],
    size: usize,
    first: usize,
}

impl<'a, T, const N: usize> Iterator for RingBufferIterRef<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            return None;
        }
        let index = self.first;
        self.first = (self.first + 1) % N;
        self.size -= 1;
        Some(unsafe { self.items[index].assume_init_ref() })
    }
}

impl<T, const N: usize> IntoIterator for RingBuffer<T, N> {
    type Item = T;
    type IntoIter = RingBufferIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIter(self.0)
    }
}

impl<T, const N: usize> Clone for ItemStorage<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new_items: [core::mem::MaybeUninit<T>; N] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        for i in 0..self.size {
            let i = (self.first + i) % N;
            new_items[i].write(unsafe { self.items[i].assume_init_ref().clone() });
        }

        ItemStorage {
            items: new_items,
            first: self.first,
            size: self.size,
        }
    }
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Creates a new empty [RingBuffer]
    pub fn new() -> Self {
        Self(ItemStorage {
            items: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            first: 0,
            size: 0,
        })
    }

    /// Gets the current size of the [RingBuffer]
    pub fn get_size(&self) -> usize {
        self.0.size
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> core::ops::Index<usize> for RingBuffer<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T, const N: usize> Drop for ItemStorage<T, N> {
    fn drop(&mut self) {
        for n in 0..self.size {
            unsafe {
                self.items
                    .get_unchecked_mut((self.first + n) % N)
                    .assume_init_drop()
            };
        }
    }
}

impl<T, const N: usize> core::ops::Index<usize> for ItemStorage<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if index > self.size {
            panic!("index > size");
        }
        unsafe { core::mem::MaybeUninit::assume_init_ref(&self.items[(self.first + index) % N]) }
    }
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Append one element to the buffer.
    ///
    /// This will not grow the buffer but instead replace existing
    /// entries when the maximum size is reached.
    /// ```
    /// # use circ_buffer::*;
    /// let mut circ_buffer = RingBuffer::<f64, 5>::default();
    /// circ_buffer.push(1.0);
    /// circ_buffer.push(2.0);
    /// circ_buffer.push(3.0);
    /// circ_buffer.push(4.0);
    /// circ_buffer.push(5.0);
    /// // Now we begin to drop the first entry when pushing more values.
    /// circ_buffer.push(6.0);
    /// let elements = circ_buffer.iter().collect::<Vec<_>>();
    /// assert_eq!(elements, vec![&2.0, &3.0, &4.0, &5.0, &6.0])
    /// ```
    pub fn push(&mut self, new_item: T) {
        if N > 0 {
            let last = (self.0.first + self.0.size) % N;
            if self.0.size == N {
                unsafe { self.0.items.get_unchecked_mut(last).assume_init_drop() };
            }
            self.0.items[last].write(new_item);
            self.0.first = (self.0.first + self.0.size.div_euclid(N)) % N;
            self.0.size = N.min(self.0.size + 1);
        }
    }

    /// Iterate over references to elements of the RingBuffer.
    pub fn iter<'a>(&'a self) -> RingBufferIterRef<'a, T, N> {
        RingBufferIterRef {
            items: &self.0.items,
            first: self.0.first,
            size: self.0.size,
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<T, const N: usize> Serialize for RingBuffer<T, N>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut s = serializer.serialize_seq(Some(self.0.size))?;
        for element in self.iter() {
            s.serialize_element(element)?;
        }
        s.end()
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
struct FixedSizedRingBufferVisitor<T, const N: usize> {
    phantom: core::marker::PhantomData<T>,
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T, const N: usize> serde::de::Visitor<'de> for FixedSizedRingBufferVisitor<T, N>
where
    T: Deserialize<'de>,
{
    type Value = RingBuffer<T, N>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::write(
            formatter,
            core::format_args!(
                "{} or less values of the type {}",
                N,
                core::any::type_name::<T>()
            ),
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut elements = RingBuffer::new();
        let mut counter = 0;
        while let Some(element) = seq.next_element()? {
            if counter >= N {
                return Err(serde::de::Error::invalid_length(
                    N,
                    &"Too many values to unpack",
                ));
            }
            elements.push(element);
            counter += 1;
        }
        Ok(elements)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de, T, const N: usize> Deserialize<'de> for RingBuffer<T, N>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let elements = deserializer.deserialize_seq(FixedSizedRingBufferVisitor::<T, N> {
            phantom: core::marker::PhantomData,
        })?;
        let mut circ_buffer = RingBuffer::new();
        for element in elements.into_iter() {
            circ_buffer.push(element);
        }
        Ok(circ_buffer)
    }
}

#[cfg(test)]
mod test_circ_buffer {
    use super::*;

    #[test]
    fn test_pushing_full() {
        let mut circ_buffer = RingBuffer::<_, 12>::default();
        for i in 0..100 {
            circ_buffer.push(i);
            assert_eq!(circ_buffer.iter().last(), Some(&i));
            println!("{i}");
        }
    }

    #[test]
    fn test_pushing_overflow() {
        let mut circ_buffer = RingBuffer::<_, 4>::default();
        circ_buffer.push("ce");
        assert_eq!(circ_buffer.iter().collect::<Vec<_>>(), vec![&"ce"]);
        circ_buffer.push("ll");
        assert_eq!(circ_buffer.iter().collect::<Vec<_>>(), vec![&"ce", &"ll"]);
        circ_buffer.push("ular");
        assert_eq!(
            circ_buffer.iter().collect::<Vec<_>>(),
            vec![&"ce", &"ll", &"ular"]
        );
        circ_buffer.push(" ");
        assert_eq!(
            circ_buffer.iter().collect::<Vec<_>>(),
            vec![&"ce", &"ll", &"ular", &" "]
        );
        circ_buffer.push("raza");
        assert_eq!(
            circ_buffer.iter().collect::<Vec<_>>(),
            vec![&"ll", &"ular", &" ", &"raza"]
        );
    }

    #[test]
    fn test_clone_full() {
        let mut circ_buffer = RingBuffer::<_, 4>::default();
        circ_buffer.push(1_usize);
        circ_buffer.push(2);
        circ_buffer.push(3);
        circ_buffer.push(4);
        let new_circ_buffer = circ_buffer.clone();
        assert_eq!(
            circ_buffer.iter().collect::<Vec<_>>(),
            new_circ_buffer.iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_clone_partial() {
        let mut circ_buffer = RingBuffer::<_, 87>::default();
        for i in 0..100 {
            circ_buffer.push(i);
            let new_circ_buffer = circ_buffer.clone();
            assert_eq!(
                circ_buffer.iter().collect::<Vec<_>>(),
                new_circ_buffer.iter().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_drop_valid() {
        #[allow(unused)]
        struct S(i32);
        static COUNTER: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
        impl Drop for S {
            fn drop(&mut self) {
                COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        let mut circ_buffer = RingBuffer::<_, 234>::new();
        circ_buffer.push(S(5));
        circ_buffer.push(S(2));
        circ_buffer.push(S(11));
        circ_buffer.push(S(87));
        drop(circ_buffer);
        assert_eq!(COUNTER.load(std::sync::atomic::Ordering::Relaxed), 4);
    }

    #[test]
    fn test_miri_push_drop_valid() {
        let mut ring_buffer = RingBuffer::<String, 2>::new();
        ring_buffer.push(format!("some string"));
        ring_buffer.push(format!("this has something"));
        ring_buffer.push(format!("this should drop the first string"));
        ring_buffer.push(format!("this should drop the second string"));
    }

    #[test]
    fn test_miri_iter_valid() {
        let mut ring_buffer = RingBuffer::<Vec<u8>, 8>::new();
        ring_buffer.push(vec![1, 2, 3]);
        ring_buffer.push(vec![128, 00, 3]);
        ring_buffer.push(vec![87, 3, 0, 1]);
        ring_buffer.push(vec![36, 048, 5, 20, 40]);
        ring_buffer.push(vec![16, 40, 3, 5, 3]);
        for e in ring_buffer.iter() {
            println!("{:?}", e);
        }
    }

    #[test]
    fn test_empty_circ_buffer() {
        let mut ring_buffer = RingBuffer::<&str, 0>::new();
        ring_buffer.push("I am a dog");
        ring_buffer.push("I was a dog");
        for _ in ring_buffer.iter() {
            panic!("This should not be called since the loop is empty");
        }
    }

    #[cfg(feature = "serde")]
    mod serde {
        use crate::*;

        #[test]
        fn test_serialize_full() {
            let mut circ_buffer = RingBuffer::<_, 4>::default();
            circ_buffer.push(1_u128);
            circ_buffer.push(2);
            circ_buffer.push(55);
            circ_buffer.push(12999);

            let serialized = serde_json::to_string(&circ_buffer).unwrap();
            assert_eq!(serialized, "[1,2,55,12999]");
        }

        #[test]
        fn test_serialize_partially_filled() {
            let mut circ_buffer = RingBuffer::<_, 4>::default();
            circ_buffer.push(1_u128);
            circ_buffer.push(2);

            let serialized = serde_json::to_string(&circ_buffer).unwrap();
            assert_eq!(serialized, "[1,2]");
        }

        #[test]
        fn test_deserialize_full() {
            let circ_buffer_string = "[-3,2,1023,-112]";
            let circ_buffer: RingBuffer<i16, 4> =
                serde_json::de::from_str(circ_buffer_string).unwrap();
            assert_eq!(
                circ_buffer.iter().collect::<Vec<_>>(),
                vec![&-3, &2, &1023, &-112]
            );
        }

        #[test]
        fn test_deserialize_partially_filled() {
            for i in 0..50 {
                let circ_buffer_values: Vec<_> = (0..i).collect();
                let string = format!("{:?}", circ_buffer_values);
                let circ_buffer: RingBuffer<_, 100> = serde_json::de::from_str(&string).unwrap();
                assert_eq!(circ_buffer.iter().collect::<Vec<_>>(), circ_buffer_values);
            }
        }

        #[test]
        #[should_panic]
        fn test_deserialize_too_many_values() {
            let circ_buffer_values: Vec<_> = (0..11).collect();
            let string = format!("{:?}", circ_buffer_values);
            println!("{}", string);
            let _circ_buffer: RingBuffer<usize, 10> = serde_json::de::from_str(&string).unwrap();
        }
    }
}

#[allow(unused)]
#[doc(hidden)]
#[cfg(feature = "serde")]
mod test_derive_serde_circ_buffer {
    /// ```
    /// use serde::Serialize;
    /// use circ_buffer::*;
    /// #[derive(Serialize)]
    /// struct Something<T, const N: usize> {
    ///     circ_buffer: RingBuffer<T, N>,
    /// }
    /// ```
    fn derive_serialize() {}

    /// ```
    /// use serde::Deserialize;
    /// use circ_buffer::*;
    /// #[derive(Deserialize)]
    /// struct Something<T, const N: usize> {
    ///     circ_buffer: RingBuffer<T, N>,
    /// }
    /// ```
    fn derive_deserialize() {}

    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use circ_buffer::*;
    /// #[derive(Deserialize, Serialize)]
    /// struct Something<T, const N: usize> {
    ///     circ_buffer: RingBuffer<T, N>,
    /// }
    /// ```
    fn derive_serialize_deserialize() {}
}
