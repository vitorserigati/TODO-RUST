use ncurses::*;
use std::cmp::*;

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

fn main() {
    initscr();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    refresh();

    let mut quit = false;
    let mut todos: Vec<String> = vec![
        "Write the todo app".to_string(),
        "Buy a bread".to_string(),
        "Feed the cat".to_string(),
    ];
    let mut dones: Vec<String> = vec![
        "Teste".to_string(),
        "Have a Breakfast".to_string(),
        "Study".to_string(),
    ];
    let mut todo_curr: usize = 0;

    let mut ui = Ui::default();

    while !quit {
        erase();

        ui.begin(0, 0);
        {
            ui.label("TODO: ", REGULAR_PAIR);
            ui.begin_list(todo_curr);
            for (index, todo) in todos.iter().enumerate() {
                ui.list_element(&format!("- [ ] {}", todo), index);
            }
            ui.end_list();

            ui.label(
                "---------------------------------------------------",
                REGULAR_PAIR,
            );

            ui.label("DONE: ", REGULAR_PAIR);
            ui.begin_list(0);
            for (index, done) in dones.iter().enumerate() {
                ui.list_element(&format!("- [X] {}", done), index + 1);
            }
            ui.end_list();
        }
        refresh();

        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            'w' => {
                if todo_curr > 0 {
                    todo_curr -= 1
                }
            }
            's' => {
                if todo_curr + 1 < todos.len() {
                    todo_curr = min(todo_curr + 1, todos.len() - 1)
                }
            }
            '\n' => {
                if todo_curr < todos.len() {
                    dones.push(todos.remove(todo_curr));
                }
            }
            _ => {}
        }
    }
    ui.end();

    endwin();
}
