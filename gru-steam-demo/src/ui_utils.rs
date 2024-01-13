use super::{data::{State, Data}, ui::{Widget, widget::{WidgetExt, layout::*, primitive::*, compose::*, dynamic::*}}, game::Symbol};
use std::borrow::Borrow;

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

pub fn build() -> impl Widget<Data, EventTag>
{
    use EventTag::*;
    
    let menu = Flex::column()
        .with(Label::new().size(2.0).own("Menu").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(button("Create Lobby", CreateLobby))
        .with(button("Exit", EndApp))
        .padding(0.5)
        .align()
        .pad().horizontal(1.0).vertical(1.0);

    let lobby = Flex::column()
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
        .pad().horizontal(1.0).vertical(1.0);

    let game = Flex::column()
        .with(Label::new().size(2.0).own("In Game").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(Flex::row().with(button("Rock", Pick(Symbol::Rock))).with(button("Paper", Pick(Symbol::Paper))).with(button("Scissor", Pick(Symbol::Scissor))).padding(0.5))
        .with(Empty.fix().height(1.0))
        .with(button("Abandon", Abandon).align())
        .padding(0.5)
        .align()
        .pad().horizontal(1.0).vertical(1.0);
        //turn count //score

    let set = Set::new()
        .with(menu.maybe(|data: &mut Data| matches!(data.state, State::Menu)))
        .with(lobby.maybe(|data: &mut Data| matches!(data.state, State::Lobby(_, _))))
        .with(game.maybe(|data: &mut Data| matches!(data.state, State::Match(_, _))));

    Flex::column()
        .with(Label::new().size(3.0).own("Rock Paper Scissors").align().center_h())
        .with(Empty.fix().height(2.0))
        .with(set)
        .pad().vertical(1.0)
}
