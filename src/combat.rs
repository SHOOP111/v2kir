use macroquad::{prelude::*, rand::gen_range};
use crate::{constants::*, math::*, model::*};
use crate::ai::{burst, floating};

fn weapon_stats(w: WeaponKind) -> (f32, f32, f32, i32, bool) { match w {
    WeaponKind::Dawnblade => (42.0, 0.12, 720.0, 2, false), WeaponKind::Repeater => (15.0, 0.085, 940.0, 0, false),
    WeaponKind::ArcCannon => (58.0, 0.55, 690.0, 1, true), WeaponKind::VoidLance => (115.0, 0.9, 1160.0, 5, false),
} }

pub fn player_aim(_game:&GameData)->Vec2{let m=mouse_position();safe_normalize(vec2(m.0-screen_width()*0.5,m.1-screen_height()*0.5))}

pub fn fire_weapon(game:&mut GameData,aim:Vec2){
    if game.player.fire_timer>0.0||game.state!=RunState::Playing{return;}
    let (damage,cooldown,speed,pierce,explosive)=weapon_stats(game.player.weapon);if game.player.energy<2.5{return;}
    let over=if game.player.overdrive>0.0{0.74}else{1.0};game.player.fire_timer=cooldown*over;game.player.energy=(game.player.energy-2.5).max(0.0);
    let shots=match game.player.weapon{WeaponKind::Dawnblade=>2,WeaponKind::VoidLance=>3,_=>1};
    for n in 0..shots{let spread=match game.player.weapon{WeaponKind::Dawnblade=>0.16,WeaponKind::VoidLance=>0.035,_=>0.012};let dir=rotate(aim,(n as f32-(shots-1)as f32*0.5)*spread);let p=game.player.pos+dir*25.0;let crit=gen_range(0.0_f32,100.0)<7.0+game.player.level as f32*0.7;let rift_bonus=if game.world.rift_event==Some(RiftEvent::Overcharge){1.22}else{1.0};let d=damage*(if crit{2.0}else{1.0})*(1.0+game.player.level as f32*0.045)*rift_bonus;game.projectiles.push(Projectile{pos:p,prev_pos:p,velocity:dir*speed,lifetime:1.65,radius:if explosive{9.0}else{5.0},damage:d,pierce,bounces:0,color:match game.player.weapon{WeaponKind::Dawnblade=>GOLD,WeaponKind::Repeater=>WHITE,WeaponKind::ArcCannon=>SKYBLUE,WeaponKind::VoidLance=>VIOLET},explosive,homing:if game.player.weapon==WeaponKind::VoidLance{0.8}else{0.0}});if crit{floating(game,p,format!("CRIT {}",d as i32),GOLD,15.0);game.shake=game.shake.max(0.18);}}
    burst(game,game.player.pos+aim*24.0,if explosive{SKYBLUE}else{WHITE},4,90.0,0.13);
}

pub fn update_projectiles(game:&mut GameData,dt:f32){
    for p in &mut game.projectiles{p.prev_pos=p.pos;if p.homing>0.0&&!game.enemies.is_empty(){let mut best=None;let mut best_d=1200.0*1200.0;for(i,e)in game.enemies.iter().enumerate(){let d=p.pos.distance_squared(e.pos);if d<best_d{best_d=d;best=Some(i);}}if let Some(i)=best{let target=safe_normalize(game.enemies[i].pos-p.pos);p.velocity=damp(p.velocity,target*p.velocity.length(),p.homing*8.0,dt);}}p.pos+=p.velocity*dt;p.lifetime-=dt;if p.pos.x.abs()>WORLD_W||p.pos.y.abs()>WORLD_H{p.lifetime=-1.0;}}
    let mut remove_proj=Vec::new();let mut hits=Vec::new();
    for(pi,p)in game.projectiles.iter().enumerate(){if p.lifetime<=0.0{remove_proj.push(pi);continue;}for(ei,e)in game.enemies.iter().enumerate(){if circle_segment_hit(e.pos,e.radius+p.radius,p.prev_pos,p.pos){hits.push((pi,ei,p.damage,p.explosive));break;}}}
    for(pi,ei,dmg,explosive)in hits{
        if let Some(p)=game.projectiles.get_mut(pi){if p.pierce>0{p.pierce-=1;}else{p.lifetime=-1.0;}}
        let hit_pos=game.enemies.get(ei).map(|e|e.pos);let hit_kind=game.enemies.get(ei).map(|e|e.kind);let mut alive=false;
        if let Some(e)=game.enemies.get_mut(ei){e.hp-=dmg;e.flash=0.18;alive=e.hp>0.0;game.player.damage_done+=dmg as f64;game.player.fury=(game.player.fury+dmg*0.12).min(100.0);}
        if let Some(pos)=hit_pos{floating(game,pos+vec2(0.0,-32.0),format!("{}",dmg as i32),if dmg>70.0{GOLD}else{WHITE},14.0);if explosive{burst(game,pos,SKYBLUE,18,230.0,0.32);for other in &mut game.enemies{let dist=other.pos.distance(pos);if dist<150.0&&other.pos!=pos{other.hp-=dmg*(1.0-dist/150.0)*0.35;other.stun=0.12;}}}else{burst(game,pos,crate::ai::enemy_color(hit_kind.unwrap_or(EnemyKind::Grunt)),5,80.0,0.18);}}
        let _=alive;
    }
    remove_proj.extend(game.projectiles.iter().enumerate().filter_map(|(i,p)|if p.lifetime<=0.0{Some(i)}else{None}));remove_proj.sort_unstable();remove_proj.dedup();for i in remove_proj.into_iter().rev(){if i<game.projectiles.len(){game.projectiles.swap_remove(i);}}
    let mut dead:Vec<usize>=game.enemies.iter().enumerate().filter_map(|(i,e)|if e.hp<=0.0{Some(i)}else{None}).collect();dead.sort_unstable();dead.dedup();for i in dead.into_iter().rev(){let e=game.enemies.swap_remove(i);kill_enemy(game,e);}
}

