use log::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use yew::format::Json;
use yew::prelude::*;
use yew::services::storage::{Area, StorageService};

const KEY: &str = "automatic-spoon.self";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    view: View,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    lists: BTreeMap<String, BTreeSet<String>>,
    groups: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Default)]
pub struct View {
    current_list: String,
    new_list_name: String,
    add_to_list: String,
    current_group: String,
    new_group_name: String,
}

impl View {
    pub fn new(current_list: String, current_group: String) -> Self {
        Self {
            current_list,
            new_list_name: "".to_owned(),
            add_to_list: "".to_owned(),
            current_group,
            new_group_name: "".to_owned(),
        }
    }
}

pub enum Msg {
    CreateList,
    CreateGroup,
    AddToSet,
    AddToGroup(String),
    UpdateListName(String),
    UpdateListAddition(String),
    UpdateGroupName(String),
    RemoveList(String),
    RemoveListItem(String),
    RemoveGroup(String),
    RemoveGroupItem(String),
    Purge,
    Nothing,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let state = {
            if let Json(Ok(restored_entries)) = storage.restore(KEY) {
                restored_entries
            } else {
                State::default()
            }
        };
        let current_list = state.lists.keys().cloned().next().unwrap_or_default();
        let current_group = state.groups.keys().cloned().next().unwrap_or_default();
        let view = View::new(current_list, current_group);
        App {
            link,
            storage,
            state,
            view,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        use Msg::*;
        match msg {
            CreateList => {
                let _ = self
                    .state
                    .lists
                    .entry(self.view.new_list_name.clone())
                    .or_default();
                self.view.current_list = self.view.new_list_name.split_off(0);
            }
            CreateGroup => {
                let _ = self
                    .state
                    .groups
                    .entry(self.view.new_group_name.clone())
                    .or_default();
                self.view.current_group = self.view.new_group_name.split_off(0);
            }
            AddToSet => {
                let entry = self.view.add_to_list.clone();
                self.view.add_to_list.truncate(0);
                self.state
                    .lists
                    .get_mut(&self.view.current_list)
                    .map(|list| list.insert(entry));
            }
            AddToGroup(entry) => {
                self.state
                    .groups
                    .get_mut(&self.view.current_group)
                    .map(|group| group.insert(entry));
            }
            UpdateListName(text) => {
                self.view.new_list_name = text;
            }
            UpdateListAddition(text) => {
                self.view.add_to_list = text;
            }
            UpdateGroupName(text) => {
                self.view.new_group_name = text;
            }
            RemoveList(name) => {
                let removed = self.state.lists.remove(&name);
                if removed.is_some() {
                    for (_, group) in self.state.groups.iter_mut() {
                        group.remove(&name);
                    }
                }
            }
            RemoveListItem(name) => {
                self.state
                    .lists
                    .get_mut(&self.view.current_list)
                    .map(|list| list.remove(&name));
            }
            RemoveGroup(name) => {
                self.state.groups.remove(&name);
            }
            RemoveGroupItem(name) => {
                self.state
                    .groups
                    .get_mut(&self.view.current_group)
                    .map(|group| group.remove(&name));
            }
            Purge => {
                self.state = State::default();
                self.view = View::default();
            }
            Nothing => {}
        }
        self.storage.store(KEY, Json(&self.state));
        true
    }

    fn view(&self) -> Html {
        info!("rendered!");
        html! {
            <>
            <h1>{"Automatic Spoon!"}</h1>
            <h2>{"Groups"}</h2>
            { self.render_groups()}
            <h2>{"Lists"}</h2>
            { self.render_lists()}
            {self.render_list()}
            <button class="purge" onclick=self.link.callback(|_| Msg::Purge)>
                {"Purge Everything"}
            </button>
            </>
        }
    }
}

impl App {
    fn render_groups(&self) -> Html {
        html! {
            <ul id="groups">
                {
                    for self.state.groups.keys().map(|group| {
                        html! {
                            <li> {group} </li>
                        }
                    })
                }
                <li>
                    <input class="edit"
                        type="text"
                        value=&self.view.new_group_name
                        oninput=self.link.callback(move |e: InputData| Msg::UpdateGroupName(e.value))
                        onkeypress=self.link.callback(move |e: KeyboardEvent| {
                            if e.key() == "Enter" { Msg::CreateGroup } else { Msg::Nothing }
                    }) />
                </li>
            </ul>
        }
    }
    fn render_lists(&self) -> Html {
        html! {
            <ul id="lists">
                {
                    for self.state.lists.keys().map(|list| {
                        html! {
                            <li> {list} </li>
                        }
                    })
                }
                <li>
                    <input class="edit"
                        type="text"
                        value=&self.view.new_list_name
                        oninput=self.link.callback(move |e: InputData| Msg::UpdateListName(e.value))
                        onkeypress=self.link.callback(move |e: KeyboardEvent| {
                            if e.key() == "Enter" { Msg::CreateList } else { Msg::Nothing }
                    }) />
                </li>
            </ul>
        }
    }
    fn render_list(&self) -> Html {
        if let Some(list) = self.state.lists.get(&self.view.current_list) {
            html! {
                <>
                <h2>{"Entries in list "}{&self.view.current_list}</h2>
                <ul id="entries">
                    {for list.iter().map(|entry| {
                        html! {
                                <li> {entry} </li>
                            }
                        })
                    }
                    <li>
                        <input class="edit"
                            type="text"
                            value=&self.view.add_to_list
                            oninput=self.link.callback(move |e: InputData| Msg::UpdateListAddition(e.value))
                            onkeypress=self.link.callback(move |e: KeyboardEvent| {
                                if e.key() == "Enter" { Msg::AddToSet } else { Msg::Nothing }
                        }) />
                    </li>
                </ul>
                </>
            }
        } else {
            html! {
                <></>
            }
        }
    }
}
