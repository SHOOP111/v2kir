use macroquad::{prelude::*, rand};
use crate::{constants::*, math::*, model::*};
use crate::ai::enemy_color;

pub struct View { pub center: Vec2, pub zoom: f32, pub shake: Vec2 }
impl View { pub fn world_to_screen(&self,p:Vec2)->Vec2{(p-self.center)*self.zoom+vec2(screen_width()*0.5,screen_height()*0.5)+self.shake} pub fn scale(&self,v:f32)->f32{v*self.zoom} }
fn fade(c:Color,a:f32)->Color{Color::new(c.r,c.g,c.b,(c.a*a).clamp(0.0,1.0))}

pub fn draw(game:&GameData){
    clear_background(Color::new(0.014,0.018,0.028,1.0));
    let zoom=(1.0/(1.0+(game.enemies.len() as f32/380.0))).clamp(0.72,1.0);
    let sh=if game.shake>0.0{vec2(rand::gen_range(-1.0_f32,1.0),rand::gen_range(-1.0_f32,1.0))*game.shake*18.0}else{Vec2::ZERO};
    let v=View{center:game.player.pos,zoom,shake:sh};
    draw_arena(game,&v);draw_pickups(game,&v);draw_projectiles(game,&v);draw_enemies(game,&v);draw_player(game,&v);draw_particles(game,&v);draw_world_effects(game,&v);
}
fn draw_arena(game:&GameData,v:&View){
    let center=v.world_to_screen(Vec2::ZERO);let radius=v.scale(game.world.arena_radius);
    draw_circle(center.x,center.y,radius,Color::new(0.025,0.034,0.055,1.0));
    let cell=ARENA_CELL;let lim=(game.world.arena_radius as i32+cell as i32) as i32;
    for x in (-lim..=lim).step_by(cell as usize){let a=v.world_to_screen(vec2(x as f32,-game.world.arena_radius));let b=v.world_to_screen(vec2(x as f32,game.world.arena_radius));draw_line(a.x,a.y,b.x,b.y,1.0,fade(Color::new(0.10,0.16,0.26,1.0),0.35));}
    for y in (-lim..=lim).step_by(cell as usize){let a=v.world_to_screen(vec2(-game.world.arena_radius,y as f32));let b=v.world_to_screen(vec2(game.world.arena_radius,y as f32));draw_line(a.x,a.y,b.x,b.y,1.0,fade(Color::new(0.10,0.16,0.26,1.0),0.35));}
    draw_circle_lines(center.x,center.y,radius,3.0,fade(SKYBLUE,0.42+game.world.anomaly*0.25));
    for i in 0..7{let rr=radius*(0.16+i as f32*0.12);draw_circle_lines(center.x,center.y,rr,1.0,fade(VIOLET,0.10));}
}
fn draw_player(game:&GameData,v:&View){let p=v.world_to_screen(game.player.pos);let r=v.scale(game.player.radius);let c=if game.player.invulnerable>0.0&&((game.elapsed*40.0)as i32)%2==0{fade(WHITE,0.25)}else{WHITE};draw_circle(p.x,p.y,r*1.7,fade(SKYBLUE,0.08));draw_circle(p.x,p.y,r,c);let aim=crate::combat::player_aim(game);let tip=p+aim*r*1.7;draw_line(p.x,p.y,tip.x,tip.y,v.scale(5.0),if game.player.overdrive>0.0{GOLD}else{SKYBLUE});draw_circle(p.x,p.y,r*0.56,Color::new(0.05,0.12,0.20,1.0));draw_circle_lines(p.x,p.y,r*1.35,2.0,fade(SKYBLUE,0.55));if game.player.dash_time>0.0{draw_circle_lines(p.x,p.y,r*2.1,2.0,fade(WHITE,0.7));}}
fn draw_enemies(game:&GameData,v:&View){for e in &game.enemies{let p=v.world_to_screen(e.pos);let r=v.scale(e.radius);let c=enemy_color(e.kind);draw_circle(p.x,p.y,r*1.45,fade(c,0.08));match e.kind{EnemyKind::Warden=>{draw_poly(p.x,p.y,6,r,0.0,c);draw_circle_lines(p.x,p.y,r*1.4,2.0,fade(c,0.7));},EnemyKind::Brute=>{draw_rectangle(p.x-r,p.y-r,r*2.0,r*2.0,c);draw_rectangle_lines(p.x-r,p.y-r,r*2.0,r*2.0,2.0,fade(WHITE,0.45));},_=>{draw_circle(p.x,p.y,r,c);draw_circle_lines(p.x,p.y,r,1.5,fade(WHITE,0.35));}}let hp=(e.hp/e.max_hp).clamp(0.0,1.0);draw_rectangle(p.x-r,p.y-r-9.0,r*2.0,3.0,fade(BLACK,0.8));draw_rectangle(p.x-r,p.y-r-9.0,r*2.0*hp,3.0,if e.elite{GOLD}else{c});}}
fn draw_projectiles(game:&GameData,v:&View){for p in &game.projectiles{let a=v.world_to_screen(p.prev_pos);let b=v.world_to_screen(p.pos);draw_line(a.x,a.y,b.x,b.y,v.scale(p.radius*1.35),fade(p.color,0.28));draw_circle(b.x,b.y,v.scale(p.radius),p.color);}}
fn draw_pickups(game:&GameData,v:&View){for p in &game.pickups{let s=v.world_to_screen(p.pos);let r=v.scale(p.radius)*(1.0+0.12*(p.spin*3.0).sin());let c=match p.kind{PickupKind::Heal=>GREEN,PickupKind::Energy=>SKYBLUE,PickupKind::Fury=>ORANGE,PickupKind::Xp=>GOLD};draw_circle(s.x,s.y,r*1.8,fade(c,0.08));draw_circle(s.x,s.y,r,c);draw_circle_lines(s.x,s.y,r*1.45,1.5,fade(WHITE,0.45));}}
fn draw_particles(game:&GameData,v:&View){for p in &game.particles{let s=v.world_to_screen(p.pos);let a=(p.life/p.max_life).clamp(0.0,1.0);draw_circle(s.x,s.y,v.scale(p.size*(0.55+0.45*a)),fade(p.color,a));}}
fn draw_world_effects(game:&GameData,v:&View){let t=game.world.time;for i in 0..12{let a=t*0.12+i as f32*0.53;let p=game.player.pos+from_angle(a)*(game.world.arena_radius*0.75+((t+i as f32*2.1).sin()*120.0));let s=v.world_to_screen(p);let d=from_angle(a);let len=v.scale(45.0);draw_line(s.x,s.y,s.x+len*d.x,s.y+len*d.y,1.0,fade(VIOLET,0.08+game.world.anomaly*0.06));}}

