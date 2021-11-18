//Mandelbrot Orbits
extern crate sdl2;

use std::convert::TryInto;
use num::Complex;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Point;
use sdl2::render::TextureCreator;
use sdl2::render::RenderTarget;
use sdl2::video::WindowContext;
//use sdl2::mouse::Cursor;
//use sdl2::surface::Surface;

//const CURSOR_SIZE_BYTES:usize = 11*11*4;

fn screen_to_complex(x:i32, y:i32, w:i32, h:i32) -> Complex<f64> {
    Complex {re: 2.0*x as f64 / w as f64 - 1.5,
             im: 2.0*y as f64 / h as f64 - 1.0}
}

fn complex_to_screen(c:Complex<f64>, w:i32, h:i32) -> Point {
    let Complex{re, im} = c;
    Point::new(((re+1.5)*(w as f64)/2.0) as i32,
               ((im+1.0)*(h as f64)/2.0) as i32)
}

fn main() -> Result<(), String> {

    #[cfg(target_os = "emscripten")]
    {
        let h1 = sdl2::hint::get("SDL_EMSCRIPTEN_ASYNCIFY");
        let h2 = sdl2::hint::set("SDL_EMSCRIPTEN_ASYNCIFY","1");
        let h3 = sdl2::hint::get("SDL_EMSCRIPTEN_ASYNCIFY");
        println!("h1:{:?}, h2: {:?}, h3:{:?}",h1,h2,h3);
    }

    //println!("Hello, Benoit B. Mandelbrot!");
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?; 
    let window = video_subsystem
        .window("Mandelbrot Set Orbit Browser", 800,600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().software().build().map_err(|e| e.to_string())?;
    //desktop_display_mode
    //current_display_mode
    let num_displays = video_subsystem.num_video_displays()?;
    println!("num_displays:{}",num_displays);
    for i in 0..num_displays {
        let dm = video_subsystem.current_display_mode(i)?;
        println!("dm:{} x:{}, y:{}",i,dm.w,dm.h);
    }
    let creator = canvas.texture_creator();
    let (initial_x,initial_y) = (800,600);
    let mut bg_texture = update_bg(&mut canvas, &creator, initial_x, initial_y);

    //Seems like the "surface" cursor is slowing things down in the browser.  Investigate further
    //Is it "software" rendering instead of a hardware accelerated "texture"?
/*
    let mut cursor_raw = cursor_pixels();
    let cursor_surface = Surface::from_data(&mut cursor_raw, 11, 11, 4*11, PixelFormatEnum::RGBA8888).unwrap();
    let potential_cursor = Cursor::from_surface(&cursor_surface,5,5);
    let cursor = match potential_cursor {
        Ok(cursor) => cursor,
        _ => panic!("cursor failed!")
    };
    cursor.set();
*/
    //Maybe try a SystemCursor::Crosshair, or SystemCursor::No
/*
    //From C-SDL
    SDL_Cursor* cursor;
    cursor = SDL_CreateSystemCursor(SDL_SYSTEM_CURSOR_HAND);
    SDL_SetCursor(cursor);
*/
    let mut pump = sdl_context.event_pump().unwrap();

    'mainloop: loop {

        let mut potential_event = Some(pump.wait_event()); //Blocking call will always succeed
        
        canvas.copy(&bg_texture, None, None).unwrap();

        while let Some(event) = potential_event {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Escape),..} | 
                Event::Quit { .. } => { 
                        break 'mainloop 
                    },
                Event::KeyDown {keycode: Some(Keycode::F),..} => { 
                    //"F" -> full screen mode
                    canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Desktop)?;
                    },
                Event::MouseMotion {x, y, .. } | 
                Event::MouseButtonUp {x,y, .. } |
                Event::MouseButtonDown {x,y, .. } => {
                        let (w1,h1) = canvas.viewport().size();
                        draw_orbits(&mut canvas,x,y,w1.try_into().unwrap(),h1.try_into().unwrap()).unwrap();
                        {}},
                Event::FingerDown {x, y, .. } |
                Event::FingerMotion {x, y, .. } |
                Event::FingerUp {x, y, .. }  => {
                        let (w1,h1) = canvas.viewport().size();
                        let x = (x*w1 as f32).floor() as i32;
                        let y = (y*h1 as f32).floor() as i32;
                        draw_orbits(&mut canvas,x,y,w1.try_into().unwrap(),h1.try_into().unwrap()).unwrap();
                        {}},
                Event::MultiGesture {x, y, d_dist, num_fingers, .. }  => {
                        if num_fingers == 2 {
                            println!("Touch Zoom {} @ ({},{})",if d_dist>0.0 {"in"} else {"out"},x,y);
                        }
                    },
                Event::MouseWheel {y, .. } => {
                        let mouse_state = pump.mouse_state();
                        let pos = (mouse_state.x(),mouse_state.y());     
                        println!("Zoom {} @ {:?}",if y>0 {"in"} else {"out"},pos);
                    },
                Event::Window {win_event: WindowEvent::SizeChanged(x,y), .. } => { 
                        println!("Got Size change -- x:{}, y:{}",x,y);
                        let new_size = canvas.viewport().size();
                        let nx = new_size.0;
                        let ny = new_size.1;
                        bg_texture = update_bg(&mut canvas, &creator, nx, ny);
                    },
                _ => {}
            } //match event
            potential_event = pump.poll_event();
        } //while
        canvas.present();
    };

    println!("Exiting...");
    Ok(()) 
}

