pub mod async_text_view;
pub mod expression_view;
use crate::async_text_view::AsyncTextView;
use cursive::event::Event;
use cursive::traits::*;
use cursive::views::{EditView, LinearLayout, SelectView};
use cursive::Cursive;
use expression_view::create_expression_view;
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self};

fn main() {
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
    let history_view = SelectView::<String>::new()
        .on_submit(move |s: &mut Cursive, data: &str| {
            let mut edit_view = s.find_name::<EditView>("edit_view").unwrap();
            edit_view.set_content(data);
            let mut lock = ex_ref.lock().unwrap();
            *lock = data.into();
        })
        .with_name("history");
    let ex_ref = expression_value.to_owned();
    let on_edit = move |_: &mut Cursive, b: &str, _: usize| {
        let mut lock = ex_ref.lock().unwrap();
        *lock = b.into();
    };
    let expression_view = create_expression_view(on_edit.clone());
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
