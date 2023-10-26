#![allow(dead_code)]

use ledger_sdk_sys::{
    buttons::{get_button_event, ButtonEvent, ButtonsState},
    seph,
};

use nanos_sdk::{io, buttons::ButtonEvent::*};

use crate::{bagls::*, fonts::OPEN_SANS};

use crate::layout;
use crate::layout::{Draw, Location, StringPlace};

use crate::bitmaps::Glyph;

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

pub fn clear_screen() {
    #[cfg(not(target_os = "nanos"))]
    {

        #[cfg(not(feature = "speculos"))]
        unsafe {
            ledger_sdk_sys::screen_clear();
        }

        #[cfg(feature = "speculos")]
        {
            // Speculos does not emulate the screen_clear syscall yet
            RectFull::new()
                .width(crate::SCREEN_WIDTH as u32)
                .height(crate::SCREEN_HEIGHT as u32)
                .erase();
        }
    }

    #[cfg(target_os = "nanos")]
    BLANK.paint();
}

/// Shorthand to display a single message
/// and wait for button action
pub fn popup(message: &str) {
    clear_screen();
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
        clear_screen();
        let mut buttons = ButtonsState::new();

        let mut lines = [Label::from_const("Cancel"), Label::from(self.message)];

        lines[0].bold = true;

        let redraw = |lines_list: &[Label; 2]| {
            clear_screen();
            lines_list.place(Location::Middle, Layout::Centered, false);

            UP_ARROW.display();
            DOWN_ARROW.display();

            crate::screen_util::screen_update();
        };
        redraw(&lines);

        let mut response = false;

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    UP_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    DOWN_S_ARROW.instant_display();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    UP_S_ARROW.erase();
                    response = false;
                    lines[0].bold = true;
                    lines[1].bold = false;
                    lines.place(Location::Middle, Layout::Centered, false);
                    redraw(&lines);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    DOWN_S_ARROW.erase();
                    response = true;
                    lines[0].bold = false;
                    lines[1].bold = true;
                    redraw(&lines);
                }
                Some(ButtonEvent::BothButtonsPress) => {
                    UP_ARROW.erase();
                    DOWN_ARROW.erase();
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

use crate::layout::*;

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
        clear_screen();
        let page_count = &self.message.len() + 2;
        let mut cur_page = 0;

        let draw_icon_and_text = |icon: Icon, strings: &[&str]| {
            // Draw icon on the center if there is no text.
            let x = match strings.len() {
                0 => 60,
                _ => 18,
            };
            icon.set_x(x).display();
            match strings.len() {
                0 => {}
                1 => {
                    strings[0].place(Location::Middle, Layout::Centered, false);
                }
                _ => {
                    strings[..2].place(Location::Middle, Layout::Centered, false);
                }
            }
        };

        let draw = |page: usize| {
            clear_screen();
            if page == page_count - 2 {
                draw_icon_and_text(CHECKMARK_ICON, &self.confirm);
                RIGHT_ARROW.display();
            } else if page == page_count - 1 {
                draw_icon_and_text(CROSS_ICON, &self.cancel);
            } else {
                self.message[page].place(Location::Middle, Layout::Centered, false);
                RIGHT_ARROW.display();
            }
            if page > 0 {
                LEFT_ARROW.display();
            }
            crate::screen_util::screen_update();
        };

        draw(cur_page);

        let mut buttons = ButtonsState::new();
        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    LEFT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    RIGHT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    LEFT_S_ARROW.erase();
                    if cur_page > 0 {
                        cur_page -= 1;
                    }
                    draw(cur_page);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    RIGHT_S_ARROW.erase();
                    if cur_page < page_count - 1 {
                        cur_page += 1;
                    }
                    draw(cur_page);
                }
                Some(ButtonEvent::BothButtonsRelease) => {
                    if cur_page == page_count - 2 {
                        // Confirm
                        return true;
                    } else if cur_page == page_count - 1 {
                        // Abort
                        return false;
                    }
                    draw(cur_page);
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
        clear_screen();
        let mut buttons = ButtonsState::new();

        let mut items: [Label; layout::MAX_LINES] =
            core::array::from_fn(|i| Label::from(*self.panels.get(i).unwrap_or(&"")));

        items[0].bold = true;
        items.place(Location::Middle, Layout::Centered, false);

        UP_ARROW.display();
        DOWN_ARROW.display();

        crate::screen_util::screen_update();

        let mut index = 0;

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    UP_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    DOWN_S_ARROW.instant_display();
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
                    clear_screen();
                    UP_ARROW.display();
                    DOWN_ARROW.display();

                    let chunk = (index / layout::MAX_LINES) * layout::MAX_LINES;
                    for (i, item) in items.iter_mut().enumerate() {
                        item.text = self.panels.get(chunk + i).unwrap_or(&"");
                        item.bold = false;
                    }
                    items[index - chunk].bold = true;
                    items.place(Location::Middle, Layout::Centered, false);
                    crate::screen_util::screen_update();
                }
                _ => (),
            }
        }
    }
}

