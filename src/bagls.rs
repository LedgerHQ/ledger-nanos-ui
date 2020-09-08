#![allow(dead_code)] 

use nanos_sdk::seph;
use nanos_sdk::seph::SephTags;

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
pub enum Icons {
  Gears = 3,
  Clear,
  Backspace,
  Check,
  Cross,
  CheckBadge,
  Left,
  Right,
  Up,
  Down,
  MiniLedger,
  CrossBadge,
  Dashboard,
  Plus,
  Less,
  ToggleON,
  ToggleOFF,
  Loading,
  Cog,
  Warning,
  Download,
  Transaction,
  Bitcoin,
  Ethereum,
  Eye,
  People,
  Lock,
}

pub const BAGL_FONT_ALIGNMENT_CENTER: u32 = 32768;

#[repr(C)]
#[derive(Copy, Clone)]
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


/// A Rust version of seph_display
/// 'PIC()' is not used, and not necessary 
/// as long as `const` bagls use no pointers
/// (no text) or are const fn's
fn io_seproxyhal_display(bagl: bagl_element_rs) {

  // let bagl = pic_rs(&bagl_);
  if seph::is_status_sent() {
      // TODO: this does not seem like the right way to fix the problem...
    let mut spi_buffer = [0u8; 16]; 
    seph::seph_recv(&mut spi_buffer, 0); 
  }

  let bagl_comp = unsafe { core::slice::from_raw_parts(&bagl.component 
                              as *const BaglComponent 
                              as *const u8,
                              core::mem::size_of::<BaglComponent>()) };

  match bagl.text {
    Some(txt) => {
      let lenbytes = ((bagl_comp.len() + txt.len()) as u16).to_be_bytes();
      seph::seph_send(&[SephTags::ScreenDisplayStatus as u8, lenbytes[0], lenbytes[1]]);
      seph::seph_send(bagl_comp);
      seph::seph_send(txt.as_bytes());
    }
    None => {
      seph::seph_send(&[SephTags::ScreenDisplayStatus as u8, 0, bagl_comp.len() as u8]);
      seph::seph_send(bagl_comp);
    }
  }
}


#[derive(Copy, Clone)]
pub enum Bagl<'a> {
  // BUTTON(Button),
  // LABEL(Label),
  LABELLINE(LabelLine<'a>),
  RECT(Rect),
  // LINE(Line),
  ICON(Icon),
  // CIRCLE(Circle),
}


impl Bagl<'_> {
  /// Erase screen and display the bagl
  pub fn display(&self) {
    BLANK.add();
    io_seproxyhal_display(bagl_element_rs::from(*self));
  }

  /// Only add to current screen (draw over)
  pub fn add(&self) {
    io_seproxyhal_display(bagl_element_rs::from(*self));
  }
}

#[repr(C)]
pub struct bagl_element_rs<'a> {
    pub component: BaglComponent,
    pub text: Option<&'a str>,
}

impl<'a> From<Bagl<'a>> for bagl_element_rs<'a> {
  fn from(x: Bagl<'a>) -> bagl_element_rs<'a> {
    match x {
      // Bagl::LABEL(y) => bagl_element_rs::from(y),
      Bagl::LABELLINE(y) => bagl_element_rs::from(y),
      Bagl::RECT(y) => bagl_element_rs::from(y),
      // Bagl::LINE(y) => bagl_element_rs::from(y),
      Bagl::ICON(y) => bagl_element_rs::from(y),
      // Bagl::CIRCLE(y) => bagl_element_rs::from(y),
      // _ => bagl_element_rs::default()
    }
  }
}

#[derive(Copy, Clone)]
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

  pub const fn dims(self, w: u16, h: u16) -> Self {
    Icon {dims: (w,h), ..self}
  }
}

