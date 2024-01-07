use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Symbol
{
    Rock,
    Paper,
    Scissor,
}

pub struct Round
{
    pub your_symbol: Option<Symbol>,
    opp_symbol: Option<Symbol>,
}

pub struct Match
{
    pub players: (String, String),
    pub target_score: u32,
    pub scores: (u32, u32),
    pub current_round: Round,
    pub last_round: Option<(Symbol, Symbol)>,
}

#[derive(Debug, Clone, Copy)]
pub enum Victor
{
    You,
    Opp,
    Tie,
}

impl Round
{
    fn new() -> Self
    {
        Self
        {
            your_symbol: None,
            opp_symbol: None,
        }
    }

    pub fn finished(&self) -> bool
    {
        self.victor().is_some()
    }

    pub fn your_turn(&mut self, symbol: Symbol) -> bool
    {
        self.your_symbol.replace(symbol);
        self.finished()
    }

    pub fn opp_turn(&mut self, symbol: Symbol) -> bool
    {
        self.opp_symbol.replace(symbol);
        self.finished()
    }

    fn victor(&self) -> Option<Victor> 
    {
        use Symbol::*;
        match (self.your_symbol, self.opp_symbol) 
        {
            (Some(you), Some(opp)) => Some(match (you, opp) 
            {
                (Rock, Rock) | (Paper, Paper) | (Scissor, Scissor) => Victor::Tie,
                (Rock, Paper) | (Paper, Scissor) | (Scissor, Rock) => Victor::Opp,
                (Rock, Scissor) | (Paper, Rock) | (Scissor, Paper) => Victor::You,
            }),
            _ => None,
        }
    }
}

impl Match
{
    pub fn new(target_score: u32, you: String, opponent: String) -> Self
    {
        Self { players: (you, opponent), target_score, scores: (0,0), current_round: Round::new(), last_round: None }
    }

    pub fn next_round(&mut self) -> Option<Victor>
    {
        if let Some(victor) = self.current_round.victor()
        {
            match victor
            {
                Victor::You => self.scores.0 += 1,
                Victor::Opp => self.scores.1 += 1,
                Victor::Tie => {},
            }
            self.victor().or_else(||
            {
                self.last_round = self.current_round.your_symbol.zip(self.current_round.opp_symbol);
                self.current_round = Round::new();
                None
            })
        } else
        {
            None
        }
    }

    pub fn victor(&self) -> Option<Victor>
    {
        if self.scores.0 >= self.target_score
        {
            Some(Victor::You)
        } 
        else if self.scores.1 >= self.target_score
        {
            Some(Victor::Opp)
        } else
        {
            None
        }
    }
}
