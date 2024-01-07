use super::{data::{State, Data}, ui::{Widget, widget::{WidgetExt, layout::*, primitive::*, compose::*, dynamic::*}}, game::Symbol};

#[derive(Clone, Copy)]
pub enum EventTag
{
    CreateLobby,
    LeaveLobby,
    StartMatch,
    Pick(Symbol)
}

pub fn build() -> impl Widget<Data, EventTag>
{
    use EventTag::*;
    
    let menu = Flex::column()
        .with(Label::new().size(2.0).own("Menu").align().center_h())
        .with(Empty.fix().height(1.0))
        .with(Label::new().own("Create Lobby").pad().horizontal(0.5).align().bg().response().event(CreateLobby))
        .with(Label::new().own("Exit").pad().horizontal(0.5).align().bg().response().action(|_, data: &mut Data| data.state = State::End))
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
            if let State::Lobby(data) = &data.state
            {
                for member in &data.members
                {
                    list.add(Label::new().own(member.to_owned()).pad().horizontal(0.5));
                }
            }
            list
        }))
        .with(Empty.fix().height(1.0))
        .with(Label::new().own("Start Match").pad().horizontal(0.5).align().bg().response().event(StartMatch))
        .with(Label::new().own("Leave Lobby").pad().horizontal(0.5).align().bg().response().event(LeaveLobby))
        .padding(0.5)
        .align()
        .pad().horizontal(1.0).vertical(1.0);

    let game = Set::new().
        with(Label::new().own("In Game"));
        //turn count //score //pick options

    let set = Set::new()
        .with(menu.maybe(|data: &mut Data| matches!(data.state, State::Menu)))
        .with(lobby.maybe(|data: &mut Data| matches!(data.state, State::Lobby(_))))
        .with(game.maybe(|data: &mut Data| matches!(data.state, State::Match(_))));

    Flex::column()
        .with(Label::new().size(3.0).own("Rock Paper Scissors").align().center_h())
        .with(Empty.fix().height(2.0))
        .with(set)
        .pad().vertical(1.0)
}
