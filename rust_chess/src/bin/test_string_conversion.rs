use shakmaty::{Bitboard, Square, Rank};

fn main() {
    let bb = Bitboard::from_square(Square::H8)
        .with(Bitboard::from_square(Square::B7))
        .with(Bitboard::from_rank(Rank::First));

    // python-chess str() representation
    let expected = "\
. . . . . . . 1
. 1 . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
1 1 1 1 1 1 1 1";

    // Format using shakmaty's Debug formatting and trim the trailing newline
    // so it perfectly matches python-chess's representation
    let formatted = format!("{:?}", bb).trim_end().to_string();

    assert_eq!(formatted, expected);
    println!("String conversion test passed successfully!");
}
