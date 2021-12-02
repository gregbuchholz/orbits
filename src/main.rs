//Mandelbrot Orbits
use std::convert::TryInto;
use std::time::{Instant};
use num::Complex;

extern crate sdl2;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Point;
use sdl2::rect::Rect;
use std::path::Path;
use sdl2::render::TextureQuery;
extern crate itertools;
use itertools::Itertools;

const SDL_TOUCH_MOUSEID:u32 = u32::MAX;

const INITIAL_ITERATIONS:u32 = 50;

#[derive(Copy, Clone)]
struct ComplexBBox {
    ll: Complex<f64>,
    ur: Complex<f64>
}

impl ComplexBBox {
    fn screen_to_complex(&self, x:i32, y:i32, w:i32, h:i32) -> Complex<f64> {
        let (x,y,w,h) = (x as f64, y as f64, w as f64, h as f64);
        let (lower, left) = (self.ll.im, self.ll.re);
        let (upper, right) = (self.ur.im, self.ur.re);

        Complex { re: (left + (x/w)*(right-left)),
                  im: (upper+(y/h)*(lower-upper)) }
    }

    fn complex_to_screen(&self, c:Complex<f64>, w:i32, h:i32) -> Point {
        let Complex{re, im} = c;
        let (w,h) = (w as f64, h as f64);
        let (lower, left) = (self.ll.im, self.ll.re);
        let (upper, right) = (self.ur.im, self.ur.re);
        let x = ((re-left)*w/(right-left)) as i32;
        let y = ((im-upper)*h/(lower-upper)) as i32;
        Point::new(x,y)
    }

    fn complex_deltas(&self, w:i32, h:i32, dx:i32, dy:i32) -> Complex<f64> {
        let (w,h) = (w as f64, h as f64);
        let (dx,dy) = (dx as f64, dy as f64);
        let left = self.ll.re;
        let right = self.ur.re;
        let lower = self.ll.im;
        let upper = self.ur.im;

        Complex { re: ((dx/w)*(right-left)),
                  im: ((-dy/h)*(upper-lower)) }
    }

    fn zoom(&self, position:Complex<f64>, scale_factor:f64) -> ComplexBBox {
        let Complex {re:x, im:y} = position;
        let new_lower = y - (y-self.ll.im)*scale_factor;
        let new_upper = new_lower + (self.ur.im - self.ll.im)*scale_factor;
        let new_left = x - (x-self.ll.re)*scale_factor;
        let new_right = new_left + (self.ur.re - self.ll.re)*scale_factor;
        
        ComplexBBox {ll:Complex{re: new_left, im: new_lower},
                     ur:Complex{re: new_right, im: new_upper}} 
    }
}

