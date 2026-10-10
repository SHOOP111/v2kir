use macroquad::prelude::*;
use crate::{constants::*, math::*, model::*};
use crate::{combat, world, save};

impl GameData {
    pub fn new() -> Self {
        let mut g = Self { state: RunState::Title, player: Player::default(), enemies:Vec::new(), projectiles:Vec::new(), pickups:Vec::new(), particles:Vec::new(), texts:Vec::new(), world:WorldState::new(0xA37E_F411), save:save::load(), score:0, accumulator:0.0, elapsed:0.0, shake:0.0, flash:0.0, near_miss:0.0, banner:"".into(), banner_timer:0.0 };
        g.save.version=SAVE_VERSION;
        g
    }

    pub fn update(&mut self, dt:f32){
        if is_key_pressed(KeyCode::Escape){self.state=match self.state{RunState::Playing=>RunState::Paused,RunState::Paused=>RunState::Playing,s=>s};}
        match self.state {
            RunState::Title => { if is_key_pressed(KeyCode::Enter)||is_mouse_button_pressed(MouseButton::Left){world::reset_run(self);} }
            RunState::Paused => {}
            RunState::Dead => { if is_key_pressed(KeyCode::R){world::reset_run(self);} }
            RunState::Playing => self.simulate(dt),
        }
    }

    fn simulate(&mut self,dt:f32){
        self.player.fire_timer=(self.player.fire_timer-dt).max(0.0);self.player.dash_timer=(self.player.dash_timer-dt).max(0.0);self.player.dash_time=(self.player.dash_time-dt).max(0.0);self.player.invulnerable=(self.player.invulnerable-dt).max(0.0);self.player.nova_timer=(self.player.nova_timer-dt).max(0.0);self.player.overdrive=(self.player.overdrive-dt).max(0.0);self.player.combo_timer=(self.player.combo_timer-dt).max(0.0);if self.player.combo_timer<=0.0{self.player.combo=0;}
        let mut input=Vec2::new(if is_key_down(KeyCode::D)||is_key_down(KeyCode::Right){1.0}else{0.0}-if is_key_down(KeyCode::A)||is_key_down(KeyCode::Left){1.0}else{0.0},if is_key_down(KeyCode::S)||is_key_down(KeyCode::Down){1.0}else{0.0}-if is_key_down(KeyCode::W)||is_key_down(KeyCode::Up){1.0}else{0.0});
        input=safe_normalize(input);
        if is_key_pressed(KeyCode::Space)&&self.player.dash_timer<=0.0{let dir=if input.length_squared()>0.0{input}else{combat::player_aim(self)};self.player.velocity=dir*DASH_SPEED;self.player.dash_time=0.14;self.player.dash_timer=0.95;self.player.invulnerable=0.22;crate::ai::burst(self,self.player.pos,SKYBLUE,24,300.0,0.28);self.shake=self.shake.max(0.35);}
        if is_key_pressed(KeyCode::Q)&&self.player.nova_timer<=0.0&&self.player.energy>=45.0{self.player.energy-=45.0;self.player.nova_timer=5.0;for e in self.enemies.iter_mut(){let d=e.pos.distance(self.player.pos);if d<360.0{e.hp-=125.0*(1.0-d/360.0);e.stun=0.6;}}crate::ai::burst(self,self.player.pos,VIOLET,70,420.0,0.65);self.shake=self.shake.max(0.8);}
        if is_key_pressed(KeyCode::E)&&self.player.fury>=100.0{self.player.fury=0.0;self.player.overdrive=8.0;self.banner="OVERDRIVE // TIME TO BREAK THE HORDE".into();self.banner_timer=1.8;crate::ai::burst(self,self.player.pos,GOLD,35,260.0,0.45);}
        if is_mouse_button_down(MouseButton::Left){combat::fire_weapon(self,combat::player_aim(self));}
        let max_speed=if self.player.dash_time>0.0{DASH_SPEED}else{PLAYER_SPEED*(if self.player.overdrive>0.0{1.18}else{1.0})};let desired=input*max_speed;if self.player.dash_time<=0.0{self.player.velocity=damp(self.player.velocity,desired,16.0,dt);}self.player.pos+=self.player.velocity*dt;
        if self.player.velocity.length_squared()>30.0{self.player.combo_timer=self.player.combo_timer.max(0.0);}
        self.player.energy=(self.player.energy+dt*11.0).min(self.player.max_energy);world::resolve_world_bounds(self);
        world::update_world(self,dt);crate::ai::update_enemies(self,dt);combat::update_projectiles(self,dt);combat::update_pickups(self,dt);update_fx(self,dt);
        if self.player.nova_timer<=0.0 && self.player.fury>90.0{self.banner="FURY PRIMED // [E] OVERDRIVE".into();self.banner_timer=self.banner_timer.max(0.1);}
        if self.elapsed>1.0 && self.elapsed%2.5<dt {save::store(&self.save);}
    }
}

fn update_fx(game:&mut GameData,dt:f32){
    for p in game.particles.iter_mut(){p.life-=dt;p.velocity*=1.0/(1.0+p.drag*dt);p.velocity.y+=p.gravity*dt;p.pos+=p.velocity*dt;}
    game.particles.retain(|p|p.life>0.0);if game.particles.len()>MAX_PARTICLES{let extra=game.particles.len()-MAX_PARTICLES;game.particles.drain(0..extra);}
    for t in game.texts.iter_mut(){t.life-=dt;t.pos.y-=dt*28.0;}game.texts.retain(|t|t.life>0.0);
}
