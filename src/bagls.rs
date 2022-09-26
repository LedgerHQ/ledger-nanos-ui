
use nanos_sdk::seph;
use nanos_sdk::seph::SephTags;
use crate::layout::{Draw, Location, Layout};

#[repr(u8)]
pub enum BaglTypes {
  NoneType = 0,
  Button = 1,
  Label = 2,
  Rectangle = 3,
  Line = 4,
  Icon = 5,
  Circle = 6,
  LabelLine = 7,
}

#[repr(u8)]
/// All icons available on the Ledger Nano-S
pub enum Icons {
  Check = 6,
  Cross,
  CheckBadge,
  Left,
  Right,
  Up,
  Down,
  CrossBadge = 14,
  TransactionBadge = 24,
  EyeBadge = 27,
}

pub const BAGL_FONT_ALIGNMENT_CENTER: u32 = 32768;

#[repr(C)]
pub struct BaglComponent {
    pub type_: u8,
    pub userid: u8,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub stroke: u8,
    pub radius: u8,
    pub fill: u8,
    pub fgcolor: u32,
    pub bgcolor: u32,
    pub font_id: u16,
    pub icon_id: u8,
}

impl BaglComponent {
  pub fn paint(&self) {
    let bagl_comp = unsafe { core::slice::from_raw_parts(self
                              as *const BaglComponent 
                              as *const u8,
                              core::mem::size_of::<BaglComponent>()) };

    seph::seph_send(&[SephTags::ScreenDisplayStatus as u8, 0, bagl_comp.len() as u8]);
    seph::seph_send(bagl_comp);
  }
}

pub trait SendToDisplay {
  fn wait_for_status(&self) {
    if seph::is_status_sent() {
      // TODO: this does not seem like the right way to fix the problem...
      let mut spi_buffer = [0u8; 16]; 
      seph::seph_recv(&mut spi_buffer, 0); 
    }
  }
  fn paint(&self);
  fn send_to_display(&self) {
    BLANK.paint();
    self.paint();
  }
}

pub enum Bagl<'a> {
  LABELLINE(Label<'a>),
  RECT(Rect),
  ICON(Icon),
}


impl Bagl<'_> {
  /// Erase screen and display the bagl
  pub fn display(&self) {
    match self {
      Bagl::LABELLINE(x) => x.send_to_display(),
      Bagl::RECT(x) => x.send_to_display(),
      Bagl::ICON(x) => x.send_to_display(),
    }
  }

  /// Only paint to current screen (draw over)
  pub fn paint(&self) {
    match self {
      Bagl::LABELLINE(x) => x.paint(),
      Bagl::RECT(x) => x.paint(),
      Bagl::ICON(x) => x.paint(),
    }
  }
}

#[repr(C)]
pub struct bagl_element_rs<'a> {
    pub component: BaglComponent,
    pub text: Option<&'a str>,
}

pub struct Icon {
  pub pos: (i16, i16),
  pub dims: (u16, u16),
  pub glyph_id: u8
}

impl Icon {
  pub const fn new(icon_id: Icons) -> Icon {
    Icon {
      pos: (12, 12),
      dims: (8, 8),
      glyph_id: icon_id as u8
    }
  }
  pub const fn icon(self, id: u8) -> Self {
    Icon {glyph_id: id, ..self}
  }
  
  pub const fn pos(self, x: i16, y: i16) -> Self {
    Icon {pos: (x,y), ..self}
  }

  pub const fn set_x(self, x: i16) -> Self {
    Icon {pos: (x,self.pos.1), ..self}
  }

  pub const fn dims(self, w: u16, h: u16) -> Self {
    Icon {dims: (w,h), ..self}
  }
}

impl Draw for Icon {
  fn display(&self) {
    self.paint();
  }
  fn erase(&self) {
    Rect::new()
          .pos(self.pos.0, self.pos.1)
          .dims(self.dims.0, self.dims.1)
          .colors(0, 0xffffff)
          .fill(true)
          .paint();
  }
}

