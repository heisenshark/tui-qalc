use std::process::{Child, Command};
use cursive::event::Key;
use cursive::theme::{Style};
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, NamedView, SelectView, TextView,
};
use cursive::Cursive;
use cursive::{traits::*, Vec2};

fn main() {
    let x = Command::new("qalc")
        .arg("1 +1 1+1 +1 ")
        .output()
        .unwrap()
        .stdout;
    let mut siv = cursive::default();
    println!("{:?}", String::from_utf8(x));
    let history: Vec<String> = vec![];

    siv.add_global_callback(Key::Enter, |s| {
        let e = s.find_name::<EditView>("editv").unwrap();

        s.call_on_name("history", |history_view: &mut SelectView<String>| {
            history_view.add_item(e.get_content().to_string(), e.get_content().to_string());
        });
    });
    let xd: NamedView<TextView> = TextView::new("").with_name("expression");
    let mut history_view: NamedView<SelectView<String>> = SelectView::new().with_name("history");
    let mut dupa = EditView::new()
        .on_edit(|s, text, _| {
            let dupa = s.call_on_name("expression", |textobj: &mut TextView| {
                if text.len() <= 0 {
                    return;
                }
                textobj.set_content_wrap(true);
                match Command::new("qalc").arg(text).output() {
                    Ok(out) => {
                        let mut stout = String::from_utf8(out.stdout).unwrap();

                        if stout.matches("error").count() >= 1 {
                            textobj.set_style(Style::highlight());
                        } else {
                            textobj.set_style(Style::primary());
                        }
                        textobj.set_content(stout);
                    }
                    Err(e) => {
                        textobj.set_content(format!("qualc errored {:?}", e));
                    }
                }
            });
        })
        .with_name("editv");
    let layout = LinearLayout::vertical()
        .child(dupa)
        .child(xd)
        .child(history_view)
        .full_screen();
    siv.add_layer(layout);
    siv.run();
}
