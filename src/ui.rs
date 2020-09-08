#![allow(dead_code)] 

use nanos_sdk::*;
use crate::bagls::*;

/// Structure keeping track of button pushes 
/// 1 -> left button, 2 -> right button
pub struct ButtonsState {
    pub button_mask: u8,
    pub cmd_buffer: [u8; 128]
}

impl Default for ButtonsState {
    fn default() -> Self {
        ButtonsState {
            button_mask: 0,
            cmd_buffer: [0u8; 128]
        }
    }
}

impl ButtonsState {
    pub fn new() -> ButtonsState {
        ButtonsState::default()
    }
}

/// Event types needed by 
/// an application
pub enum Event {
    LeftButtonPress,
    RightButtonPress,
    BothButtonsPress,
    LeftButtonRelease,
    RightButtonRelease,
    BothButtonsRelease 
}


/// Distinguish between button press and button release
fn get_button_event(buttons: &mut ButtonsState, new: u8) -> Option<Event> {
    let old =  buttons.button_mask;
    buttons.button_mask |= new;
    match (old, new) {
        (0, 1) => Some(Event::LeftButtonPress), 
        (0, 2) => Some(Event::RightButtonPress), 
        (_, 3) => Some(Event::BothButtonsPress), 
        (b, 0) => {
            buttons.button_mask = 0; // reset state on release
            match b {
                1 => Some(Event::LeftButtonRelease),
                2 => Some(Event::RightButtonRelease),
                3 => Some(Event::BothButtonsRelease),
                _ => None
            }
        } 
        _ => None
    }
}

/// Handles communication to filter
/// out actual events, and converts key
/// events into presses/releases
fn get_event(buttons: &mut ButtonsState) -> Option<Event> {
    if !seph::is_status_sent() {
        seph::send_general_status();
    }

    // TODO: Receiving an APDU while in UX will lead to .. exit ?
    while seph::is_status_sent() {
        seph::seph_recv(&mut buttons.cmd_buffer, 0);
        let tag = buttons.cmd_buffer[0];

        // button push event
        if tag == 0x05 { 
            let button_info = buttons.cmd_buffer[3]>>1;
            return get_button_event(buttons, button_info)
        }
    }
    None
}


/// Display a single screen with a message,
/// and exit the function with 'true'
/// if the user validated 'message'
/// or false if the user aborted
pub struct Validator {
    message: &'static str,
}

impl Validator {
    pub fn new(message: &'static str) -> Self {
        Validator { message }
    }

    pub fn ask(&self) -> bool {
        let mut buttons = ButtonsState::new();

        let cancel = LabelLine::new().dims(128, 11).pos(0, 26).text("Cancel"); 
        let yes = LabelLine::new().dims(128, 11).pos(0, 12)
                                    .text(self.message);

        Bagl::LABELLINE(cancel).display();
        Bagl::LABELLINE(yes.bold()).add();

        let mut response = true;

        loop {
            match get_event(&mut buttons) {
                Some(Event::LeftButtonPress) => {
                    UP_ARROW.add();
                }
                Some(Event::RightButtonPress) => {
                    DOWN_ARROW.add();
                }
                Some(Event::LeftButtonRelease) => {
                    response = true;
                    Bagl::LABELLINE(cancel).display();
                    Bagl::LABELLINE(yes.bold()).add();
                } 
                Some(Event::RightButtonRelease) => {
                    response = false;
                    Bagl::LABELLINE(cancel.bold()).display();
                    Bagl::LABELLINE(yes).add();
                }
                Some(Event::BothButtonsPress) => {
                    let highlighted_bagl = match response {
                        true => yes,
                        false => cancel
                    };
                    Bagl::LABELLINE(highlighted_bagl.bold()).display();
                }
                Some(Event::BothButtonsRelease) => {
                    return response
                }
                _ => ()
            }
        }
    }
}
pub struct Menu<'a> {
    panels: &'a[&'a str],
}

