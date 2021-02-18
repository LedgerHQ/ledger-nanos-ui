use crate::bagls::*;
use nanos_sdk::buttons::{get_button_event, ButtonEvent, ButtonsState};
use nanos_sdk::*;

/// Handles communication to filter
/// out actual events, and converts key
/// events into presses/releases
pub fn get_event(buttons: &mut ButtonsState) -> Option<ButtonEvent> {
    if !seph::is_status_sent() {
        seph::send_general_status();
    }

    // TODO: Receiving an APDU while in UX will lead to .. exit ?
    while seph::is_status_sent() {
        seph::seph_recv(&mut buttons.cmd_buffer, 0);
        let tag = buttons.cmd_buffer[0];

        // button push event
        if tag == 0x05 {
            let button_info = buttons.cmd_buffer[3] >> 1;
            return get_button_event(buttons, button_info);
        }
    }
    None
}

/// Shorthand to display a single message
/// and wait for button action
pub fn popup(message: &str) {
    SingleMessage::new(&message).show_and_wait();
}

/// Display a single screen with a message,
/// and exit the function with 'true'
/// if the user validated 'message'
/// or false if the user aborted
pub struct Validator<'a> {
    message: &'a str,
}

impl<'a> Validator<'a> {
    pub fn new(message: &'a str) -> Self {
        Validator { message }
    }

    pub fn ask(&self) -> bool {
        let mut buttons = ButtonsState::new();

        let cancel = LabelLine::new().dims(128, 11).pos(0, 26).text("Cancel");
        let yes = LabelLine::new().dims(128, 11).pos(0, 12).text(self.message);

        cancel.display();
        yes.bold().paint();

        let mut response = true;

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    UP_ARROW.paint();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    DOWN_ARROW.paint();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    response = true;
                    cancel.display();
                    yes.bold().paint();
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    response = false;
                    cancel.bold().display();
                    yes.paint();
                }
                Some(ButtonEvent::BothButtonsPress) => {
                    match response {
                        true => {
                            yes.bold().display();
                        }
                        false => {
                            cancel.bold().display();
                        }
                    };
                }
                Some(ButtonEvent::BothButtonsRelease) => return response,
                _ => (),
            }
        }
    }
}

pub struct MessageValidator<'a> {
    /// Strings displayed in the pages. One string per page. Can be empty.
    message: &'a [&'a str],
    /// Strings displayed in the confirmation page.
    /// 0 element: only the icon is displayed, in center of the screen.
    /// 1 element: icon and one line of text displayed.
    /// 2 elements: icon and two lines of text displayed.
    confirm: &'a [&'a str],
    /// Strings displayed in the cancel page.
    /// 0 element: only the icon is displayed, in center of the screen.
    /// 1 element: icon and one line of text displayed.
    /// 2 elements: icon and two lines of text displayed.
    cancel: &'a [&'a str],
}

impl<'a> MessageValidator<'a> {
    pub const fn new(
        message: &'a [&'a str],
        confirm: &'a [&'a str],
        cancel: &'a [&'a str],
    ) -> Self {
        MessageValidator {
            message,
            confirm,
            cancel,
        }
    }

    pub fn ask(&self) -> bool {
        let page_count = &self.message.len() + 2;
        let mut cur_page = 0;

        let draw_icon_and_text = |icon: Icons, strings: &[&str]| {
            // Draw icon on the center if there is no text.
            let (x, y) = match strings.len() {
                0 => (16, 12),
                _ => (16, 12),
            };
            Bagl::Icon(Icon::new(icon).pos(x, y)).display();
            match strings.len() {
                0 => {}
                1 => {
                    Bagl::LabelLine(LabelLine::new().text(&strings[0]).pos(0, 20)).paint();
                }
                _ => {
                    Bagl::LabelLine(LabelLine::new().text(&strings[0]).pos(0, 13)).paint();
                    Bagl::LabelLine(LabelLine::new().text(&strings[1]).pos(0, 26)).paint();
                }
            }
        };

        let draw = |page: usize| {
            if page == page_count - 2 {
                draw_icon_and_text(Icons::CheckBadge, &self.confirm);
                RIGHT_ARROW.paint();
            } else if page == page_count - 1 {
                draw_icon_and_text(Icons::CrossBadge, &self.cancel);
            } else {
                Bagl::LabelLine(LabelLine::new().text(&self.message[page])).display();
                RIGHT_ARROW.paint();
            }
            if page > 0 {
                LEFT_ARROW.paint();
            }
        };

        draw(cur_page);

        let mut buttons = ButtonsState::new();
        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonRelease) => {
                    if cur_page > 0 {
                        cur_page -= 1;
                        draw(cur_page);
                    }
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    if cur_page < page_count - 1 {
                        cur_page += 1;
                        draw(cur_page);
                    }
                }
                Some(ButtonEvent::BothButtonsRelease) => {
                    if cur_page == page_count - 2 {
                        // Confirm
                        return true;
                    } else if cur_page == page_count - 1 {
                        // Abort
                        return false;
                    }
                }
                _ => (),
            }
        }
    }
}

pub struct Menu<'a> {
    panels: &'a [&'a str],
}

