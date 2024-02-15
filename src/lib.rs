use std::collections::VecDeque;

use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::{go, robot_map},
    runner::{backpack::BackPack, Robot, Runnable},
    world::{coordinates::Coordinate, tile::Content, World},
};

use sense_and_find_by_Rustafariani::Lssf;
use spyglass::spyglass::{Spyglass, SpyglassResult};
use ui_lib::RunnableUi;
use utils::is_content_rock;

pub mod utils;

enum State {
    Ready,
    Discover,
    Find,
    Collect,
    Build,
}

impl Default for State {
    fn default() -> Self {
        State::Ready
    }
}

pub struct BuilderAi {
    ui: Box<dyn RunnableUi>,
    world_size: usize,
    robot: Robot,
    state: State,
    row: usize,
    col: usize,
    rocks: VecDeque<(usize, usize)>,
}

impl BuilderAi {
    pub fn new(ui: Box<dyn RunnableUi>, world_size: usize) -> Self {
        Self {
            ui,
            world_size,
            robot: Robot::new(),
            state: State::Ready,
            row: 0,
            col: 0,
            rocks: VecDeque::new(),
        }
    }

    pub fn run(&mut self, world: &mut World) {
        self.row = self.get_coordinate().get_row();
        self.col = self.get_coordinate().get_col();

        match self.state {
            State::Ready => {
                self.do_ready();
            }
            State::Discover => {
                self.do_discover(world);
            }
            State::Find => {
                self.do_find(world);
            }
            State::Collect => {
                self.do_collect(world);
            }
            State::Build => {
                self.do_build(world);
            }
        }
    }

    fn do_ready(&mut self) {
        self.state = State::Discover;
    }

    fn do_discover(&mut self, world: &mut World) {
        let mut spyglass = Spyglass::new(
            self.row,
            self.col,
            self.world_size,
            self.world_size,
            None,
            true,
            1.0,
            |t| is_content_rock(&t.content),
        );

        let result = spyglass.new_discover(self, world);

        match result {
            SpyglassResult::Complete => self.state = State::Collect,
            SpyglassResult::Failed(error) => {
                println!("{:?}", error);
            }
            _ => {}
        }
    }

    fn do_find(&mut self, world: &mut World) {
        let map = robot_map(world).unwrap();

        let mut lssf = Lssf::new();

        lssf.update_map(&map);
        let _ = lssf.update_cost(self.row, self.col);

        self.rocks.extend(lssf.get_content_vec(&Content::Rock(0)));

        if !self.rocks.is_empty() {
            self.state = State::Collect;
        }
    }

    fn do_collect(&mut self, world: &mut World) {}

    fn do_build(&mut self, world: &mut World) {}
}

impl Runnable for BuilderAi {
    fn process_tick(&mut self, world: &mut World) {
        self.run(world);
        self.ui.process_tick(world);
    }

    fn handle_event(&mut self, event: Event) {
        self.ui.handle_event(event);
    }

    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }

    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }

    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }

    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }

    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }

    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
}
