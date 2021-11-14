//Mandelbrot Orbits
extern crate sdl2;

use std::convert::TryInto;
use num::Complex;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Point;
use sdl2::render::RenderTarget;

/*
#[cfg(target_os = "emscripten")]
pub mod emscripten;
*/
//use sdl2::rect::Rect;

//fn screen_to_complex<I,F>(x:I, y:I, w:I, h:I) -> Complex<f64> {
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
    let creator = canvas.texture_creator();
    let mut bg_texture = creator
        .create_texture_target(PixelFormatEnum::RGBA8888, 800, 600)
        .map_err(|e| e.to_string())?;

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
            }).map_err(|e| e.to_string())?;

    //let origin = Point::new(0,0);

    let mut pump = sdl_context.event_pump().unwrap();

    //println!("before main loop");
    'mainloop: loop {
    //let mut main_loop = move || {

        //let event = sdl_context.event_pump()?.wait_event();
        let event = pump.wait_event();
        
        canvas.copy(&bg_texture, None, None).unwrap();
       
        match event {
            Event::KeyDown {keycode: Some(Keycode::Escape),..} | 
            Event::Quit { .. } => { 
                    break 'mainloop 
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
                },
            _ => {}
        }

        canvas.present();
    };
    //println!("Hello, Benoit B. Mandelbrot!");

    println!("Exiting...");
    Ok(()) 
}

fn draw_orbits<T:RenderTarget>(canvas:&mut sdl2::render::Canvas<T>, 
                x: i32, y: i32, w: i32, h:i32) -> Result<(), String> {
    let iter = 50;
    let limit_sqr = 2.0 * 2.0;
    //let origin = Complex{re: 0.0, im: 0.0};
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