fn enemy_tint(k:EnemyKind)->Color{match k{EnemyKind::Grunt=>RED,EnemyKind::Shooter=>VIOLET,EnemyKind::Dasher=>ORANGE,EnemyKind::Brute=>MAROON,EnemyKind::Warden=>SKYBLUE,EnemyKind::Harvester=>YELLOW}}
fn kill_enemy(game:&mut GameData,e:Enemy){game.player.kills+=1;game.player.combo=game.player.combo.saturating_add(1);game.player.combo_timer=2.2;game.world.kills_for_wave+=1;game.save.lifetime_kills+=1;let xp=(14.0+e.max_hp*0.17)*(if e.elite{2.8}else{1.0})*(1.0+game.world.wave as f32*0.016);grant_xp(game,xp as u32);game.score=game.score.saturating_add((e.max_hp as u64)*if e.elite{7}else{1});burst(game,e.pos,enemy_tint(e.kind),if e.elite{38}else{18},if e.elite{340.0}else{210.0},if e.elite{0.55}else{0.34});let fortune_bonus=if game.world.rift_event==Some(RiftEvent::FortuneFlux){22.0}else{0.0};if e.elite||gen_range(0.0_f32,100.0)<8.0+game.world.anomaly*12.0+fortune_bonus{let kind=if game.player.hp<55.0&&gen_range(0,2)==0{PickupKind::Heal}else if gen_range(0,3)==0{PickupKind::Fury}else{PickupKind::Energy};game.pickups.push(Pickup{pos:e.pos,velocity:Vec2::ZERO,kind,radius:12.0,ttl:18.0,spin:0.0});}if gen_range(0.0_f32,100.0)<4.5{game.pickups.push(Pickup{pos:e.pos+vec2(gen_range(-8.0,8.0),gen_range(-8.0,8.0)),velocity:Vec2::ZERO,kind:PickupKind::Xp,radius:9.0,ttl:25.0,spin:0.0});}}
fn grant_xp(game:&mut GameData,xp:u32){game.player.xp=game.player.xp.saturating_add(xp);game.save.lifetime_xp+=xp as u64;loop{let req=xp_to_next(game.player.level);if game.player.xp<req{break;}game.player.xp-=req;game.player.level+=1;game.player.max_hp+=8.0;game.player.max_energy+=4.0;game.player.hp=game.player.max_hp;game.player.energy=game.player.max_energy;game.banner=format!("LEVEL {} // POWER ASCENDED",game.player.level);game.banner_timer=1.8;game.shake=0.45;}}
pub fn update_pickups(game:&mut GameData,dt:f32){for p in &mut game.pickups{p.ttl-=dt;p.spin+=dt*3.2;if game.player.pos.distance(p.pos)<170.0{p.velocity=damp(p.velocity,safe_normalize(game.player.pos-p.pos)*260.0,7.0,dt);}p.pos+=p.velocity*dt;}let mut remove=Vec::new();for(i,p)in game.pickups.iter().enumerate(){if p.ttl<=0.0{remove.push(i);continue;}if p.pos.distance(game.player.pos)<p.radius+game.player.radius{match p.kind{PickupKind::Heal=>game.player.hp=(game.player.hp+28.0).min(game.player.max_hp),PickupKind::Energy=>game.player.energy=(game.player.energy+35.0).min(game.player.max_energy),PickupKind::Fury=>game.player.fury=(game.player.fury+30.0).min(100.0),PickupKind::Xp=>grant_xp(game,44)}burst(game,p.pos,match p.kind{PickupKind::Heal=>GREEN,PickupKind::Energy=>SKYBLUE,PickupKind::Fury=>ORANGE,PickupKind::Xp=>GOLD},12,130.0,0.22);remove.push(i);}}for i in remove.into_iter().rev(){if i<game.pickups.len(){game.pickups.swap_remove(i);}}}
