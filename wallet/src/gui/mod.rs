use cashurs_core::model::Tokens;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{Column, Container},
    Element, Length,
};
use iced_aw::TabLabel;

use crate::error::CashuWalletError;

use self::{settings_tab::SettingsMessage, wallet_tab::WalletMessage};

pub mod settings_tab;
pub mod wallet_tab;

#[derive(Debug)]
pub enum Message {
    TabSelected(usize),
    Settings(SettingsMessage),
    Wallet(WalletMessage),
    Something(Result<Tokens, CashuWalletError>),
}

pub trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<'_, Self::Message> {
        let column = Column::new().spacing(20).push(self.content());

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(5)
            .into()
    }

    fn content(&self) -> Element<'_, Self::Message>;
}
