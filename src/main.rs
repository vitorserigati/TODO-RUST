use ncurses::*;
use std::cmp::{self, min};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::ops::{Add, Mul};
use std::{env, process};

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

enum LayoutKind {
    Vertical,
    Horizontal,
}

#[derive(Default, Clone, Copy, Debug)]
struct Vec2d {
    x: i32,
    y: i32,
}

impl Add for Vec2d {
    type Output = Vec2d;

    fn add(self, rhs: Vec2d) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Mul for Vec2d {
    type Output = Vec2d;

    fn mul(self, rhs: Vec2d) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Vec2d {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
struct Layout {
    kind: LayoutKind,
    pos: Vec2d,
    size: Vec2d,
}

impl Layout {
    fn new(kind: LayoutKind, pos: Vec2d) -> Self {
        Self {
            kind,
            pos,
            size: Vec2d::new(0, 0),
        }
    }

    fn available_pos(&self) -> Vec2d {
        use LayoutKind::*;

        match self.kind {
            Horizontal => self.pos + self.size * Vec2d::new(1, 0),
            Vertical => self.pos + self.size * Vec2d::new(0, 1),
        }
    }

    fn add_widget(&mut self, size: Vec2d) {
        use LayoutKind::*;

        match self.kind {
            Vertical => {
                self.size.x = cmp::max(self.size.x, size.x);
                self.size.y += size.y;
            }
            Horizontal => {
                self.size.x += size.x;
                self.size.y = cmp::max(self.size.y, size.y);
            }
        }
    }
}

#[derive(Default)]
struct Ui {
    layouts: Vec<Layout>,
}

impl Ui {
    fn begin(&mut self, pos: Vec2d, kind: LayoutKind) {
        assert!(self.layouts.is_empty());

        self.layouts.push(Layout {
            kind,
            pos,
            size: Vec2d::new(0, 0),
        })
    }

    fn begin_layout(&mut self, kind: LayoutKind) {
        let layout = self
            .layouts
            .last()
            .expect("Can't create a layout outside of Ui::begin() and Ui::end()");

        let pos = layout.available_pos();
        self.layouts.push(Layout::new(kind, pos))
    }

    fn end_layout(&mut self) {
        let layout = self
            .layouts
            .pop()
            .expect("Unbalanced UI::begin_layout() and UI::end_layout() calls.");

        self.layouts
            .last_mut()
            .expect("Unbalanced UI::begin_layout() and UI::end_layout() calls.")
            .add_widget(layout.size);
    }

    fn label_fixed_width(&mut self, label: &str, pair: i16, size: i32) {
        let layout = self
            .layouts
            .last_mut()
            .expect("Trying to render label outside of any layout");

        let pos = layout.available_pos();

        mv(pos.y, pos.x);
        attron(COLOR_PAIR(pair));
        addstr(label);
        attroff(COLOR_PAIR(pair));

        layout.add_widget(Vec2d::new(size, 1));
    }

    #[allow(dead_code)]
    fn label(&mut self, label: &str, pair: i16) {
        self.label_fixed_width(label, pair, label.len() as i32)
    }

    fn end(&mut self) {
        self.layouts
            .pop()
            .expect("Unbalanced UI::begin_layout() and UI::end_layout() calls.");
    }
}

#[derive(Debug, PartialEq)]
enum Status {
    Todo,
    Done,
}
impl Status {
    fn toggle(&self) -> Self {
        match self {
            Status::Todo => Status::Done,
            Status::Done => Status::Todo,
        }
    }
}

fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_item = line
        .strip_prefix("TODO: ")
        .map(|title| (Status::Todo, title));
    let done_item = line
        .strip_prefix("DONE: ")
        .map(|title| (Status::Done, title));

    todo_item.or(done_item)
}

fn drag_up(list: &mut [String], pos: &mut usize) {
    if !list.is_empty() && *pos > 0 {
        list.swap(*pos - 1, *pos);
        *pos = *pos - 1;
    }
}

fn drag_down(list: &mut [String], pos: &mut usize) {
    if !list.is_empty() && *pos < list.len() - 1 {
        list.swap(*pos + 1, *pos);
        *pos = *pos + 1;
    }
}

fn list_up(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr -= 1
    }
}

fn list_down(list: &Vec<String>, list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        *list_curr = min(*list_curr + 1, list.len() - 1)
    }
}