impl<'a> From<Icon> for bagl_element_rs<'a> {
  fn from(icon: Icon) -> bagl_element_rs<'a> {
    bagl_element_rs {
      component: BaglComponent {
        type_: BaglTypes::Icon as u8,
        userid: 0,
        x: icon.pos.0,
        y: icon.pos.1,
        width: icon.dims.0,
        height: icon.dims.1,
        stroke: 0,
        radius: 0,
        fill: 0,
        fgcolor: 0xffffffu32,
        bgcolor: 0,
        font_id: 0,
        icon_id: icon.glyph_id,
      },
      text: None,
    }
  }
}

#[derive(Copy,Clone)]
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

#[derive(Copy, Clone)]
pub struct LabelLine<'a> {
  pub pos: (i16, i16),
  pub dims: (u16, u16),
  pub font_id: Font,
  pub text: Option<&'a str>
}

impl<'a> LabelLine<'a> {
  pub const fn new() -> Self {
    LabelLine {
      pos: (0, 16),
      dims: (128, 8),
      font_id: Font::OpenSansRegular11px,
      text: None
    }
  }

  pub const fn pos(self, x: i16, y: i16) -> Self {
    LabelLine {pos: (x,y), ..self}
  }
  pub const fn dims(self, w: u16, h: u16) -> Self {
    LabelLine {dims: (w,h), ..self}
  }
  pub const fn bold(self) -> Self {
    LabelLine {font_id: Font::OpenSansExtrabold11px, ..self } 
  }
  /// TODO:This one won't display
  // pub const fn light(self) -> Self {
  //   LabelLine {font_id: Font::OpenSansLight16px, ..self } 
  // }
  pub const fn font(self, font_id: Font) -> Self {
    LabelLine {font_id, ..self}
  }
  pub fn text(self, m: &'a str) -> Self {
    LabelLine {text: Some(m), ..self}
  }
}

impl<'a> From<LabelLine<'a>> for bagl_element_rs<'a> {
  fn from(labelline: LabelLine<'a>) -> bagl_element_rs<'a> {
    bagl_element_rs {
      component: BaglComponent {
        type_: BaglTypes::LabelLine as u8,
        userid: 0,  // FIXME
        x: labelline.pos.0,
        y: labelline.pos.1,
        width: labelline.dims.0,
        height: labelline.dims.1,
        stroke: 0,
        radius: 0,
        fill: 0,
        fgcolor: 0xffffffu32,
        bgcolor: 0,
        font_id: labelline.font_id as u16 | BAGL_FONT_ALIGNMENT_CENTER as u16,
        icon_id: 0,
      },
      text: labelline.text,
    }
  }
}
#[derive(Copy, Clone)]
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

impl<'a> From<Rect> for bagl_element_rs<'a> {
  fn from(rect: Rect) -> bagl_element_rs<'a> {
    bagl_element_rs {
      component: BaglComponent {
        type_: BaglTypes::Rectangle as u8,
        userid: rect.userid,
        x: rect.pos.0,
        y: rect.pos.1,
        width: rect.dims.0,
        height: rect.dims.1,
        stroke: 0,
        radius: 0,
        fill: rect.fill as u8,
        fgcolor: rect.colors.0,
        bgcolor: rect.colors.1,
        font_id: 0,
        icon_id: 0,
      },
      text: None,
    }
  }
}


/// Some common constant Bagls
pub const BLANK: Bagl<'static> = Bagl::RECT(Rect::new().pos(0,0).dims(128, 32).colors(0, 0xffffff).fill(true));

pub const LEFT_ARROW: Bagl = Bagl::ICON(Icon::new(Icons::Left).pos(2, 12));
pub const RIGHT_ARROW: Bagl = Bagl::ICON(Icon::new(Icons::Right).pos(120, 12));
pub const UP_ARROW: Bagl = Bagl::ICON(Icon::new(Icons::Up).pos(2, 12));
pub const DOWN_ARROW: Bagl = Bagl::ICON(Icon::new(Icons::Down).pos(126-7-2, 12));