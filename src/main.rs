pub mod async_text_view;
pub mod expression_view;
use crate::async_text_view::AsyncTextView;
use cursive::event::{Event, Key};
use cursive::theme::{Color, ColorStyle, Style};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::utils::span::SpannedString;
use cursive::views::{EditView, LinearLayout, OnEventView, SelectView};
use cursive::Cursive;
use expression_view::{create_expression_view, open_history};
use itertools::Itertools;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self};

fn main() {
    let (mut history_file, mut history_lines) = open_history();
    let mut siv = cursive::default();
    let cb_sink = siv.cb_sink().clone();
    siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());
    let (tx, rx) = mpsc::channel();
    let expression_value = Arc::new(Mutex::new("".to_owned()));
    let ex_ref = expression_value.to_owned();
    thread::spawn(move || {
        update_message(&tx, cb_sink, &ex_ref);
    });
    let ex_ref = expression_value.clone();
    let mut history_inner: SelectView<String> =
        SelectView::<String>::new().on_submit(move |s: &mut Cursive, data: &str| {
            let mut edit_view = s.find_name::<EditView>("edit_view").unwrap();
            edit_view.set_content(data);
            let mut lock = ex_ref.lock().unwrap();
            *lock = data.into();
            let _ = s.focus_name("edit_view");
        });
    let iter_his = history_lines
        .chunks_mut(2)
        .into_iter()
        .map(|chunk| {
            let mut chiter = chunk.iter();
            let value = chiter.next().unwrap().to_string();
            let mut ss = String::new();
            let result = chiter.next().unwrap_or_else(|| {&mut ss});
            Style::primary();
            let sss = ColorStyle::new(Color::TerminalDefault,Color::TerminalDefault);
            let stringg= SpannedString::single_span(format!("{} {}", value,result),Style::from(sss));
            // StyledString::from(format!("{} {}", valuer result)); 
            return (stringg, value);
        })
        .into_iter();
    history_inner.add_all(iter_his);
    let history_view = OnEventView::new(history_inner.with_name("history"))
        .on_event('j', |s| {
            let mut hist = s.find_name::<SelectView>("history").unwrap();
            hist.select_down(1);
        })
        .on_event('k', |s| {
            let mut hist = s.find_name::<SelectView>("history").unwrap();
            hist.select_up(1);
        })
        .on_event(Event::Key(Key::PageDown), |s| {
            let mut hist = s.find_name::<SelectView>("history").unwrap();
            hist.select_down(10);
        })
        .on_event(Event::Key(Key::PageUp), |s| {
            let mut hist = s.find_name::<SelectView>("history").unwrap();
            hist.select_up(10);
        })
        .on_pre_event(Event::Ctrl(Key::Down), |s| {
            let _ = s.focus_name("edit_view");
        })
        .on_pre_event(Event::CtrlChar('k'), |s| {
            let _ = s.focus_name("edit_view");
        })
        .on_pre_event(Event::CtrlChar('j'), |s| {
            let _ = s.focus_name("edit_view");
        });
    let ex_ref = expression_value.to_owned();
    let on_edit = move |_: &mut Cursive, b: &str, _: usize| {
        let mut lock = ex_ref.lock().unwrap();
        *lock = b.into();
    };

    let expression_view = create_expression_view(on_edit.clone(), &mut history_file);
    let result_preview = AsyncTextView::new("", "".to_owned(), rx);
    let layout = LinearLayout::vertical()
        .child(
            LinearLayout::vertical()
                .child(history_view)
                .scrollable()
                .scroll_y(true)
                .full_height(),
        )
        .child(expression_view)
        .child(result_preview)
        .full_width();
    siv.add_fullscreen_layer(layout);
    siv.run();
}

#[cached::proc_macro::cached]
pub fn qalc_cache(equation: String) -> String {
    match Command::new("qalc").arg(equation).output() {
        Ok(out) => {
            return String::from_utf8(out.stdout).unwrap();
        }
        Err(_) => {
            return "error???!!!".to_owned();
        }
    }
}

fn update_message(tx: &mpsc::Sender<String>, cb_sink: cursive::CbSink, vedit_ref: &Mutex<String>) {
    loop {
        thread::sleep(std::time::Duration::from_millis(40));
        let xd = vedit_ref.lock().unwrap();
        let xx = qalc_cache(xd.to_owned());
        if tx.send(xx).is_err() {
            return;
        }
        cb_sink.send(Box::new(Cursive::noop)).unwrap();
    }
}
