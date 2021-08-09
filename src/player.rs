use crate::game::{Character, Weapon, Room, Card};
use crate::player::Guess::Suggest;

use enum_iterator::IntoEnumIterator;
use rand::seq::SliceRandom;
use rand::thread_rng;


#[derive(Clone, Copy)]
pub enum Guess {
    Suggest(Character, Weapon, Room),
    Accuse(Character, Weapon, Room)
}

pub struct Response {
    pub(crate) from: usize,
    pub(crate) card: Option<Card> //None in the case the response was not to you but another player so you only know there was a response
}

pub trait Player {
    fn new(position: usize, number_of_players: usize, hand: Vec<Card>) -> Self where Self: Sized;
    fn make_guess(&self) -> Guess;
    fn handle_response(&mut self, guess_from: usize, guess: Guess, response: Option<Response>);
}

pub struct RandomPlayer {}
impl Player for  RandomPlayer {
    fn new(position: usize, number_of_players: usize, hand: Vec<Card>) -> Self {
        RandomPlayer{}
    }

    fn make_guess(&self) -> Guess {
        Suggest(
            Character::shuffled_list().pop().unwrap(),
            Weapon::shuffled_list().pop().unwrap(),
            Room::shuffled_list().pop().unwrap()
        )
    }

    fn handle_response(&mut self, guess_from: usize, guess: Guess, response: Option<Response>) {
    }
}

pub struct ProcessOfEliminationPlayer {
    position: usize,
    possibleCharacters: Vec<Character>,
    possibleWeapons: Vec<Weapon>,
    possibleRooms: Vec<Room>
}

impl ProcessOfEliminationPlayer {
    fn eliminate_card(&mut self, card: Card) {
        match card {
            Card::Character(c) => {self.possibleCharacters.retain(|x| *x != c)}
            Card::Weapon(w) => {self.possibleWeapons.retain(|x| *x != w)}
            Card::Room(r) => {self.possibleRooms.retain(|x| *x != r)}
        }
    }
}

impl Player for ProcessOfEliminationPlayer {

    fn new(position: usize, number_of_players: usize, mut hand: Vec<Card>) -> Self {
        let mut possibleCharacters = Character::shuffled_list();
        let mut possibleWeapons = Weapon::shuffled_list();
        let mut possibleRooms = Room::shuffled_list();
        let mut out = ProcessOfEliminationPlayer {position, possibleCharacters, possibleWeapons, possibleRooms};
        for card in hand.drain(..) {
           out.eliminate_card(card)
        }
        out
    }

    fn make_guess(&self) -> Guess {
        if self.possibleCharacters.len() == 1 && self.possibleWeapons.len() == 1 && self.possibleRooms.len() == 1 {
            return Guess::Accuse(
                (*self.possibleCharacters.first().unwrap()),
                (*self.possibleWeapons.first().unwrap()),
                (*self.possibleRooms.first().unwrap())
            )
        }
        Guess::Suggest(
            (*self.possibleCharacters.first().unwrap()),
            (*self.possibleWeapons.first().unwrap()),
            (*self.possibleRooms.first().unwrap())
        )
    }

    fn handle_response(&mut self, guess_from: usize, guess: Guess, response: Option<Response>) {
        match response {
            Some(Response{card: Some(card), ..}) => {
                self.eliminate_card(card)
            },
            None => {
                if (guess_from != self.position) {
                    return
                }
                if let Suggest(c,w,r) = guess {
                    self.possibleCharacters = vec![c];
                    self.possibleWeapons = vec![w];
                    self.possibleRooms = vec![r];
                }
            },
            _ => {}
        }
    }
}