fn main() -> Result<(), String> {

    #[cfg(target_os = "emscripten")]
    {
        let _ = sdl2::hint::set("SDL_EMSCRIPTEN_ASYNCIFY","1");
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?; 
    let window = video_subsystem
        .window("Mandelbrot Set Orbit Browser", 800,600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().accelerated().build().map_err(|e| e.to_string())?;
    println!("renderer info: {:?}",canvas.info());
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    //desktop_display_mode
    //current_display_mode
    /*
    let num_displays = video_subsystem.num_video_displays()?;
    println!("num_displays:{}",num_displays);
    for i in 0..num_displays {
        let dm = video_subsystem.current_display_mode(i)?;
        println!("dm:{} x:{}, y:{}",i,dm.w,dm.h);
    }*/
    let creator = canvas.texture_creator();
    let (initial_width,initial_height) = (800,600);
    
    let j = Complex {re: 0.0, im: 1.0};
    let initial_view = ComplexBBox { ll: -1.5-j, ur: 0.5+j }; 
    let mut view = initial_view; 
    let mut iterations = INITIAL_ITERATIONS;
    
    let initial_bg_rect = Rect::new(0,0, initial_width, initial_height);
    let mut bg_rect_dest = initial_bg_rect.clone();
    let mut bg_rect_src = initial_bg_rect.clone(); 

    let mut bg_texture = creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, initial_width, initial_height)
        .map_err(|e| e.to_string()).unwrap();
    update_bg(&mut bg_texture, &view, iterations);

    let mut drag_x:i32 = 0_i32;
    let mut drag_y:i32 = 0;
    
    let font_path = Path::new("assets/DejaVuSansMono.ttf");
    let font = ttf_context.load_font(font_path, 12)?;

    let red = Color::RGBA(255,0,0,255);
    let green = Color::RGBA(0,255,0,255);
    let _blue = Color::RGBA(0,0,255,255);
    let cyan = Color::RGBA(0,255,255,255);
    let magenta = Color::RGBA(255,0,255,255);
    let white = Color::RGBA(255,255,255,255);

    let mut show_coords_q = true;
    let mut touch_zoom_in_progress = false;
    let mut touch_zoom_pos = Point::new(0,0);

    let mut pump = sdl_context.event_pump().unwrap();
    let mut position = Complex { re:0.0, im:0.0 };
    let mut saved_orbits:Vec<Complex<f64>> = Vec::new();

    'mainloop: loop {

        let mut potential_event = Some(pump.wait_event()); //Blocking call will always succeed
        
        while let Some(event) = potential_event {
            let win_size = canvas.viewport().size();
            let win_width:i32 = win_size.0.try_into().unwrap(); 
            let win_height:i32 = win_size.1.try_into().unwrap(); 

            match event {
                Event::KeyDown {keycode: Some(Keycode::Escape),..} | 
                Event::Quit { .. } => { 
                        break 'mainloop 
                    },
                Event::KeyDown {keycode: Some(Keycode::C),..} => { 
                    show_coords_q = !show_coords_q;
                    },
                Event::KeyDown {keycode: Some(Keycode::I),..} => { 
                    iterations *= 2;
                    update_bg(&mut bg_texture, &view, iterations);
                    },
                Event::KeyDown {keycode: Some(Keycode::F),..} => { 
                    //"F" -> full screen mode
                    //investigate "full screen" in browser, seems to be less than full resolution
                    //suspicously 20% lower: (1138 x 640) instead of (1366 x 768)
                    canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Desktop)?;
                    },
                Event::KeyDown {keycode: Some(Keycode::Home),..} => { 
                    view = initial_view;
                    iterations = INITIAL_ITERATIONS;
                    bg_rect_src = Rect::new(0,0,win_size.0,win_size.1); 
                    bg_rect_dest = Rect::new(0,0,win_size.0,win_size.1);
                    update_bg(&mut bg_texture, &view, iterations);
                    },
                Event::KeyDown {keycode: Some(Keycode::Right), repeat:_, ..} => {
                    let mouse_state = pump.mouse_state(); let (mx,my) = (mouse_state.x(),mouse_state.y());     
                    sdl_context.mouse().warp_mouse_in_window(canvas.window(),mx+1,my);
                    }, 
                Event::KeyDown {keycode: Some(Keycode::Left), repeat:_, ..} => {
                    let mouse_state = pump.mouse_state(); let (mx,my) = (mouse_state.x(),mouse_state.y());     
                    sdl_context.mouse().warp_mouse_in_window(canvas.window(),mx-1,my);
                    }, 
                Event::KeyDown {keycode: Some(Keycode::Up), repeat:_, ..} => {
                    let mouse_state = pump.mouse_state(); let (mx,my) = (mouse_state.x(),mouse_state.y());     
                    sdl_context.mouse().warp_mouse_in_window(canvas.window(),mx,my-1);
                    }, 
                Event::KeyDown {keycode: Some(Keycode::Down), repeat:_, ..} => {
                    let mouse_state = pump.mouse_state(); let (mx,my) = (mouse_state.x(),mouse_state.y());     
                    sdl_context.mouse().warp_mouse_in_window(canvas.window(),mx,my+1);
                    }, 
                Event::MouseButtonUp {which, mouse_btn, .. } if which != SDL_TOUCH_MOUSEID => {
                    //recalculate new view bounding box
                    if mouse_btn == MouseButton::Left {
                        let shift = view.complex_deltas(win_width, win_height, drag_x, drag_y);
                        view = ComplexBBox {ll: view.ll-shift, ur: view.ur-shift};
                        bg_rect_dest = Rect::new(0, 0, win_size.0, win_size.1);//reset bg_rect
                        update_bg(&mut bg_texture, &view, iterations);
                        let _state = pump.relative_mouse_state(); //reset relative coordinates
                        drag_x = 0;
                        drag_y = 0;
                    }
                    },
                Event::MouseButtonDown{which, mouse_btn, .. } if which != SDL_TOUCH_MOUSEID => {
                    match mouse_btn {
                        MouseButton::Left => {
                            let _state = pump.relative_mouse_state(); //reset relative coordinates in SDL land
                            drag_x = 0;
                            drag_y = 0;
                        },
                        MouseButton::Right => {
                            let mouse_state = pump.mouse_state();
                            let (mx,my) = (mouse_state.x(),mouse_state.y());     
                            println!("right button down");
                            let c = view.screen_to_complex(mx,my,win_width,win_height);
                            saved_orbits = calc_orbits(c);
                        },
                        _ => { println!("unhandeled mouse button"); }
                    }
                    },
                Event::MouseMotion {x, y, which, .. } if which != SDL_TOUCH_MOUSEID => {
                    if pump.mouse_state().is_mouse_button_pressed(MouseButton::Left) {
                        let state = pump.relative_mouse_state();
                        drag_x += state.x();
                        drag_y += state.y();
                        bg_rect_dest.set_x(bg_rect_dest.x() + state.x());
                        bg_rect_dest.set_y(bg_rect_dest.y() + state.y());
                    } else {
                        position = view.screen_to_complex(x, y, win_width, win_height);
                    {}}},
                Event::FingerDown {x, y, .. } |
                Event::FingerMotion {x, y, .. } => {
                        let _ignore = (x,y);
                        //println!("event: {:?}",event);
                        /*
                        if !touch_zoom_in_progress {
                            //orbit_points = calc_orbits(x,y,w1.try_into().unwrap(),h1.try_into().unwrap(),&view);
                        }*/
                        {}},
                Event::FingerUp {x:_, y:_, .. }  => {
                        if touch_zoom_in_progress {
                            touch_zoom_in_progress = false;
                            let (zx,zy) = (touch_zoom_pos.x(),touch_zoom_pos.y());
                            let complex_pos = view.screen_to_complex(zx, zy, win_width, win_height);
                            let zoomies = (bg_rect_src.width() as f64) / (bg_rect_dest.width() as f64);//if y>0 {0.5} else {2.0};
                            view = view.zoom(complex_pos,zoomies);
                            bg_rect_dest = Rect::new(0,0,win_size.0,win_size.1);
                            bg_rect_src = Rect::new(0,0,win_size.0,win_size.1);
                            update_bg(&mut bg_texture, &view, iterations);
                        }
                        {}},
                Event::MultiGesture {x, y, d_dist, num_fingers, .. }  => {
                        if num_fingers == 2 {
                            touch_zoom_in_progress = true;
                            let x = (x*win_width as f32).floor() as i32;
                            let y = (y*win_height as f32).floor() as i32;
                            touch_zoom_pos = Point::new(x,y);
                            //println!("Touch Zoom {}: {:.4} @ ({},{})",if d_dist>0.0 {"in "} else {"out"},d_dist,x,y);
                                                                           
                            //rescale image until FingerUp, then recalculate bg_texture, and reset 
                            // bg_rect_src, bg_rect_dest, etc.
                            /*Copies a portion of the texture to the current rendering target.
                                If src is None, the entire texture is copied.
                                If dst is None, the texture will be stretched to fill the given rectangle.*/
                            //todo also do panning with two fingers 
                            //fix to only adjust one rect if the other hasn't yet been modified.  i.e. pinch zoom isn't finished yet
                            if d_dist>0.0 { //zoom in
                                let new_width = ((bg_rect_src.width() as f32) * (1.0-10.0*d_dist)) as u32;
                                let new_height = ((bg_rect_src.height() as f32) * (1.0-10.0*d_dist)) as u32;
                                bg_rect_src.set_width(new_width);
                                bg_rect_src.set_height(new_height);
                                bg_rect_src.center_on(Point::new(x,y));
                                bg_rect_dest = Rect::new(0,0,win_size.0,win_size.1); //initial_bg_rect.clone(); //maybe don't reset dest?
                            } else { //zoom out
                                bg_rect_src = Rect::new(0,0,win_size.0,win_size.1); //initial_bg_rect.clone(); //maybe don't reset src? 
                                                                                //remember d_dist is negative here
                                let new_width = ((bg_rect_dest.width() as f32) * (1.0+10.0*d_dist)) as u32;
                                let new_height = ((bg_rect_dest.height() as f32) * (1.0+10.0*d_dist)) as u32;
                                bg_rect_dest.set_width(new_width);
                                bg_rect_dest.set_height(new_height);
                                bg_rect_dest.center_on(Point::new(x,y));
                            }
                        }//num_fingers == 2
                        else {
                            println!("Multi-touch num_fingers: {}",num_fingers);
                        }
                    }, //Event::MultiGesture
                Event::MouseWheel {y, .. } => {
                        let mouse_state = pump.mouse_state();
                        let (mx,my) = (mouse_state.x(),mouse_state.y());     
                        //let (w1,h1) = canvas.viewport().size();
                        let complex_pos = view.screen_to_complex(mx, my, win_width, 
                                                                         win_height);
                        //println!("Zoom {} @ {:?}",if y>0 {"in"} else {"out"},(mx,my));
                        let zoomies = if y>0 {0.5} else {2.0};
                        view = view.zoom(complex_pos,zoomies);
                        update_bg(&mut bg_texture, &view, iterations);
                    },
                Event::Window {win_event: WindowEvent::SizeChanged(x,y), .. } => { 
                        println!("Got Size change -- x:{}, y:{}",x,y);
                        let new_size = canvas.viewport().size();
                        let nx = new_size.0;
                        let ny = new_size.1;
                        bg_rect_src = Rect::new(0, 0, nx, ny);
                        bg_rect_dest = Rect::new(0, 0, nx, ny);
                         let before = Instant::now();
                        //NEED NEW TEXTURE HERE, CAN'T JUST UPDATE!!
                        bg_texture = creator
                            .create_texture_streaming(PixelFormatEnum::ARGB8888, nx, ny)
                            .map_err(|e| e.to_string()).unwrap();
                        update_bg(&mut bg_texture, &view, iterations);
                         let after = before.elapsed();
                         println!("Resize time: {:?}",after);
                    },
                _ => { 
                        println!("unhandeled event: {:?}",event);
                     }
            } //match event
            potential_event = pump.poll_event();
        } //while events
        
        canvas.set_draw_color(white);
        canvas.clear(); 
        canvas.copy(&bg_texture, bg_rect_src, bg_rect_dest).unwrap();

        //draw orbits for current position
        {
            let (w1,h1) = canvas.viewport().size();
            let (w,h) = (w1.try_into().unwrap(),h1.try_into().unwrap());
            let mouse_state = pump.mouse_state();
            let (mx,my) = (mouse_state.x(),mouse_state.y());     
            let c = view.screen_to_complex(mx,my,w,h);
            let orbit_points = calc_orbits(c);
            let current_points = orbit_points.iter().map(|x| {view.complex_to_screen(*x,w,h)});
            draw_orbits(&mut canvas, &current_points.collect(), red, green)?;
            if saved_orbits.len() > 0 {
                let saved_points = saved_orbits.iter().map(|x| {view.complex_to_screen(*x,w,h)});
                draw_orbits(&mut canvas, &saved_points.collect(), magenta, cyan)?;
            }
        }

        if show_coords_q {
            let tmp = format!("{:.8} {:+.8}i",position.re,position.im);
            let coord_disp_surf = font.render(tmp.as_str()).
                    shaded(Color::RGBA(125, 0, 125, 255),Color::RGBA(200,200,200,255)).
                    map_err(|e| e.to_string())?;
            let coord_disp_texture = creator.create_texture_from_surface(&coord_disp_surf).
                    map_err(|e| e.to_string())?;
            let TextureQuery {width, height, .. } = coord_disp_texture.query();
            let text_rect = Rect::new(5, (canvas.viewport().size().1-height-5).
                                                try_into().unwrap(), 
                                width, height);
            canvas.copy(&coord_disp_texture, None, text_rect)?;
        }
        canvas.present();
    }; //mainloop

    println!("Exiting...");
    Ok(()) 
}

