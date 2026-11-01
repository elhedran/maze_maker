//! What the game needs of `maze-core` beyond `World`'s own API (DR-0043
//! point 3). G.1's step rule is `World::joined` (DR-0036's *joined*: both
//! cells stored, neither facing side hedge), which 2.6 step 3 added, so
//! this module holds only the promise that a `World` can live on its own
//! thread beside a game engine (DR-0043 point 10).

use crate::world::World;

/// Don't remove this to make a build pass; raise it with Yew.
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<World>();
};