impl<'a> Menu<'a> {
    pub fn new(panels: &'a[&'a str]) -> Self {
        Menu { panels }
    }

    pub fn show(&self) -> usize {
        let mut buttons = ButtonsState::new();

        let lines = [
            LabelLine::new().dims(128, 11).pos(0, 26), 
            LabelLine::new().dims(128, 11).pos(0, 12)
        ];

        Bagl::LABELLINE(lines[0].text(self.panels[1])).display();
        Bagl::LABELLINE(lines[1].text(self.panels[0]).bold()).add();
        UP_ARROW.add();
        DOWN_ARROW.add();

        let mut index = 0;

        loop {
            match get_event(&mut buttons) {
                Some(Event::LeftButtonPress) => {
                    Bagl::ICON(Icon::new(Icons::Up).pos(2, 8)).add();
                }
                Some(Event::RightButtonPress) => {
                    Bagl::ICON(Icon::new(Icons::Down).pos(126-7-2, 8)).add();
                }
                Some(Event::BothButtonsRelease) => {
                    return index 
                }
                Some(x) => {
                    match x {
                        Event::LeftButtonRelease => { 
                           index = index.saturating_sub(1);
                        },
                        Event::RightButtonRelease => { 
                            if index < self.panels.len() - 1 {
                                index += 1;
                            }
                        }
                        _ => ()
                    }
                    BLANK.display();
                    UP_ARROW.add();
                    DOWN_ARROW.add();
                    let a = (index / 2) * 2;
                    for i in a..=a+1 {
                        let d = i-a;
                        match self.panels.get(i) {
                            Some(p) => {
                                if d == index & 1 {
                                    Bagl::LABELLINE(lines[1-d].text(p).bold()).add();
                                } else {
                                    Bagl::LABELLINE(lines[1-d].text(p)).add();
                                }
                            }
                            None => ()
                        }
                    }
                } 
                _ => ()
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
        Bagl::LABELLINE(LabelLine::new().text(self.message)).display();
    }
    /// Display the message and wait
    /// for any kind of button release 
    pub fn show_and_wait(&self) {
        let mut buttons = ButtonsState::new();

        self.show();

        loop {
            match get_event(&mut buttons) {
                Some(Event::LeftButtonRelease) | 
                Some(Event::RightButtonRelease) | 
                Some(Event::BothButtonsRelease) => return,
                _ => ()
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
        let mut cur_idx = 0;
        const CHAR_N: usize = 16;
 
        let label = LabelLine::new(); 
        
        Bagl::LABELLINE(label.text(&self.message[..CHAR_N])).display();
        RIGHT_ARROW.add();

        loop {
            match get_event(&mut buttons) {
                Some(Event::LeftButtonPress) => {
                    Bagl::ICON(Icon::new(Icons::Left).pos(6, 12)).add();
                }
                Some(Event::RightButtonPress) => {
                    Bagl::ICON(Icon::new(Icons::Right).pos(116, 12)).add();
                }
                Some(Event::LeftButtonRelease) => {
                    if cur_idx >= CHAR_N {
                        cur_idx -= CHAR_N; // Otherwise block onto first panel
                    } 
                    
                    RIGHT_ARROW.display();
                    if cur_idx >= CHAR_N {
                        LEFT_ARROW.add();
                    }
                    let upper_bound = (self.message.len() - 1).min(cur_idx + CHAR_N);
                    Bagl::LABELLINE(label.text(&self.message[cur_idx..upper_bound])).add();
                }    
                Some(Event::RightButtonRelease) => {
                    let last_item = self.message.len() - 1 - CHAR_N;
                    if cur_idx < last_item {
                        cur_idx += CHAR_N; // Otherwise block onto last panel
                    }

                    LEFT_ARROW.display();
                    if cur_idx < last_item {
                        RIGHT_ARROW.add();
                    }
                    let upper_bound = self.message.len().min(cur_idx + CHAR_N);
                    Bagl::LABELLINE(label.text(&self.message[cur_idx..upper_bound])).add();
                }
                Some(Event::BothButtonsRelease) => {
                    break;
                }
                Some(_) | None => ()
            }
        }
    } 
}

/// Horizontal scroller that
/// displays a number of Bagls 
/// over the same number of panes
pub struct HScroller<'a> {
    screens: &'a[Bagl<'a>],
}

impl<'a> HScroller<'a> {
    pub fn new(screens: &'a [Bagl<'a>]) -> Self {
        HScroller { screens }
    }

    pub fn event_loop(&self) {
        let mut buttons = ButtonsState::new();
        let mut cur_idx = 0;

        RIGHT_ARROW.display();
        self.screens[cur_idx].add();

        loop {
            match get_event(&mut buttons) {
                Some(Event::LeftButtonPress) => {
                    Bagl::ICON(Icon::new(Icons::Left).pos(6, 12)).add();
                }
                Some(Event::LeftButtonRelease) => {
                    if cur_idx > 0 {
                        cur_idx -= 1; // Otherwise block onto first panel
                    } 

                    RIGHT_ARROW.display();
                    if cur_idx != 0 {
                        LEFT_ARROW.add();
                    }
                    self.screens[cur_idx].add();
                }    
                Some(Event::RightButtonPress) => {
                    Bagl::ICON(Icon::new(Icons::Right).pos(116, 12)).add();
                }
                Some(Event::RightButtonRelease) => {
                    let last_item = self.screens.len() - 1;
                    if cur_idx < last_item {
                        cur_idx += 1; // Otherwise block onto last panel
                    }

                    LEFT_ARROW.display();
                    if cur_idx != last_item {
                        RIGHT_ARROW.add();
                    }
                    self.screens[cur_idx].add();
                }
                Some(Event::BothButtonsRelease) => {
                    break;
                }
                Some(_) | None => ()
            }
        }
    } 
}