pub fn draw_overlay(game:&GameData){let w=screen_width();let h=screen_height();if game.flash>0.0{draw_rectangle(0.0,0.0,w,h,fade(RED,game.flash*0.35));}if let Some(event)=game.world.rift_event{let tint=match event{RiftEvent::BloodMoon=>RED,RiftEvent::Overcharge=>SKYBLUE,RiftEvent::TimeSnare=>VIOLET,RiftEvent::FortuneFlux=>GOLD};draw_rectangle(0.0,0.0,w,h,Color::new(tint.r,tint.g,tint.b,0.035));}for t in &game.texts{let s=vec2(w*0.5+(t.pos.x-game.player.pos.x),h*0.5+(t.pos.y-game.player.pos.y));let a=(t.life/t.max_life).clamp(0.0,1.0);draw_text(&t.text,s.x,s.y,t.size,fade(t.color,a));}if game.banner_timer>0.0{let alpha=(game.banner_timer.min(0.5)*2.0).min(1.0);let m=measure_text(&game.banner,None,28,1.0);draw_text(&game.banner,w*0.5-m.width*0.5,84.0,28.0,fade(WHITE,alpha));}match game.state{RunState::Title=>title_screen(),RunState::Paused=>pause_screen(),RunState::Dead=>dead_screen(),RunState::Playing=>{}}}
fn title_screen(){let w=screen_width();let h=screen_height();draw_rectangle(0.0,0.0,w,h,Color::new(0.01,0.012,0.02,0.90));let title="AETHERFALL";let sub="THE SHATTERED CROWN";let m=measure_text(title,None,72,1.0);draw_text(title,w*0.5-m.width*0.5,h*0.34,72.0,SKYBLUE);let s=measure_text(sub,None,24,1.0);draw_text(sub,w*0.5-s.width*0.5,h*0.34+42.0,24.0,VIOLET);let hint="PRESS ENTER OR CLICK TO DESCEND";let hm=measure_text(hint,None,18,1.0);draw_text(hint,w*0.5-hm.width*0.5,h*0.70,18.0,WHITE);}
fn pause_screen(){let w=screen_width();let h=screen_height();draw_rectangle(0.0,0.0,w,h,Color::new(0.0,0.0,0.02,0.64));let m=measure_text("PAUSED",None,54,1.0);draw_text("PAUSED",w*0.5-m.width*0.5,h*0.43,54.0,WHITE);}
fn dead_screen(){let w=screen_width();let h=screen_height();draw_rectangle(0.0,0.0,w,h,Color::new(0.05,0.0,0.01,0.68));let m=measure_text("RUN ENDED",None,54,1.0);draw_text("RUN ENDED",w*0.5-m.width*0.5,h*0.40,54.0,RED);let s=measure_text("PRESS R TO RESTART",None,20,1.0);draw_text("PRESS R TO RESTART",w*0.5-s.width*0.5,h*0.54,20.0,WHITE);}
