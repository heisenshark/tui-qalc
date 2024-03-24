use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use closure::closure;
use cursive::{
    event::{Event, Key}, theme::Style, view::Nameable, views::{EditView, NamedView, OnEventView, SelectView}, Cursive
};
use regex::Regex;

pub fn create_expression_view<F>(
    on_edit: F,
    history_file: std::sync::Mutex<File>,
) -> OnEventView<NamedView<EditView>>
where
    F: 'static + Fn(&mut Cursive, &str, usize) + Send + Sync + Clone,
{
    let mut edit_v = EditView::new()
        .on_edit(on_edit.clone())
        .on_submit_mut(move |s, data| {
            let mut e = s.find_name::<EditView>("edit_view").unwrap();
            let generated = crate::qalc_cache(e.get_content().to_string());
            if generated.matches("error").count() >= 1 || data.to_string().trim().is_empty() {
                return;
            }
            s.call_on_name("history", |history_view: &mut SelectView<String>| {
                history_view.add_item(
                    format!("{} {}", e.get_content().to_string(), generated),
                    e.get_content().to_string(),
                );
                let mut lock = history_file.lock().unwrap();
                lock
                    .write(format!("{}\n{}", e.get_content().to_string(), generated).as_bytes())
            });
            e.set_content("");
        });
    let on_editt = on_edit.clone();
    let wrapped_edit_v = OnEventView::new(edit_v.with_name("edit_view"))
        .on_pre_event(Event::CtrlChar('h'), move |s| {
            let mut edit_view = s.find_name::<EditView>("edit_view").unwrap();
            edit_view.get_cursor();
            let content = edit_view.get_content();
            let (l, r) = content.split_at(edit_view.get_cursor());
            let l = String::from_iter(l.chars().rev());
            let re = Regex::new(r"^.\s{0,}[(\w\d]{0,}").unwrap();
            let res = re.replacen(&l, 0, "").to_string();
            let ss: String = res.chars().rev().into_iter().collect::<String>();
            let res = format!("{}{}", ss, r);
            edit_view.set_content(&res);
            edit_view.set_cursor(ss.len());
            on_editt(s, &res, 0);
        })
        .on_pre_event(Event::Ctrl(Key::Left), move |s| {
            let mut edit_view = s.find_name::<EditView>("edit_view").unwrap();
            edit_view.get_cursor();
            let content = edit_view.get_content();
            let (l, _) = content.split_at(edit_view.get_cursor());
            let l = String::from_iter(l.chars().rev());
            let re = Regex::new(r"^.\s{0,}[(\w\d]{0,}").unwrap();
            let res = re.replacen(&l, 0, "").to_string();
            let ss: String = res.chars().rev().into_iter().collect::<String>();
            edit_view.set_cursor(ss.len());
        })
        .on_pre_event(Event::Ctrl(Key::Right), move |s| {
            let mut edit_view = s.find_name::<EditView>("edit_view").unwrap();
            edit_view.get_cursor();
            let content = edit_view.get_content();
            let (l, r) = content.split_at(edit_view.get_cursor());
            let r = String::from_iter(r.chars());
            let re = Regex::new(r"^.\s{0,}[(\w\d]{0,}").unwrap();
            let regex_match = match re.find(&r) {
                Some(m) => m,
                None => return,
            };
            edit_view.set_cursor(l.len() + regex_match.end());
        })
        .on_event(Event::CtrlChar('d'), |s| {
            s.quit();
        })
        .on_event(Event::Ctrl(Key::Up), |s| {
            let _ = s.focus_name("history");
        })
        .on_pre_event(Event::CtrlChar('k'), |s| {
            let _ = s.focus_name("history");
        });
    return wrapped_edit_v;
}

pub fn open_history() -> (File, Vec<String>) {
    let _dir_create_res = fs::create_dir_all(Path::join(
        &dirs::home_dir().unwrap(),
        ".local/share/qalc-tui",
    ));
    // let xdd = File::create(Path::join(&dirs::home_dir().unwrap(), ".local/share/qalc-tui/history"));
    let file_create_res = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(Path::join(
            &dirs::home_dir().unwrap(),
            ".local/share/qalc-tui/history",
        ));
    let mut file_create_res = file_create_res.unwrap();
    let mut history_data = String::new();
    let mut history = file_create_res.read_to_string(&mut history_data).unwrap();
    let mut history_data: Vec<String> = history_data.lines().map(|x| x.to_string()).collect();
    return (file_create_res, history_data);
}