fn draw_orbits<T:RenderTarget>(canvas:&mut sdl2::render::Canvas<T>, 
                x: i32, y: i32, w: i32, h:i32) -> Result<(), String> {
    let iter = 50;
    let limit_sqr = 2.0 * 2.0;
    let c = screen_to_complex(x,y,w,h);
    let mut z = Complex{re: 0.0, im: 0.0};
   
    for i in 0 .. iter {
        let z_next = z*z + c;
        if z_next.norm_sqr() > limit_sqr {
            break;
        }
        let p1 = complex_to_screen(z,w,h);
        let p2 = complex_to_screen(z_next,w,h);

        if i == 0 {
            canvas.set_draw_color(Color::RGBA(255,0,0,255));
        } else {
            canvas.set_draw_color(Color::RGBA(0,255,0,255));
        }
        canvas.draw_line(p1,p2)?;
        z = z_next;
    }

    Ok(())
}

fn update_bg<'a>(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: &'a TextureCreator<WindowContext>, win_x:u32, win_y:u32) -> sdl2::render::Texture<'a> {
    let mut bg_texture = texture_creator
        .create_texture_target(PixelFormatEnum::RGBA8888, win_x, win_y)
        .map_err(|e| e.to_string()).unwrap();

    canvas.with_texture_canvas(&mut bg_texture, |texture_canvas| {
            let (w1,h1) = texture_canvas.viewport().size();
            let w:i32 = w1.try_into().unwrap();
            let h:i32 = h1.try_into().unwrap();

            texture_canvas.set_draw_color(Color::RGBA(255,255,255,255));
            texture_canvas.clear();

            for y in 0 .. h {
                //println!("y: {}",y);
                for x in 0 .. w {
                    let c = screen_to_complex(x,y,w,h);
                    let mut z = Complex::<f64>{re: 0.0, im: 0.0};

                    for _i in 0 .. 50 {
                        z = z*z + c;
                        if z.norm_sqr() > 4.0 { break; }
                    }

                    if z.norm_sqr() > 4.0 {
                        texture_canvas.set_draw_color(Color::RGBA(255,255,255,255));
                    }
                    else {
                        texture_canvas.set_draw_color(Color::RGBA(0,0,0,255));
                    }
                    //Maybe do something better to get the complier to shut up
                    //maybe panic if draw_point fails
                    let _foo = texture_canvas.draw_point(Point::new(x,y));
                }
            }
            }).map_err(|e| e.to_string()).unwrap();
    bg_texture
}

/*
fn cursor_pixels() -> [u8; CURSOR_SIZE_BYTES] {

    //Change mouse cursor to 11x11 pixel crosshairs
    let mut cursor_raw:[u8; CURSOR_SIZE_BYTES] = [0; CURSOR_SIZE_BYTES];
    for i in 0..11 {
        //vertical
        cursor_raw[i*11*4+20] = 255; //Alpha
        cursor_raw[i*11*4+21] = 255; //Blue
        cursor_raw[i*11*4+22] = 128; //Green
        cursor_raw[i*11*4+23] = 0; //Red
        //horizontal
        cursor_raw[5*11*4+i*4+0] = 255;
        cursor_raw[5*11*4+i*4+1] = 255;
        cursor_raw[5*11*4+i*4+2] = 128;
        cursor_raw[5*11*4+i*4+3] = 0;
    }
    cursor_raw
}
*/