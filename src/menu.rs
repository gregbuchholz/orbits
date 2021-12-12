use sdl2;
use sdl2::{render::{TextureCreator, Texture}, video::WindowContext};
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::rect::{Point, Rect};
use std::path::Path;
use sdl2::surface::Surface;
use sdl2::ttf::Sdl2TtfContext;

pub struct Menu<'a> {
    pub texture: Texture<'a>,
    pub buttons: Vec<(String, Rect)>,
    pub offset_rect: Rect
}

impl <'a> Menu <'a>{
    pub fn init (tc:&'a TextureCreator<WindowContext>, ttf_context:& Sdl2TtfContext) -> Menu<'a> {
        let padding = 10;
        let bg_color = Color::RGBA(245,245,245,230);

        let mut menu_surface = Surface::new(250, 400, PixelFormatEnum::ARGB8888).unwrap();
        menu_surface.fill_rect(None, bg_color).unwrap();

        let mut buttons = Vec::new();

        let font_path = Path::new("assets/DejaVuSansMono.ttf");
        let button_font = ttf_context.load_font(font_path, 16).unwrap();
        let hint_font = ttf_context.load_font(font_path, 14).unwrap();
  
        let menu_text_color = Color::RGBA(240, 170, 0, 255);
        for (y, message) in menu_items().iter().enumerate() {
            let plain_text:String = message.chars().filter(|x|{*x != '_'}).collect();
            let underscored:String = message.chars().map(|x|{if x == '_' {'_'} else {' '}}).collect();
           
            let mut first_q = true;
            for text in vec![plain_text, underscored] { 
                let m = button_font.render(text.as_str())
                    .blended(menu_text_color).unwrap();
                let (width,height) = m.size();
                let displacement:i32 = y as i32 * (height+6) as i32 + padding; 
                let m_rect = Rect::new(padding, displacement, width, height);
                if first_q { buttons.push((text.clone(),m_rect.clone())) };
                m.blit(None,&mut menu_surface,m_rect).unwrap();
                first_q = false;
            }
        }
        
        let hints_text_color = Color::RGBA(120, 120, 120, 255);
        for (y, message) in hints().iter().enumerate() {
            let m = hint_font.render(message)
                .blended(hints_text_color).unwrap();
            let (width,height) = m.size();
            let displacement:i32 = y as i32 * (height+6) as i32 + 150; 
            let m_rect = Rect::new(padding, displacement, width, height);
            m.blit(None,&mut menu_surface,m_rect).unwrap();
        }
       
        let menu_offset_x = 10;
        let menu_offset_y = 10;
        let menu_texture = menu_surface.as_texture(tc).unwrap();
        let menu_query = menu_texture.query();

        Menu { 
            texture: menu_texture,
            buttons: buttons,
            offset_rect: Rect::new(menu_offset_x, menu_offset_y,menu_query.width, menu_query.height)
        }
    } //init

    pub fn selected(&self,mouse_x: i32, mouse_y: i32) -> () {
        //correct for menu offset (where it gets draw on screen)

        let mouse_point = Point::new(mouse_x-self.offset_rect.x, mouse_y-self.offset_rect.y);

        for (name,rect) in self.buttons.iter() {
            if rect.contains_point(mouse_point) {
                println!("Hover over: {}!",name);
            }
        }
        ()
    }
} //impl Menu

fn menu_items() -> Vec<&'static str> {
    vec![
        "_Fullscreen",
        "Display _Coordinates",
        "_Menu",
        "_Quit",
        "_About",
    ]
}

fn hints() -> Vec<&'static str> {
    vec![
        "_________________________",
        "Zoom:",
        "  Scroll Wheel",
        "  +,-",
        "  Touch Pinch",
        "Left Mouse Button -> Pan",
        "Save Orbit:",
        "  Right Mouse Button",
        "  Spacebar",
        "  Touch double tap",
        "Arrow Keys -> Move cursor",
    ]
}

fn about() -> Vec<&'static str> {
    vec![
        "Orbits --",
        "  A Mandelbrot set investigation tool",
        "  https://escriben.org",
        "  ",
        "  Made availble under the GPLv3 license",
        "  ",
        "  Source code can be found at:",
        "    https://github.com/gregbuchholz/orbits",
        "  ",
        "  by Greg Buchholz <greg.buchholz@ymail.com>",
        "  (c) 2021",
        "  ",
    ]
}