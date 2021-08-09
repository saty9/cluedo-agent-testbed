use enum_iterator::IntoEnumIterator;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use crate::player::{Player, RandomPlayer, ProcessOfEliminationPlayer, Guess, Response};
use std::collections::VecDeque;


const MAX_PLAYERS: i32 = 6;

#[derive(IntoEnumIterator, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Character {
    SCARLET,
    MUSTARD,
    WHITE,
    PLUM,
    PEACOCK,
    GREEN,
}

impl Character {
    pub(crate) fn shuffled_list() -> Vec<Self> {
        let mut rng = thread_rng();
        let mut out = Character::into_enum_iter().collect::<Vec<Character>>();
        out.shuffle(&mut rng);
        out
    }

    pub fn to_card(&self) -> Card {
        Card::Character(*self)
    }
}

#[derive(IntoEnumIterator, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Weapon {
    PIPE,
    ROPE,
    DAGGER,
    REVOLVER,
    CANDLESTICK,
}

impl Weapon {
    pub(crate) fn shuffled_list() -> Vec<Self> {
        let mut rng = thread_rng();
        let mut out = Weapon::into_enum_iter().collect::<Vec<Weapon>>();
        out.shuffle(&mut rng);
        out
    }

    pub fn to_card(&self) -> Card {
        Card::Weapon(*self)
    }
}

#[derive(IntoEnumIterator, Debug, Eq, PartialEq, Copy, Clone)]
pub enum Room {
    KITCHEN,
    BALLROOM,
    CONSERVATORY,
    BILLIARD_ROOM,
    LIBRARY,
    STUDY,
    HALL,
    LOUNGE,
    DINING_ROOM,
}

impl Room {
    pub(crate) fn shuffled_list() -> Vec<Self> {
        let mut rng = thread_rng();
        let mut out = Room::into_enum_iter().collect::<Vec<Room>>();
        out.shuffle(&mut rng);
        out
    }

    pub fn to_card(&self) -> Card {
        Card::Room(*self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Card {
    Character(Character),
    Weapon(Weapon),
    Room(Room),
}

#[derive(Debug)]
struct Game {
    solution: (Character, Weapon, Room),
    hands: Vec<Vec<Card>>,
}

struct GameResult {
    injected_agent_won: bool,
    number_of_turns: usize,
}

//fn find_matching_card(players: &Vec<(usize, Vec<Card>, Box<dyn Player>)>, searchCards: Vec<Card>) -> Option<(usize, Card)> {
//
//}

impl Game {
    pub fn new(number_of_players: u8) -> Game {
        if number_of_players < 3 || number_of_players > MAX_PLAYERS as u8 {
            panic!("bad number of players")
        }
        let mut hands = Vec::new();
        for _ in 0..number_of_players {
            hands.push(Vec::new())
        }

        let mut rng = thread_rng();
        let mut deck: Vec<Card> = Vec::new();
        let mut characters: Vec<Character> = Character::into_enum_iter().collect();
        characters.shuffle(&mut rng);
        let mut weapons: Vec<Weapon> = Weapon::into_enum_iter().collect();
        weapons.shuffle(&mut rng);
        let mut rooms: Vec<Room> = Room::into_enum_iter().collect();
        rooms.shuffle(&mut rng);
        let solution = (characters.pop().unwrap(), weapons.pop().unwrap(), rooms.pop().unwrap());
        deck.append(&mut characters.drain(..).map(|x| Card::Character(x)).collect());
        deck.append(&mut weapons.drain(..).map(|x| Card::Weapon(x)).collect());
        deck.append(&mut rooms.drain(..).map(|x| Card::Room(x)).collect());
        deck.shuffle(&mut rng);

        let mut counter = 0;
        for card in deck.drain(..) {
            hands[counter % number_of_players as usize].push(card);
            counter += 1;
        }
        Game { solution, hands }
    }

    pub fn play<InjectPlayerType: Player>(&self) -> GameResult {
        let mut rng = thread_rng();
        let injected_player_position = rng.gen_range(0..self.hands.len());
        let mut players: VecDeque<(usize, Vec<Card>, Box<dyn Player>)> = VecDeque::new();
        for (position, hand) in self.hands.iter().enumerate() {
            let player_box: Box<dyn Player> = if position == injected_player_position {
                Box::new(InjectPlayerType::new(position, self.hands.len(), (*hand).clone()))
            } else if rng.gen_bool(1.0) {
                Box::new(RandomPlayer::new(position, self.hands.len(), (*hand).clone()))
            } else {
                Box::new(ProcessOfEliminationPlayer::new(position, self.hands.len(), (*hand).clone()))
            };
            players.push_back((position, (*hand).clone(), player_box))
        }

        let mut turn_number = 0;
        loop {
            let current_player_position = players.front().unwrap().0;
            let guess = players.front().unwrap().2.make_guess();
            match guess {
                Guess::Accuse(c, w, r) => {
                    if c == self.solution.0 && w == self.solution.1 && r == self.solution.2 {
                        return GameResult { injected_agent_won: turn_number % self.hands.len() == injected_player_position, number_of_turns: turn_number }
                    }
                    panic!("incorrect accusation made")
                }
                Guess::Suggest(c, w, r) => {
                    let search_cards = vec![c.to_card(), w.to_card(), r.to_card()];
                    if let Some((match_position, matching_card)) = players.range(1..)
                        .flat_map(|(position, hand, _)| hand.iter().map(move |card| (position.clone(), card.clone())))
                        .find(|(position, hand_card)| search_cards.contains(hand_card) )
                    {
                        for (position, _, player) in players.iter_mut() {
                            if (*position == current_player_position) {
                                let response = Some(Response { from: match_position, card: Some(matching_card) });
                                player.handle_response(current_player_position, guess, response);
                            } else {
                                let response = Some(Response { from: match_position, card: None });
                                player.handle_response(current_player_position, guess, response);
                            }
                        }
                    } else {
                        for (_,_, player) in players.iter_mut() {
                            player.handle_response(current_player_position, guess, None)
                        }
                    }
                }
            }
            turn_number += 1;
            players.rotate_left(1);
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::game::Game;
    use crate::player::ProcessOfEliminationPlayer;

    #[test]
    fn test_game_finishes() {
        Game::new(6).play::<ProcessOfEliminationPlayer>();
    }
}