fn list_transfer(
    list_dst: &mut Vec<String>,
    list_src: &mut Vec<String>,
    list_src_curr: &mut usize,
) {
    if *list_src_curr < list_src.len() {
        list_dst.push(list_src.remove(*list_src_curr));
        if *list_src_curr >= list_src.len() && !list_src.is_empty() {
            *list_src_curr = list_src.len() - 1;
        }
    }
}

fn delete_from_list(list: &mut Vec<String>, pos: &usize) {
    if !list.is_empty() {
        list.remove(*pos);
    }
}

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) {
    let file = File::open(file_path).unwrap();
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line.unwrap()) {
            Some((Status::Todo, line)) => todos.push(line.to_string()),
            Some((Status::Done, line)) => dones.push(line.to_string()),
            None => {
                eprintln!("{}: {}: Item mal formatado", &file_path, &index + 1);
                process::exit(1);
            }
        }
    }
}

fn save_state(todos: &[String], dones: &[String], file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        writeln!(file, "TODO: {}", todo).unwrap();
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done).unwrap();
    }
}

// TODO: add new elements to TODO
// TODO: delete items
// TODO: Edit the elements
// TODO: keep track of date when the item was DONE
// TODO: undo system
// TODO: Handle sigint

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let file_path = {
        match args.next() {
            Some(file_path) => file_path,
            None => {
                eprintln!("Usage: todo <file-path>");
                eprintln!("Error: File path is not provided");
                process::exit(1);
            }
        }
    };
    let mut todos: Vec<String> = Vec::<String>::new();
    let mut dones: Vec<String> = Vec::<String>::new();
    let mut dones_curr: usize = 0;
    let mut todo_curr: usize = 0;
    let mut panel = Status::Todo;

    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    refresh();

    load_state(&mut todos, &mut dones, &file_path);

    let mut ui = Ui::default();

    loop {
        erase();

        let mut x = 0;
        let mut y = 0;

        getmaxyx(stdscr(), &mut x, &mut y);

        ui.begin(Vec2d::new(0, 0), LayoutKind::Horizontal);
        {
            ui.begin_layout(LayoutKind::Vertical);
            {
                match panel {
                    Status::Todo => ui.label_fixed_width("[TODO]:", REGULAR_PAIR, x / 2),
                    Status::Done => ui.label_fixed_width(" TODO :", REGULAR_PAIR, x / 2),
                }

                for (index, todo) in todos.iter().enumerate() {
                    ui.label_fixed_width(
                        &format!("- [ ] {}", todo),
                        if index == todo_curr && panel == Status::Todo {
                            HIGHLIGHT_PAIR
                        } else {
                            REGULAR_PAIR
                        },
                        x / 2,
                    );
                }
            }
            ui.end_layout();

            ui.begin_layout(LayoutKind::Vertical);
            {
                match panel {
                    Status::Todo => ui.label_fixed_width(" DONE :", REGULAR_PAIR, x / 2),
                    Status::Done => ui.label_fixed_width("[DONE]:", REGULAR_PAIR, x / 2),
                }

                for (index, done) in dones.iter().enumerate() {
                    ui.label_fixed_width(
                        &format!("- [X] {}", done),
                        if index == dones_curr && panel == Status::Done {
                            HIGHLIGHT_PAIR
                        } else {
                            REGULAR_PAIR
                        },
                        x / 2,
                    );
                }
            }
            ui.end_layout();
        }
        ui.end();

        refresh();

        let key = getch();
        match key as u8 as char {
            'q' => break,
            'W' => match panel {
                Status::Todo => drag_up(&mut todos, &mut todo_curr),
                Status::Done => drag_up(&mut dones, &mut dones_curr),
            },
            'w' => match panel {
                Status::Todo => list_up(&mut todo_curr),
                Status::Done => list_up(&mut dones_curr),
            },
            's' => match panel {
                Status::Todo => list_down(&todos, &mut todo_curr),
                Status::Done => list_down(&dones, &mut dones_curr),
            },
            'S' => match panel {
                Status::Todo => drag_down(&mut todos, &mut todo_curr),
                Status::Done => drag_down(&mut dones, &mut dones_curr),
            },
            '\n' => match panel {
                Status::Todo => list_transfer(&mut dones, &mut todos, &mut todo_curr),
                Status::Done => list_transfer(&mut todos, &mut dones, &mut dones_curr),
            },
            'd' => match panel {
                Status::Todo => delete_from_list(&mut todos, &todo_curr),
                Status::Done => delete_from_list(&mut dones, &dones_curr),
            },
            '\t' => panel = panel.toggle(),
            'i' => {}
            _ => {}
        }
    }

    save_state(&todos, &dones, &file_path);
    endwin();
}