pub enum PageStyle {
    PictureNormal, // Picture (should be 16x16) with two lines of text (page layout depends on device).
    PictureBold,   // Icon on top with one line of text on the bottom.
    BoldNormal,    // One line of bold text and one line of normal text.
    Normal,        // 2 lines of centered text.
}

pub struct Page<'a> {
    style: PageStyle,
    label: [&'a str; 2],
    glyph: Option<&'a Glyph<'a>>,
}

// new_picture_normal
impl<'a> From<([&'a str; 2], &'a Glyph<'a>)> for Page<'a> {
    fn from((label, glyph): ([&'a str; 2], &'a Glyph<'a>)) -> Page<'a> {
        Page::new(PageStyle::PictureNormal, label, Some(glyph))
    }
}

// new bold normal or new normal
impl<'a> From<([&'a str; 2], bool)> for Page<'a> {
    fn from((label, bold): ([&'a str; 2], bool)) -> Page<'a> {
        if bold {
            Page::new(PageStyle::BoldNormal, label, None)
        }
        else
        {
            Page::new(PageStyle::Normal, label, None)
        }
    }
}

// new picture bold
impl<'a> From<(&'a str, &'a Glyph<'a>)> for Page<'a> {
    fn from((label, glyph): (&'a str, &'a Glyph<'a>)) -> Page<'a> {
        let label = [label, ""];
        Page::new(PageStyle::PictureBold, label, Some(glyph))
    }
}

impl<'a> Page<'a> {
    pub fn new(style: PageStyle, label: [&'a str; 2], glyph: Option<&'a Glyph<'a>>) -> Self {
        Page {
            style,
            label,
            glyph,
        }
    }

    }

    pub fn place(&self) {
        clear_screen();
        match self.style {
            PageStyle::PictureNormal => {
                let mut icon_x = 16;
                let mut icon_y = 8;
                if cfg!(target_os = "nanos") {
                    self.label
                        .place(Location::Middle, Layout::Custom(41), false);
                } else {
                    icon_x = 57;
                    icon_y = 10;
                    self.label
                        .place(Location::Custom(28), Layout::Centered, false);
                }
                match self.glyph {
                    Some(glyph) => {
                        let icon = Icon::from(glyph);
                        icon.set_x(icon_x).set_y(icon_y).display();
                    }
                    None => {}
                }
            }
            PageStyle::PictureBold => {
                let mut icon_x = 56;
                let mut icon_y = 2;
                if cfg!(target_os = "nanos") {
                    self.label[0].place(Location::Bottom, Layout::Centered, true);
                } else {
                    icon_x = 57;
                    icon_y = 17;
                    self.label[0].place(Location::Custom(35), Layout::Centered, true);
                }
                match self.glyph {
                    Some(glyph) => {
                        let icon = Icon::from(glyph);
                        icon.set_x(icon_x).set_y(icon_y).display();
                    }
                    None => {}
                }
            }
            PageStyle::BoldNormal => {
                let padding = 1;
                let total_height = OPEN_SANS[0].height as usize
                    + OPEN_SANS[1].height as usize
                    + 2 * padding as usize;
                let mut cur_y = Location::Middle.get_y(total_height);
                self.label[0].place(Location::Custom(cur_y), Layout::Centered, true);
                cur_y += OPEN_SANS[0].height as usize + 2 * padding as usize;
                self.label[1].place(Location::Custom(cur_y), Layout::Centered, false);
            }
            PageStyle::Normal => {
                self.label.place(Location::Middle, Layout::Centered, false);
            }
        }
    }
}

pub enum EventOrPageIndex {
    Event(io::Event<io::ApduHeader>),
    Index(usize)
}

pub struct MultiPageMenu<'a> {
    comm : &'a mut io::Comm,
    pages: &'a [&'a Page<'a>],
}

impl<'a> MultiPageMenu<'a> {
    pub fn new(comm: &'a mut io::Comm,  pages: &'a [&'a Page]) -> Self {
        MultiPageMenu { comm, pages }
    }

    pub fn show(&mut self) -> EventOrPageIndex {
        clear_screen();
        
        self.pages[0].place();

        LEFT_ARROW.display();
        RIGHT_ARROW.display();

        crate::screen_util::screen_update();

        let mut index = 0;

        loop {
            match self.comm.next_event() {
                io::Event::Button(button) => {
                    match button {
                        LeftButtonPress => {
                            LEFT_S_ARROW.instant_display();
                        }
                        RightButtonPress => {
                            RIGHT_S_ARROW.instant_display();
                        }
                        BothButtonsRelease => return EventOrPageIndex::Index(index), 
                        b => {
                            match b {
                                LeftButtonRelease => {
                                    LEFT_S_ARROW.erase();
                                    if index as i16 - 1 < 0 {
                                        index = self.pages.len() - 1;
                                    } else {
                                        index = index.saturating_sub(1);
                                    }
                                }
                                RightButtonRelease => {
                                    RIGHT_S_ARROW.erase();
                                    if index < self.pages.len() - 1 {
                                        index += 1;
                                    } else {
                                        index = 0;
                                    }
                                }
                                _ => (),
                            }
                            clear_screen();
                            self.pages[index].place();
                            LEFT_ARROW.display();
                            RIGHT_ARROW.display();
                            crate::screen_util::screen_update();
                        }
                    }
                },
                io::Event::Command(ins) => return EventOrPageIndex::Event(io::Event::Command(ins)),
                _ => (),
            };
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
        clear_screen();
        self.message
            .place(Location::Middle, Layout::Centered, false);
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
        clear_screen();
        let mut buttons = ButtonsState::new();
        const CHAR_N: usize = 16;
        let page_count = (self.message.len() - 1) / CHAR_N + 1;
        if page_count == 0 {
            return;
        }
        let mut label = Label::from("");
        let mut cur_page = 0;

        // A closure to draw common elements of the screen
        // cur_page passed as parameter to prevent borrowing
        let mut draw = |page: usize| {
            let start = page * CHAR_N;
            let end = (start + CHAR_N).min(self.message.len());
            let chunk = &self.message[start..end];
            label.erase();
            label.text = &chunk;
            LEFT_ARROW.erase();
            RIGHT_ARROW.erase();
            if page > 0 {
                LEFT_ARROW.display();
            }
            if page + 1 < page_count {
                RIGHT_ARROW.display();
            }
            label.instant_display();
        };

        draw(cur_page);

        loop {
            match get_event(&mut buttons) {
                Some(ButtonEvent::LeftButtonPress) => {
                    LEFT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::RightButtonPress) => {
                    RIGHT_S_ARROW.instant_display();
                }
                Some(ButtonEvent::LeftButtonRelease) => {
                    LEFT_S_ARROW.erase();
                    if cur_page > 0 {
                        cur_page -= 1;
                    }
                    // We need to draw anyway to clear button press arrow
                    draw(cur_page);
                }
                Some(ButtonEvent::RightButtonRelease) => {
                    RIGHT_S_ARROW.erase();
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
