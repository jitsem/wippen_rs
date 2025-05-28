mod shuffle;
use crate::Suit::Spades;
use shuffle::Shuffle;
use wasm_bindgen::prelude::*;

/// The engine is the main entrypoint. Representing a game that is "set up" but not started yet.
#[wasm_bindgen]
struct FreshGame {
    seed: u64,
}

/// Represents a game in progress
#[derive(Debug, Clone)]
#[wasm_bindgen]
struct InProgressGame {
    game_state: GameState,
    events: Vec<GameEvent>,
}

#[wasm_bindgen]
impl InProgressGame {
    pub fn get_events(&self) -> Vec<GameEvent> {
        self.events.clone()
    }
    pub fn get_player_1_hand(&self) -> Vec<Card> {
        self.game_state.player1.hand.cards.clone()
    }
    pub fn get_player_1_score_pile(&self) -> Vec<Card> {
        self.game_state.player1.score_pile.cards.clone()
    }
    pub fn get_player_2_hand(&self) -> Vec<Card> {
        self.game_state.player2.hand.cards.clone()
    }

    pub fn get_player_2_score_pile(&self) -> Vec<Card> {
        self.game_state.player2.score_pile.cards.clone()
    }

    pub fn get_middle(&self) -> Vec<Card> {
        self.game_state.middle.cards.clone()
    }
    pub fn get_deck(&self) -> Vec<Card> {
        self.game_state.deck.cards.clone()
    }

    pub fn get_player_1_score(&self) -> usize {
        self.game_state.player1.score_pile.count_score() + self.game_state.player1.wip_gains
    }

    pub fn get_player_2_score(&self) -> usize {
        self.game_state.player2.score_pile.count_score() + self.game_state.player2.wip_gains
    }
    /// Play a card from the player hand (if possible). Ideally, this would he more enum-y,
    /// but the wasm_bindgen does not support valued enums. So the hacky way is to consume self
    ///and return a new instance.
    pub fn play_card(mut self, card: Card) -> Option<InProgressGame> {
        match self.game_state.player1.hand.take_same_card(&card) {
            None => None,
            Some(player1_card) => {
                let scored = Self::play_and_score(
                    &mut self.game_state.middle,
                    &mut self.game_state.player1.score_pile,
                    player1_card,
                );
                if scored {
                    self.events.push(GameEvent::Player1Scored)
                }
                if self.game_state.middle.is_empty() {
                    self.game_state.player1.wip_gains += 1;
                    self.events.push(GameEvent::Player1Wipped);
                }
                self.events.push(GameEvent::Player1Played);

                //TODO actually implement a strategy
                let player2_card = self
                    .game_state
                    .player2
                    .hand
                    .cards
                    .pop()
                    .expect("Player 2 has no card. This should not happen");
                let scored = Self::play_and_score(
                    &mut self.game_state.middle,
                    &mut self.game_state.player2.score_pile,
                    player2_card,
                );
                if scored {
                    self.events.push(GameEvent::Player2Scored)
                }
                if self.game_state.middle.is_empty() {
                    self.game_state.player2.wip_gains += 1;
                    self.events.push(GameEvent::Player2Wipped);
                }
                self.events.push(GameEvent::Player2Played);

                if self.game_state.player1.hand.is_empty() {
                    self.events.push(GameEvent::RoundOver);
                    if self.game_state.can_deal() {
                        self.game_state.deal();
                        self.events.push(GameEvent::Dealt);
                    } else {
                        //TODO find a way to transition to a Finished game. For now, our JS will use
                        //the events to check for game over
                        let last_scored = self
                            .events
                            .iter()
                            .filter(|e| {
                                **e == GameEvent::Player1Scored || **e == GameEvent::Player2Scored
                            })
                            .next_back()
                            .expect("Expected at least one score at the end of the game");
                        let remainder_score_pile = match last_scored {
                            GameEvent::Player1Scored => &mut self.game_state.player1.score_pile,
                            GameEvent::Player2Scored => &mut self.game_state.player1.score_pile,
                            _ => panic!("We filter on scored events, yet also got other stuff"),
                        };
                        while let Some(card) = self.game_state.middle.take_top_card() {
                            remainder_score_pile.push_card(card);
                        }
                        self.events.push(GameEvent::GameOver);
                    }
                }
                Some(self)
            }
        }
    }