#[repr(u8)]
pub enum Font {
  LucidaConsole8px = 0,
  OpenSansLight16_22px,
  OpenSansRegular8_11px,
  OpenSansRegular10_13px,
  OpenSansRegular11_14px,
  OpenSansRegular13_18px,
  OpenSansRegular22_30px,
  OpenSansSemibold8_11px,
  OpenSansExtrabold11px,
  OpenSansLight16px, 
  OpenSansRegular11px, 
  OpenSansSemibold10_13px, 
  OpenSansSemibold11_16px, 
  OpenSansSemibold13_18px, 
  Symbols0,
  Symbols1,
}

pub struct Label<'a> {
  pub loc: Location,
  pub layout: Layout,
  pub dims: (u16, u16),
  pub bold: bool,
  pub text: &'a str
}

impl<'a> Label<'a> {
  pub const fn new() -> Self {
    Label {
      loc: Location::Middle, 
      layout: Layout::Centered,
      dims: (128, 11),
      bold: false,
      text: ""
    }
  }

  pub const fn from_const(text: &'static str) -> Self {
    Label {
      loc: Location::Middle,
      layout: Layout::Centered,
      dims: (128, 11),
      bold: false,
      text 
    }
  }

  pub const fn location(self, loc: Location) -> Self {
    Label {loc, ..self}
  }
  pub const fn layout(self, layout: Layout) -> Self {
    Label {layout, ..self}
  }
  pub const fn dims(self, w: u16, h: u16) -> Self {
    Label {dims: (w,h), ..self}
  }
  pub const fn bold(self) -> Self {
    Label {bold: true, ..self } 
  }
  pub fn text(self, text: &'a str) -> Self {
    Label {text, ..self}
  }
}

impl<'a> From<&'a str> for Label<'a> {
  fn from(text: &'a str) -> Label<'a> {
    Label::new().text(text)
  }
}

impl<'a> Draw for Label<'a> {
  fn display(&self) {
    self.paint();
  }
  fn erase(&self) {
    let x = self.layout.get_x(self.dims.0 as usize) as i16;
    let y = self.loc.get_y(self.dims.1 as usize) as i16;
    Rect::new()
          .pos(x, y)
          .dims(self.dims.0, self.dims.1)
          .colors(0, 0xffffff)
          .fill(true)
          .paint();
  }
}

pub struct Rect {
  pub pos: (i16,i16),
  pub dims: (u16,u16),
  pub colors: (u32, u32), 
  pub fill: bool,
  pub userid: u8
}

impl Rect {
  pub const fn new() -> Rect {
    Rect {pos: (32-5, 64-5), dims: (10,10), colors: (0xffffffu32, 0), fill:false, userid:0}
  }
  pub const fn pos(self, x: i16, y: i16) -> Rect {
    Rect {pos: (x,y), ..self}
  }
  pub const fn colors(self, fg: u32, bg: u32) -> Rect {
    Rect {colors: (fg,bg), ..self}
  }
  pub const fn dims(self, w: u16, h: u16) -> Rect {
    Rect {dims: (w,h), ..self}
  }
  pub const fn fill(self, x: bool) -> Rect {
    Rect {fill: x, ..self}
  }
  pub const fn userid(self, id: u8) -> Rect {
    Rect {userid: id, ..self}
  }
}

impl Draw for Rect {
  fn display(&self) {
    self.paint();
  }
  fn erase(&self) {
    Rect::new()
          .pos(self.pos.0, self.pos.1)
          .dims(self.dims.0, self.dims.1)
          .colors(0, 0xffffff)
          .fill(true)
          .paint();
  }
}

