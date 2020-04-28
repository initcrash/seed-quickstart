use seed::{prelude::*, *};

/* use futures::Future; */

struct Model {
    pub change: Option<Change>,
}

impl Default for Model {
    fn default() -> Self {
        Self { change: None }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct Change {
    pub diff: String,
}

#[derive(Clone)]
enum Msg {
    DataFetched(Change),
    FetchData,
}


fn code_line(cls: &str, oline: &str, nline: &str, value: &str) -> Node<Msg> {
    return tr![
        attrs!(At::Class => {cls}),
        td![attrs!(At::Class => "linenr"), pre!({ oline }),],
        td![attrs!(At::Class => "linenr"), pre!({ nline }),],
        td![attrs!(At::Class => "code"), pre!({ value }),]
    ];
}

fn unified_diff(diff: &str) -> Node<Msg> {
    let mut current_ofile = "".to_string();
    let mut current_nfile = "".to_string();

    let mut arr = vec![];

    let mut oline = 0;
    let mut nline = 0;
    for value in diff.split("\n") {
        if value.starts_with("@") {
            continue;
        }
        if value.starts_with("+++") {
            continue;
        }
        if value.starts_with("---") {
            continue;
        }
        if value.starts_with("index") {
            if let [idx, ff, rest] = value.split(" ").collect::<Vec<_>>().as_slice() {
                if let [o, n] = ff.split("..").collect::<Vec<_>>().as_slice() {
                    current_ofile = o.to_string();
                    current_nfile = n.to_string();
                }
            }
            continue;
        }
        if value.starts_with("diff --git") {
            if let [d, g, a, b] = value.split(" ").collect::<Vec<_>>().as_slice() {
                oline = 1;
                nline = 1;
                arr.push(tr![
                    attrs!(At::Class=>"head"),
                    td![attrs!(At::ColSpan=> 3), pre![""]]
                ]);
                arr.push(tr![
                    attrs!(At::Class=>"head"),
                    td![
                        attrs!(At::ColSpan=> 3),
                        pre![format!("{} -> {}", &a[2..], &b[2..])]
                    ]
                ]);
                arr.push(tr![
                    attrs!(At::Class=>"head"),
                    td![attrs!(At::ColSpan=> 3), pre![""]]
                ]);
                continue;
            }
        }
        if value.starts_with(" ") {
            arr.push(code_line(
                "",
                &oline.to_string(),
                &nline.to_string(),
                &value[1..],
            ));
            oline += 1;
            nline += 1;
        } else if value.starts_with("+") {
            arr.push(code_line("addition", "", &nline.to_string(), &value[1..]));
            nline += 1;
        } else if value.starts_with("-") {
            arr.push(code_line("removal", &oline.to_string(), "", &value[1..]));
            oline += 1;
        }
    }

    return table![tbody![{ arr }]];
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchData => {
            orders.skip();
            orders.perform_cmd( async {
                let url = "/public/sample.diff";
                let response = fetch(url).await.expect("fetch failed");

                let change = response
                    .check_status()
                    .expect("check_status failed")
                    .json::<Change>()
                    .await
                    .expect("deserialization failed");

                Msg::DataFetched(change)
            });
        }

        Msg::DataFetched(change) => model.change = Some(change),
    }
}

fn view(model: &Model) -> Node<Msg> {
    if let Some(change) = &model.change {
        unified_diff(&change.diff)
    } else {
        div!["No diff"]
    }
}

fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    orders.send_msg(Msg::FetchData);
    AfterMount::default()
}

#[wasm_bindgen(start)]
pub fn render() {
    App::builder(update, view)
        .after_mount(after_mount)
        .build_and_start();
}
