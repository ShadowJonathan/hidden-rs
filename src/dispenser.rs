//! An emitter of choices that shuffles its order internally afterwards.
//!
//! The recommended way of using this module is with it's [`Dispenser`](Dispenser) struct:
//!
//! ```
//! use hidden::dispenser::Dispenser;
//!
//! let elements = vec!['a', 'b', 'c', 'd', 'e', 'f'];
//! let mut dispenser = Dispenser::new(elements.len());
//!
//! let za_hando = dispenser.make_hand(&elements).expect("created with the same elements slice");
//! let star_finger = dispenser.make_hand(&elements).expect("created with the same elements slice");
//!
//! let hando_choice = za_hando.choose(1).unwrap();
//! let star_choice = star_finger.choose(1).unwrap();
//!
//! // At this point, it's possible that hando_choice and star_choice are or aren't the same,
//! // by design of random shuffling.
//!
//! // However, choosing from the same index, per hand, *is* guaranteed to be the same.
//! assert_eq!(hando_choice, za_hando.choose(1).unwrap());
//! assert_eq!(star_choice, star_finger.choose(1).unwrap());
//! ```
//!
//! Upon every call to [`make_hand`](Dispenser::make_hand), and upon [creation](Dispenser::new),
//! the dispenser shuffles it's internal state, so that it becomes an internal state it may
//! "dispense", and then change, which stays that way until the next "dispensing".
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// A struct that holds a hidden variable, dispenses [`Hand`s](Hand) with a lock on a
/// state, and shuffles afterward.
#[derive(Debug)]
pub struct Dispenser {
    seq: Vec<usize>,
    rng: ThreadRng,
}

impl Dispenser {
    /// Creates a new [`Dispenser`](Dispenser), initializing it with choices for a slice of a given
    /// `len`.
    pub fn new(len: usize) -> Self {
        let mut disp = Self {
            seq: (0..len).collect(),
            rng: thread_rng(),
        };
        disp.shuffle();
        disp
    }

    /// Returns the effective `len` argument given to [`new`](Dispenser::new) for this object.
    pub fn len(&self) -> usize {
        self.seq.len()
    }

    /// Creates a [`Hand`](Hand) from `deck` and an internal variable, this shuffles the variable afterwards.
    ///
    /// This makes first sure that all possible choices are possible for `deck`, it does this by
    /// checking the length of `deck`, and effectively comparing it to the `len` given when the
    /// [`Dispenser`](Dispenser) [was created](Dispenser::new).
    ///
    /// This function returns [`None`](None) when it's possible that elements in `deck` cannot be chosen, or
    /// there are more possible choices than that `deck` has. (`deck.len() != len`)
    ///
    /// This function returns [`Some`](Some) when checks pass, and all choices has a corresponding element
    /// in `deck`.
    pub fn make_hand<'h, T>(&mut self, deck: &'h [T]) -> Option<Hand<'h, T>> {
        // Quickly checking equal sizes.
        // self.seq is (0..len), which means that getting the original len is self.seq.len()
        //
        // This check is needed to check hand-making, to make sure that the range of choices and
        // the slice which is chosen
        if deck.len() != self.len() {
            None
        } else {
            Some(self.make_hand_unchecked(deck))
        }
    }

    /// Creates a [`Hand`](Hand), similar to [`make_hand`](Dispenser::make_hand), but without the
    /// check to see if all choices can land, and all elements are choosable.
    pub fn make_hand_unchecked<'h, T>(&mut self, deck: &'h [T]) -> Hand<'h, T> {
        let b: Box<[usize]> = self.seq.clone().into_boxed_slice();
        self.shuffle();
        Hand::new(b, deck)
    }

    fn shuffle(&mut self) {
        self.seq.shuffle(&mut self.rng);
    }
}

/// A lock on a slice of choices, with a slice of elements to match them.
#[derive(Debug)]
pub struct Hand<'h, T>(Box<[usize]>, &'h [T]);

impl<'h, T> Hand<'h, T> {
    /// Creates a new hand from a slice of choices, and a slice of elements.
    ///
    /// Slice length equivalence isn't checked.
    pub fn new(choices: Box<[usize]>, elements: &'h [T]) -> Hand<'h, T> {
        Hand(choices, elements)
    }

    /// Pick from a series of choices by index, which then picks a corresponding element from the list.
    ///
    /// This function uses the internal (frozen) slice of choices to choose an element out of the
    /// slice of elements.
    ///
    /// With a choice slice of `[2,3,1,0,4]`, and an element slice of `[A,B,C,D,E]`, picking choice
    /// `1` (as an index) would use choice `4` to pick `D`.
    ///
    /// `1 -> [..., 4, ...] -> [..., D, ...]`
    ///
    /// ```
    /// use hidden::dispenser::Hand;
    ///
    /// // Note, this is not how you'd go about getting a Hand
    /// // Please use a Dispenser instead
    /// let choices = Box::from([2,3,1,0,4]);
    /// let elements = &['a','b','c','d','e'];
    /// let hand = Hand::new(choices, elements);
    ///
    /// assert_eq!(hand.choose(1).unwrap(), &'d'); // idx 1 -> choice 3 -> element d
    /// assert_eq!(hand.choose(2).unwrap(), &'b'); // idx 2 -> choice 1 -> element b
    /// ```
    ///
    /// Returns [`None`](None) if `idx` exceeds amount of choices, or if choice can't correspond to
    /// an element in the list.
    ///
    /// Returns [`Some`](Some) with a reference to an element if the choosing succeeds.
    pub fn choose(&self, idx: usize) -> Option<&T> {
        if let Some(u) = self.0.get(idx) {
            if let Some(t) = self.1.get(u.to_owned()) {
                return Some(t);
            }
        }
        None
    }

    /// Returns the amount of choices that this hand has.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

// Tests

#[test]
fn numbers() {
    let choices = (1..10).collect::<Vec<u8>>();
    let mut dispenser = Dispenser::new(choices.len());
    let hand1 = dispenser.make_hand_unchecked(&choices);
    let hand2 = dispenser.make_hand_unchecked(&choices);
    let hand3 = dispenser.make_hand_unchecked(&choices);

    let choice1 = hand1.choose(0).unwrap();
    assert!(choices.contains(choice1));

    let choice2 = hand2.choose(0).unwrap();
    assert!(choices.contains(choice2));

    let choice3 = hand1.choose(0).unwrap();
    assert!(choices.contains(choice3));

    dbg!(hand1, hand2, hand3);
}

#[test]
fn enums() {
    #[derive(Eq, PartialEq, Debug)]
    enum Something {
        A,
        B,
        C,
        D,
    }

    let choices: Vec<Something> = vec![Something::A, Something::B, Something::C, Something::D];

    let mut dispenser = Dispenser::new(choices.len());
    let hand1 = dispenser.make_hand_unchecked(&choices);
    let hand2 = dispenser.make_hand_unchecked(&choices);

    let choice1 = hand1.choose(0).unwrap();
    let choice2 = hand2.choose(0).unwrap();

    assert!(choices.contains(choice1));
    assert!(choices.contains(choice2));

    dbg!(choice1, choice2);
    dbg!(hand1, hand2);
}

#[test]
fn some_and_none() {
    let choices = vec![0];
    let mut dispenser = Dispenser::new(choices.len());
    let za_hando = dispenser.make_hand_unchecked(&choices);

    assert!(za_hando.choose(0).is_some());
    assert!(za_hando.choose(1).is_none());
}
