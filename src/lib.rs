mod shuffle;
use shuffle::Shuffle;
use wasm_bindgen::prelude::*;

//Idea would be to have the interface to JS be an collection of "Events"
//Each event will be an atomic change on the gameboard.
// Eg. EnemyTook (cardnr, cardvalue, Vec<cards>)
// Eg. PlayerDropped(cardnr, cardValue)
// Eg. PlayerDealt()
// Eg. LastCall()
// Event will also contain the updated game state.
//

#[derive(Debug, Copy, Clone)]
#[wasm_bindgen]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Copy, Clone)]
#[wasm_bindgen]
pub struct CardNumber(pub u8);

#[derive(Debug, Copy, Clone)]
#[wasm_bindgen]
pub struct Card {
    pub suit: Suit,
    pub card_number: CardNumber,
}

#[wasm_bindgen]
struct Deck {
    cards: Vec<Card>,
}

#[wasm_bindgen]
impl Deck {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut cards: Vec<Card> = Vec::with_capacity(52);
        for i in 1..=13 {
            cards.push(Card {
                suit: Suit::Hearts,
                card_number: CardNumber(i),
            });
            cards.push(Card {
                suit: Suit::Diamonds,
                card_number: CardNumber(i),
            });
            cards.push(Card {
                suit: Suit::Clubs,
                card_number: CardNumber(i),
            });
            cards.push(Card {
                suit: Suit::Spades,
                card_number: CardNumber(i),
            });
        }
        assert_eq!(cards.len(), 52);
        Deck { cards }
    }

    #[wasm_bindgen]
    pub fn shuffle(&mut self, seed:u64) {
        self.cards.shuffle(seed, 9128);
    }

    #[wasm_bindgen]
    pub fn take_card(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_shuffle_deck(){
        let mut deck = Deck::new();
        deck.shuffle(123);
        assert_eq!(deck.cards.len(), 52)
    }
}
