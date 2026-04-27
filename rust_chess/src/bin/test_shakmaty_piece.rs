use shakmaty::Piece;

fn main() {
    let p = Piece::from_char('P');
    println!("{:?}", p);
    let p2 = Piece::from_char('p');
    println!("{:?}", p2);
}
