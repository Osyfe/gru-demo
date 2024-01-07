#[derive(Debug, Clone, Copy)]
pub enum Symbol {
    Rock,
    Paper,
    Scissor,
}

pub struct Round {
    pub index: u32,
    pub your_symbol: Option<Symbol>,
    opp_symbol: Option<Symbol>,
}

pub struct Game {
    pub players: (String, String),
    pub max_round: u32,
    pub current_round: Round,
    pub scores: (u32, u32),
}

pub enum Victor {
    You,
    Opp,
    Tie,
}

impl Round {
    fn new(index: u32) -> Self
    {
        Self {
            index,
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

impl Game {
    pub fn new(max_round: u32, you: String, opponent: String) -> Self
    {
        Self { players: (you, opponent), max_round, current_round: Round::new(0), scores: (0,0) }
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

            self.victor().or_else(|| {self.current_round = Round::new(self.current_round.index + 1); None})
        } else
        {
            None
        }
    }

    pub fn victor(&self) -> Option<Victor>
    {
        let posible_points = self.max_round - self.current_round.index;
        let max_scores = (self.scores.0 + posible_points, self.scores.1 + posible_points);

        if max_scores.0 < self.scores.1
        {
            Some(Victor::Opp)
        } 
        else if max_scores.1 < self.scores.0
        {
            Some(Victor::You)
        }
        else if posible_points == 0
        {
            Some(Victor::Tie)
        }
        else
        {
            None
        }
    }
}