    fn play_and_score(middle: &mut Deck, score_pile: &mut Deck, card: Card) -> bool {
        let mut scored = false;
        while let Some(matching) = middle.take_same_number(&card) {
            scored = true;
            score_pile.push_card(matching);
        }
        if scored {
            score_pile.push_card(card)
        } else {
            middle.push_card(card)
        }
        scored
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
struct GameState {
    deck: Deck,
    player1: Player,
    player2: Player,
    middle: Deck,
}

impl GameState {
    fn can_deal(&self) -> bool {
        self.deck.cards.len() >= 4
    }
    fn deal(&mut self) {
        for _ in 0..2 {
            self.player1.hand.push_card(
                self.deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            self.player1.hand.push_card(
                self.deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            self.player2.hand.push_card(
                self.deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            self.player2.hand.push_card(
                self.deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
        }
    }
}
#[derive(Debug, Clone)]
#[wasm_bindgen]
struct Player {
    hand: Deck,
    score_pile: Deck,
    wip_gains: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[wasm_bindgen]
pub enum GameEvent {
    Started,
    Shuffled,
    Dealt,
    Player1Wipped,
    Player1Played,
    Player1Scored,
    Player2Wipped,
    Player2Played,
    Player2Scored,
    RoundOver,
    GameOver,
}

impl Player {
    pub fn new() -> Self {
        Player {
            hand: Deck::empty(),
            score_pile: Deck::empty(),
            wip_gains: 0,
        }
    }
}

#[wasm_bindgen]
impl FreshGame {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> Self {
        FreshGame { seed }
    }

    #[wasm_bindgen]
    pub fn start(self) -> InProgressGame {
        let mut game_state = GameState {
            deck: Deck::new(),
            middle: Deck::empty(),
            player1: Player::new(),
            player2: Player::new(),
        };
        let mut events = Vec::new();
        events.push(GameEvent::Started);
        game_state.deck.shuffle(self.seed);
        events.push(GameEvent::Shuffled);
        for _ in 0..2 {
            game_state.player1.hand.push_card(
                game_state
                    .deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            game_state.player1.hand.push_card(
                game_state
                    .deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            game_state.middle.push_card(
                game_state
                    .deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            game_state.middle.push_card(
                game_state
                    .deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            game_state.player2.hand.push_card(
                game_state
                    .deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
            game_state.player2.hand.push_card(
                game_state
                    .deck
                    .take_top_card()
                    .expect("We expect a new deck to always yield cards"),
            );
        }
        events.push(GameEvent::Dealt);
        InProgressGame { game_state, events }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[wasm_bindgen]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[wasm_bindgen]
pub struct CardNumber(pub u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[wasm_bindgen]
pub struct Card {
    pub suit: Suit,
    pub card_number: CardNumber,
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
struct Deck {
    cards: Vec<Card>,
}

impl Deck {
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

    pub fn empty() -> Self {
        let cards = Vec::new();
        Deck { cards }
    }

    pub fn shuffle(&mut self, seed: u64) {
        self.cards.shuffle(seed, 9128);
    }

    pub fn take_top_card(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn take_same_card(&mut self, card: &Card) -> Option<Card> {
        let index = self.cards.iter().position(|c| c == card)?;
        Some(self.cards.swap_remove(index))
    }
    pub fn take_same_number(&mut self, card: &Card) -> Option<Card> {
        let index = self
            .cards
            .iter()
            .position(|c| c.card_number == card.card_number)?;
        Some(self.cards.swap_remove(index))
    }

    pub fn push_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    pub fn count_score(&self) -> usize {
        let mut count = 0;
        //Aces are worth 1 point
        count += self.cards.iter().filter(|c| c.card_number.0 == 1).count();
        //schoppen 2
        count += self
            .cards
            .iter()
            .filter(|c| c.card_number.0 == 2 && c.suit == Suit::Spades)
            .count();
        //Koeken 10
        count += self
            .cards
            .iter()
            .filter(|c| c.card_number.0 == 10 && c.suit == Suit::Diamonds)
            .count()
            * 2;
        //Most cards
        if self.cards.len() > 26 {
            count += 2
        };
        //Most schoppens
        if self.cards.iter().filter(|c| c.suit == Spades).count() >= 7 {
            count += 2
        };

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_shuffle_deck() {
        let mut deck = Deck::new();
        deck.shuffle(123);
        assert_eq!(deck.cards.len(), 52)
    }

    #[test]
    fn can_start_a_game_from_fresh_game() {
        let fresh = FreshGame::new(123);
        let in_progress = fresh.start();
        assert_eq!(in_progress.game_state.player1.hand.cards.len(), 4);
        assert_eq!(in_progress.game_state.player2.hand.cards.len(), 4);
        assert_eq!(in_progress.game_state.player1.score_pile.cards.len(), 0);
        assert_eq!(in_progress.game_state.player2.score_pile.cards.len(), 0);
        assert_eq!(in_progress.game_state.middle.cards.len(), 4);
        assert_eq!(in_progress.game_state.deck.cards.len(), 40);
    }
}