impl<'a> Menu<'a> {
    pub fn new(panels: &'a [&'a str]) -> Self {
        Menu { panels }
    }

    pub fn show(&self) -> usize {
        let mut buttons = ButtonsState::new();

        let bot = LabelLine::new().dims(128, 11).pos(0, 26);
        let top = LabelLine::new().dims(128, 11).pos(0, 12);

        bot.text(self.panels[1]).display();
        top.text(self.panels[0]).bold().paint();

        UP_ARROW.paint();
        DOWN_ARROW.paint();

        let mut index = 0;

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    UP_S_ARROW.paint();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    DOWN_S_ARROW.paint();
                }
                Some(ButtonEvent::BothButtonsRelease) => return index,
                Some(x) => {
                    match x {
                        ButtonEvent::LeftButtonRelease => {
                            index = index.saturating_sub(1);
                        }
                        ButtonEvent::RightButtonRelease => {
                            if index < self.panels.len() - 1 {
                                index += 1;
                            }
                        }
                        _ => (),
                    }
                    UP_ARROW.display();
                    DOWN_ARROW.paint();
                    let a = (index / 2) * 2;
                    let newtop = self.panels[a];
                    let newbot = self.panels.get(a + 1);

                    if index & 1 == 0 {
                        top.text(newtop).bold().paint();
                        if let Some(b) = newbot {
                            bot.text(b).paint();
                        }
                    } else {
                        top.text(newtop).paint();
                        if let Some(b) = newbot {
                            bot.text(b).bold().paint();
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

/// A gadget that displays
/// a short message in the
/// middle of the screen and
/// waits for a button press
pub struct SingleMessage<'a> {
    message: &'a str,
}

impl<'a> SingleMessage<'a> {
    pub fn new(message: &'a str) -> Self {
        SingleMessage { message }
    }

    pub fn show(&self) {
        LabelLine::new().text(self.message).display();
    }
    /// Display the message and wait
    /// for any kind of button release
    pub fn show_and_wait(&self) {
        let mut buttons = ButtonsState::new();

        self.show();

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonRelease)
                | Some(ButtonEvent::RightButtonRelease)
                | Some(ButtonEvent::BothButtonsRelease) => return,
                _ => (),
            }
        }
    }
}

/// A horizontal scroller that
/// splits any given message
/// over several panes in chunks
/// of CHAR_N characters.
/// Press both buttons to exit.
pub struct MessageScroller<'a> {
    message: &'a str,
}

impl<'a> MessageScroller<'a> {
    pub fn new(message: &'a str) -> Self {
        MessageScroller { message }
    }

    pub fn event_loop(&self) {
        let mut buttons = ButtonsState::new();
        const CHAR_N: usize = 16;
        let page_count = (self.message.len() - 1) / CHAR_N + 1;
        if page_count == 0 {
            return;
        }
        let label = LabelLine::new();
        let mut cur_page = 0;

        // A closure to draw common elements of the screen
        // cur_page passed as parameter to prevent borrowing
        let draw = |page: usize| {
            let start = page * CHAR_N;
            let end = (start + CHAR_N).min(self.message.len());
            let chunk = &self.message[start..end];
            label.text(&chunk).display();
            if page > 0 {
                LEFT_ARROW.paint();
            }
            if page + 1 < page_count {
                RIGHT_ARROW.paint();
            }
        };

        draw(cur_page);

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    LEFT_S_ARROW.paint();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    RIGHT_S_ARROW.paint();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    if cur_page > 0 {
                        cur_page -= 1;
                    }
                    // We need to draw anyway to clear button press arrow
                    draw(cur_page);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    if cur_page + 1 < page_count {
                        cur_page += 1;
                    }
                    // We need to draw anyway to clear button press arrow
                    draw(cur_page);
                }
                Some(ButtonEvent::BothButtonsRelease) => break,
                Some(_) | None => (),
            }
        }
    }
}

/// Horizontal scroller that
/// displays a number of Bagls
/// over the same number of panes
pub struct HScroller<'a> {
    screens: &'a [Bagl<'a>],
}

impl<'a> HScroller<'a> {
    pub fn new(screens: &'a [Bagl<'a>]) -> Self {
        HScroller { screens }
    }

    pub fn event_loop(&self) {
        let mut buttons = ButtonsState::new();
        let mut cur_idx = 0;

        RIGHT_ARROW.display();
        self.screens[cur_idx].paint();

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    LEFT_S_ARROW.paint();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    RIGHT_S_ARROW.paint();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    if cur_idx > 0 {
                        cur_idx -= 1; // Otherwise block onto first panel
                    }

                    RIGHT_ARROW.display();
                    if cur_idx != 0 {
                        LEFT_ARROW.paint();
                    }
                    self.screens[cur_idx].paint();
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    let last_item = self.screens.len() - 1;
                    if cur_idx < last_item {
                        cur_idx += 1; // Otherwise block onto last panel
                    }

                    LEFT_ARROW.display();
                    if cur_idx != last_item {
                        RIGHT_ARROW.paint();
                    }
                    self.screens[cur_idx].paint();
                }
                Some(ButtonEvent::BothButtonsRelease) => {
                    break;
                }
                Some(_) | None => (),
            }
        }
    }
}
