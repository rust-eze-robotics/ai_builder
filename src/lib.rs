use std::{collections::VecDeque, f32::consts::E};

use pmp_street_picasso::{ToolError, ToolStreetPicasso};
use robotics_lib::{
    energy::Energy,
    event::events::Event,
    interface::{go, robot_map, robot_view, teleport, Direction},
    runner::{backpack::BackPack, Robot, Runnable},
    world::{
        coordinates::Coordinate,
        tile::{Content, TileType},
        World,
    },
};

use sense_and_find_by_Rustafariani::{Action, Lssf};
use spyglass::spyglass::{Spyglass, SpyglassResult};
use ui_lib::RunnableUi;
use utils::is_content_rock;
use OhCrab_collection::collection::{self, CollectTool};

pub mod utils;

#[derive(Debug)]
enum State {
    Ready,
    Discover,
    Locate,
    Find,
    Collect,
    Build,
    Terminate,
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
    actions: VecDeque<Action>,
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
            actions: VecDeque::new(),
        }
    }

    pub fn run(&mut self, world: &mut World) {
        robot_view(self, world);

        self.row = self.get_coordinate().get_row();
        self.col = self.get_coordinate().get_col();

        match self.state {
            State::Ready => {
                self.do_ready();
            }
            State::Discover => {
                self.do_discover(world);
            }
            State::Locate => {
                self.do_locate(world);
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
            State::Terminate => {
                self.do_terminate(world);
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
            10,
            self.world_size,
            None,
            true,
            1.0,
            |tile| is_content_rock(&tile.content),
        );

        let result = spyglass.new_discover(self, world);

        match result {
            SpyglassResult::Failed(error) => {
                println!("{:?}", error);
            }
            _ => {
                self.state = State::Locate;
            }
        }
    }

    fn do_locate(&mut self, world: &World) {
        let map = robot_map(world).unwrap();

        let mut lssf = Lssf::new();
        lssf.update_map(&map);
        let _ = lssf.update_cost(self.row, self.col);

        let vec = lssf.get_content_vec(&Content::Rock(0));
        self.rocks = VecDeque::new();

        for (row, col) in vec {
            if map[row][col].as_ref().unwrap().tile_type != TileType::Street {
                self.rocks.push_back((row, col));
            }
        }

        if self.rocks.is_empty() {
            self.state = State::Discover;
        } else {
            self.state = State::Find;
        }
    }

    fn do_find(&mut self, world: &mut World) {
        if self.actions.is_empty() {
            let map = robot_map(world).unwrap();

            let mut lssf = Lssf::new();
            lssf.update_map(&map);
            let _ = lssf.update_cost(self.row, self.col);

            if let Some((row, col)) = self.rocks.pop_front() {
                self.actions.extend(lssf.get_action_vec(row, col).unwrap());

                if self.actions.is_empty() {
                    let _ = go(self, world, Direction::Left);
                    self.state = State::Collect;
                }
            }
        }

        if self.actions.len() > 1 {
            if let Some(action) = self.actions.pop_front() {
                match action {
                    Action::East => {
                        let _ = go(self, world, Direction::Right);
                    }
                    Action::South => {
                        let _ = go(self, world, Direction::Down);
                    }
                    Action::West => {
                        let _ = go(self, world, Direction::Left);
                    }
                    Action::North => {
                        let _ = go(self, world, Direction::Up);
                    }
                    Action::Teleport(col, row) => {
                        let _ = teleport(self, world, (row, col));
                    }
                }
            }
        }

        if self.actions.len() == 1 {
            self.actions = VecDeque::new();
            self.state = State::Collect;
        }
    }

    fn do_collect(&mut self, world: &mut World) {
        let result = CollectTool::collect_instantly_reachable(self, world, &Content::Rock(0));

        if result.is_ok() {
            self.state = State::Build;
        }
    }

    fn do_build(&mut self, world: &mut World) {
        robot_view(self, world);
        let result = ToolStreetPicasso::create_street(self, world, 1, Direction::Right, 1);

        match result {
            Ok(()) => {
                self.state = State::Discover;
            }
            Err(error) => {
                self.state = State::Discover;
            }
        }
    }

    fn do_terminate(&mut self, _world: &World) {}
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
