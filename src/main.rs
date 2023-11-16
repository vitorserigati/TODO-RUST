use ncurses::*;
use std::cmp::min;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::{env, process};

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

type Id = usize;

#[derive(Default)]
struct Ui {
    list_curr: Option<Id>,
    row: usize,
    col: usize,
}

impl Ui {
    fn begin(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
    fn begin_list(&mut self, id: Id) {
        assert!(self.list_curr.is_none(), "Nested Lists are not allowed!");
        self.list_curr = Some(id);
    }

    fn label(&mut self, label: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        addstr(label);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }

    fn list_element(&mut self, label: &str, id: Id) -> bool {
        let id_curr = self
            .list_curr
            .expect("Not allowed to create list elements outside of lists");

        self.label(label, {
            if id_curr == id {
                HIGHLIGHT_PAIR
            } else {
                REGULAR_PAIR
            }
        });

        return false;
    }

    fn end_list(&mut self) {
        self.list_curr = None;
    }

    fn end(&mut self) {
        todo!()
    }
}

#[derive(Debug)]
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
    let todo_prefix: &str = "TODO: ";
    let done_prefix: &str = "DONE: ";

    if line.starts_with("TODO: ") {
        Some((Status::Todo, &line[todo_prefix.len()..]))
    } else if line.starts_with(done_prefix) {
        Some((Status::Done, &line[done_prefix.len()..]))
    } else {
        None
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
        if *list_src_curr >= list_src.len() && list_src.len() > 0 {
            *list_src_curr = list_src.len() - 1;
        }
    }
}

// TODO: persist the state of the application
// TODO: add new elements to TODO
// TODO: delete items
// TODO: Edit the elements
// TODO: keep track of date when the item was DONE
// TODO: undo system

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
    let mut quit = false;
    let mut dones_curr: usize = 0;
    let mut todo_curr: usize = 0;
    let mut status = Status::Todo;

    {
        let file = File::open(file_path).unwrap();
        for line in BufReader::new(file).lines() {
            match parse_item(&line.unwrap()) {
                Some((Status::Todo, line)) => todos.push(line.to_string()),
                Some((Status::Done, line)) => dones.push(line.to_string()),
                None => {}
            }
        }
    }

    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    refresh();

    let mut ui = Ui::default();

    while !quit {
        erase();

        ui.begin(0, 0);
        {
            match status {
                Status::Todo => {
                    ui.label("[TODO] DONE ", REGULAR_PAIR);
                    ui.label("------------", REGULAR_PAIR);
                    ui.begin_list(todo_curr);
                    for (index, todo) in todos.iter().enumerate() {
                        ui.list_element(&format!("- [ ] {}", todo), index);
                    }
                    ui.end_list();
                }
                Status::Done => {
                    ui.label(" TODO [DONE]", REGULAR_PAIR);
                    ui.label("------------", REGULAR_PAIR);
                    ui.begin_list(dones_curr);
                    for (index, done) in dones.iter().enumerate() {
                        ui.list_element(&format!("- [X] {}", done), index);
                    }
                    ui.end_list();
                }
            }
        }
        refresh();

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            'w' => match status {
                Status::Todo => list_up(&mut todo_curr),
                Status::Done => list_up(&mut dones_curr),
            },
            's' => match status {
                Status::Todo => list_down(&todos, &mut todo_curr),
                Status::Done => list_down(&dones, &mut dones_curr),
            },
            '\n' => match status {
                Status::Todo => list_transfer(&mut dones, &mut todos, &mut todo_curr),
                Status::Done => list_transfer(&mut todos, &mut dones, &mut dones_curr),
            },
            '\t' => status = status.toggle(),
            'e' => {}
            'f' => {
                let mut file = File::create("TODO").unwrap();
                for todo in todos.iter() {
                    writeln!(file, "TODO: {}", todo).unwrap();
                }
                for done in dones.iter() {
                    writeln!(file, "DONE: {}", done).unwrap();
                }
            }

            _ => {}
        }
    }
    ui.end();

    endwin();
}
