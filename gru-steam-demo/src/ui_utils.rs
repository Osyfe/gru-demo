use super::{data::{State, Data}, game::{Match, Symbol}};
use gru_ui::{Widget, widget::{WidgetExt, layout::*, primitive::*, compose::*, dynamic::*}, lens::{Lens, LensTuple0, LensTuple1, LensExt}};
use std::borrow::Borrow;
use EventTag::*;

#[derive(Clone, Copy)]
pub enum EventTag
{
    CreateLobby,
    LeaveLobby,
    StartMatch,
    Pick(Symbol),
    Abandon,
    EndApp,
}

fn button<T, L: Borrow<str>>(label: L, tag: EventTag) -> impl Widget<T, EventTag>
{
    Label::new().own(label).pad().horizontal(0.5).align().bg().response().event(tag)
}

fn menu() -> impl Widget<Data, EventTag>
{
    Flex::column()
        .with(Label::new().size(2.0).own("Menu").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(button("Create Lobby", CreateLobby))
        .with(button("Exit", EndApp))
        .padding(0.5)
        .align()
        .pad().horizontal(1.0).vertical(1.0)
}

fn lobby() -> impl Widget<Data, EventTag>
{
    Flex::column()
        .with(Label::new().size(2.0).own("Lobby").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(Dynamic::new(|data: &mut Data|
        {
            let mut list = Flex::column();
            list.add(Label::new().own("Players:"));
            if let State::Lobby(_, data) = &data.state
            {
                for member in &data.members
                {
                    list.add(Label::new().own(member.1.to_owned()).pad().horizontal(0.5));
                }
            }
            list
        }))
        .with(Empty.fix().height(1.0))
        .with(button("Start Match", StartMatch))
        .with(button("Leave Lobby", LeaveLobby))
        .padding(0.5)
        .align()
        .pad().horizontal(1.0).vertical(1.0)
}

fn game() -> impl Widget<Data, EventTag>
{
    let your_score = Label::new().map(|score| format!("Your Score: {score}")).lens(LensTuple0);
    let opp_score = Label::new().map(|score| format!("Opponent Score: {score}")).lens(LensTuple1);
    Flex::column()
        .with(Label::new().size(2.0).own("In Game").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(Flex::row().with(your_score).with(opp_score).padding(0.5).lens(Match::scores))
        .with(Flex::row().with(button("Rock", Pick(Symbol::Rock))).with(button("Paper", Pick(Symbol::Paper))).with(button("Scissor", Pick(Symbol::Scissor))).padding(0.5))
        .with(Empty.fix().height(1.0))
        .with(button("Abandon", Abandon).align())
        .padding(0.5)
        .align()
        .pad().horizontal(1.0).vertical(1.0)
        .lens(Data::state.chain(MatchLens))
        //turn count
}

pub fn build() -> impl Widget<Data, EventTag>
{
    let set = Set::new()
        .with(menu().maybe(|data: &mut Data| matches!(data.state, State::Menu)))
        .with(lobby().maybe(|data: &mut Data| matches!(data.state, State::Lobby(_, _))))
        .with(game().maybe(|data: &mut Data| matches!(data.state, State::Match(_, _))));

    Flex::column()
        .with(Label::new().size(3.0).own("Rock Paper Scissors").align().center_h())
        .with(Empty.fix().height(2.0))
        .with(set)
        .pad().vertical(1.0)
}

struct MatchLens;

impl Lens<State, Match> for MatchLens
{
    fn with<A, F: FnOnce(&Match) -> A>(&mut self, data: &State, f: F) -> A
    {
        match data
        {
            State::Match(_, game) => f(game),
            _ => f(&Default::default()),
        }
    }

    fn with_mut<A, F: FnOnce(&mut Match) -> A>(&mut self, data: &mut State, f: F) -> A
    {
        match data
        {
            State::Match(_, game) => f(game),
            _ => f(&mut Default::default()),
        }
    }
}