impl SendToDisplay for Icon {
  fn paint(&self) {
    self.wait_for_status();
    let baglcomp = BaglComponent {
      type_: BaglTypes::Icon as u8,
      userid: 0,
      x: self.pos.0,
      y: self.pos.1,
      width: self.dims.0,
      height: self.dims.1,
      stroke: 0,
      radius: 0,
      fill: 0,
      fgcolor: 0xffffffu32,
      bgcolor: 0,
      font_id: 0,
      icon_id: self.glyph_id,
    };
    baglcomp.paint();
  }
}

impl SendToDisplay for Rect {
  fn paint(&self) {
    self.wait_for_status();
    let baglcomp = BaglComponent {
      type_: BaglTypes::Rectangle as u8,
      userid: self.userid,
      x: self.pos.0,
      y: self.pos.1,
      width: self.dims.0,
      height: self.dims.1,
      stroke: 0,
      radius: 0,
      fill: self.fill as u8,
      fgcolor: self.colors.0,
      bgcolor: self.colors.1,
      font_id: 0,
      icon_id: 0,
    };
    baglcomp.paint();
  }
}

use core::ffi::c_void;

impl<'a> SendToDisplay for Label<'a> {
  fn paint(&self) {
    self.wait_for_status();
    let font_id = if self.bold {
      Font::OpenSansExtrabold11px
    } else {
      Font::OpenSansRegular11px
    };
    let x = match self.layout {
      Layout::RightAligned => self.layout.get_x(self.text.len() * 7),
      _ => 0
    };
    let y = self.loc.get_y(self.dims.1 as usize) as i16;
    let width = match self.layout {
      Layout::Centered => crate::SCREEN_WIDTH,
      _ => self.text.len() * 6,
    };
    let baglcomp = BaglComponent {
      type_: BaglTypes::LabelLine as u8,
      userid: 0,  // FIXME
      x: x as i16,
      y: y - 1 + self.dims.1 as i16,
      width: width as u16, //self.dims.0,
      height: self.dims.1,
      stroke: 0,
      radius: 0,
      fill: 0,
      fgcolor: 0xffffffu32,
      bgcolor: 0,
      font_id: font_id as u16 | BAGL_FONT_ALIGNMENT_CENTER as u16,
      icon_id: 0,
    };

    let bagl_comp = unsafe { core::slice::from_raw_parts(&baglcomp
                              as *const BaglComponent 
                              as *const u8,
                              core::mem::size_of::<BaglComponent>()) };
    let lenbytes = ((bagl_comp.len() + self.text.len()) as u16).to_be_bytes();
    seph::seph_send(&[SephTags::ScreenDisplayStatus as u8, lenbytes[0], lenbytes[1]]);
    seph::seph_send(bagl_comp);


    unsafe {
      let pic_text = nanos_sdk::bindings::pic(self.text.as_ptr() as *mut u8 as *mut c_void);
      nanos_sdk::bindings::io_seph_send(pic_text as *mut u8, self.text.len() as u16);
    }
  }
}


/// Some common constant Bagls
pub const BLANK: Rect = Rect::new().pos(0,0).dims(128, 32).colors(0, 0xffffff).fill(true);

pub const LEFT_ARROW: Icon = Icon::new(Icons::Left).pos(0, 12);
pub const RIGHT_ARROW: Icon = Icon::new(Icons::Right).pos(120, 12);
pub const LEFT_S_ARROW: Icon = Icon::new(Icons::Left).pos(4, 12);
pub const RIGHT_S_ARROW: Icon = Icon::new(Icons::Right).pos(116, 12);
pub const UP_ARROW: Icon = Icon::new(Icons::Up).pos(2, 12);
pub const DOWN_ARROW: Icon = Icon::new(Icons::Down).pos(117, 12);
pub const UP_S_ARROW: Icon = Icon::new(Icons::Up).pos(2, 8);
pub const DOWN_S_ARROW: Icon = Icon::new(Icons::Down).pos(117, 8);

pub const CHECKMARK_ICON: Icon = Icon::new(Icons::CheckBadge);
pub const CROSS_ICON: Icon = Icon::new(Icons::CrossBadge);