fn calc_orbits(c: Complex<f64>) -> Vec<Complex<f64>> {
    let iter = 50;
    let limit_sqr = 2.0 * 2.0;
    let mut z = Complex{re: 0.0, im: 0.0};
    let mut points = Vec::new();
   
    points.push(z); //origin
    points.push(c); //first point/mouse cursor position

    for _i in 0 .. iter {
        let z_next = z*z + c;
        if z_next.norm_sqr() > limit_sqr {
            break;
        }
        points.push(z_next);
        z = z_next;
    }

    points
}

fn draw_orbits(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, ps:& Vec<Point>, 
        c1: Color, c2: Color) -> Result<(), String> {
    
    let mut first_q = true;

    for (p1,p2) in ps.iter().tuple_windows() { 
        if first_q {
            first_q = false;
            canvas.set_draw_color(c1);
        } else {
            canvas.set_draw_color(c2);
        }
        canvas.draw_line(*p1,*p2)?;
    }

    Ok(())
}

fn update_bg(bg_texture: &mut sdl2::render::Texture, view:&ComplexBBox, iter:u32 ) -> () {
    
    let TextureQuery{format:_, access:_, width, height} = bg_texture.query();
    //println!("texture query: {:?} x {:?}, passed: {},{}",width,height,win_width,win_height);

    let w:usize = width.try_into().unwrap();
    let h:usize = height.try_into().unwrap();
            
    bg_texture.with_lock(None, |pixel_buffer: &mut [u8], pitch: usize| {
        //TODO: farm this out to multiple threads
        for y in 0 .. h {
            for x in 0 .. w {
                let c = view.screen_to_complex(x.try_into().unwrap(),y.try_into().unwrap(),
                                               w.try_into().unwrap(),h.try_into().unwrap());
                let mut z = Complex::<f64>{re: 0.0, im: 0.0};

                for _i in 0 .. iter {
                    z = z*z + c;
                    if z.norm_sqr() > 4.0 { break; }
                }

                let color = if z.norm_sqr() > 4.0 { 255 } else { 0 };
                let offset:usize = y * pitch + x * 4;
                pixel_buffer[offset+0] = color; //Blue
                pixel_buffer[offset+1] = color; //Green
                pixel_buffer[offset+2] = color; //Red
                pixel_buffer[offset+3] = 255; //Alpha 
            }//for x
        }//for y 
    }).unwrap();
    ()
}
