use near_sdk::collections::LookupMap;

use crate::bit_math::BitMathTrait;
use crate::num24::I24;
use crate::num256::U256;
use crate::tick_math::TickConstants;

/// @title Packed tick initialized state library
/// @notice Stores a packed mapping of tick index to its initialized state
/// @dev The mapping uses int16 for keys since ticks are represented as int24 and there are 256 (2^8) values per word.

/// @notice Computes the position in the mapping where the initialized bit for a tick lives
/// @param tick The tick for which to compute the position
/// @return wordPos The key in the mapping containing the word in which the bit is stored
/// @return bitPos The bit position in the word where the flag is stored
pub fn position(tick: i32) -> (i16, u8) {
    assert!(
        (TickConstants::MIN_TICK..=TickConstants::MAX_TICK).contains(&tick),
        "INVALID TICK RANGE",
    );
    let word_pos = (tick >> 8) as i16;
    let bit_pos = (tick % 256) as u8;
    (word_pos, bit_pos)
}

/// !MODIFIED
/// @notice Flips the initialized state for a given tick from false to true, or vice versa
/// @param self The mapping in which to flip the tick
/// @param tick The tick to flip
/// @param
/// @param tickSpacing The spacing between usable ticks
pub fn flip_tick(tick_bitmap: &mut LookupMap<i16, U256>, tick: i32, tick_spacing: i32) {
    assert!(
        (TickConstants::MIN_TICK..=TickConstants::MAX_TICK).contains(&tick),
        "INVALID TICK RANGE",
    );
    assert_eq!(tick % tick_spacing, 0); // ensure that the tick is spaced
    let (word_pos, bit_pos) = position(tick / tick_spacing);
    let mask = U256::one() << bit_pos;
    let mut current = tick_bitmap.get(&word_pos).unwrap_or_default();
    current ^= mask;
    tick_bitmap.insert(&word_pos, &current);
}

/// !MODIFIED
/// @notice Returns the next initialized tick contained in the same word (or adjacent word) as the tick that is either
/// to the left (less than or equal to) or right (greater than) of the given tick
/// @param self The mapping in which to compute the next initialized tick
/// @param tick The starting tick
/// @param tickSpacing The spacing between usable ticks
/// @param lte Whether to search for the next initialized tick to the left (less than or equal to the starting tick)
/// !@param get_word A function to get word by word position from a tick bitmap
/// @return next The next initialized or uninitialized tick up to 256 ticks away from the current tick
/// @return initialized Whether the next tick is initialized, as the function only searches within up to 256 ticks
pub fn next_initialized_tick_within_one_word(
    tick_bitmap: &LookupMap<i16, U256>,
    tick: i32,
    tick_spacing: i32,
    lte: bool,
) -> (i32, bool) {
    assert!(
        (TickConstants::MIN_TICK..=TickConstants::MAX_TICK).contains(&tick),
        "INVALID TICK RANGE",
    );
    assert_ne!(tick_spacing, 0);
    let mut compressed = tick / tick_spacing;
    if tick < 0 && tick % tick_spacing != 0 {
        // round towards negative infinity
        compressed -= 1;
    }
    let initialized: bool;
    let next: I24;

    if lte {
        let (word_pos, bit_pos) = position(compressed);
        // all the 1s at or to the right of the current bitPos
        let mask = (U256::one() << bit_pos) - U256::one() + (U256::one() << bit_pos);
        let masked = tick_bitmap.get(&word_pos).unwrap_or_default() & mask;

        // if there are no initialized ticks to the right of or at the current tick, return rightmost in the word
        initialized = masked != U256::zero();
        // overflow/underflow is possible, but prevented externally by limiting both tickSpacing and tick
        next = if initialized {
            ((compressed - (bit_pos as I24 - masked.most_significant_bit() as I24)) as I24)
                * tick_spacing
        } else {
            (compressed - (bit_pos as I24)) * tick_spacing
        };
    } else {
        // start from the word of the next tick, since the current tick state doesn't matter
        let (word_pos, bit_pos) = position(compressed + 1);
        // all the 1s at or to the left of the bitPos
        let mask = !((U256::one() << bit_pos) - U256::one());
        let masked = tick_bitmap.get(&word_pos).unwrap_or_default() & mask;

        // if there are no initialized ticks to the left of the current tick, return leftmost in the word
        initialized = masked != U256::zero();
        next = if initialized {
            (compressed + 1 + (masked.least_significant_bit() as I24 - bit_pos as I24))
                * tick_spacing
        } else {
            (compressed + 1 + (u8::MAX as I24 - bit_pos as I24)) * tick_spacing
        };
    }

    (next, initialized)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_position() {
        // TODO: @galin-chung-nguyen
    }

    #[test]
    fn test_flip_tick() {
        // TODO: @galin-chung-nguyen
    }

    #[test]
    fn test_next_initialized_tick_within_one_word() {
        // TODO: @galin-chung-nguyen
    }
}
