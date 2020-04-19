use log::*;
use rand::{rngs::OsRng, seq::IteratorRandom};
use serde_derive::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};
use yew::format::Json;
use yew::prelude::*;
use yew::services::{
    storage::{Area, StorageService},
    DialogService,
};

const KEY: &str = "automatic-spoon.self";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    dialog: DialogService,
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
    cache: RefCell<BTreeMap<String, String>>,
}

impl View {
    pub fn new(current_list: String, current_group: String) -> Self {
        Self {
            current_list,
            current_group,
            ..Default::default()
        }
    }
}

pub enum Msg {
    CreateList,
    SelectList(String),
    AddToList,
    UpdateListName(String),
    UpdateListAddition(String),
    RemoveList(String),
    RemoveListItem(String),
    CreateGroup,
    SelectGroup(String),
    AddToGroup(String),
    UpdateGroupName(String),
    RemoveGroup(String),
    RemoveGroupItem(String),
    Regenerate,
    Purge,
    Nothing,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let dialog = DialogService::new();
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
            dialog,
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
            SelectList(name) => {
                self.view.current_list = name;
            }
            SelectGroup(name) => {
                self.view.current_group = name;
            }
            AddToList => {
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
                if self
                    .dialog
                    .confirm(&format!("Really delete list {}?", name))
                {
                    let removed = self.state.lists.remove(&name);
                    if removed.is_some() {
                        for (_, group) in self.state.groups.iter_mut() {
                            group.remove(&name);
                        }
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
                if self
                    .dialog
                    .confirm(&format!("Really delete group {}?", name))
                {
                    self.state.groups.remove(&name);
                }
            }
            RemoveGroupItem(name) => {
                self.state
                    .groups
                    .get_mut(&self.view.current_group)
                    .map(|group| group.remove(&name));
            }
            Regenerate => {
                self.view.cache.get_mut().clear();
            }
            Purge => {
                if self
                    .dialog
                    .confirm("Really delete all saved lists and groups?")
                {
                    self.state = State::default();
                    self.view = View::default();
                }
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
            <div class={"autospoon"}>
                { self.render_groups()}
                { self.render_group()}
                { self.render_lists()}
                {self.render_list()}
                <button class="purge" onclick=self.link.callback(|_| Msg::Purge)>
                    {"Purge Everything"}
                </button>
            </div>
            </>
        }
    }
}

impl App {
    fn render_groups(&self) -> Html {
        html! {
            <div class="groups">
            <p>{"Groups"}</p>
            <ul>
                {
                    for self.state.groups.keys().map(|group| {
                        let name = group.to_owned();
                        let name2 = name.clone();
                        let class = if name == self.view.current_group {
                            "selected"
                        } else {
                            ""
                        };
                        html! {
                            <li
                                class=class
                                onclick=self.link.callback(move |_| Msg::SelectGroup(name.clone()))
                            > {group}
                                <button class="delete" onclick=self.link.callback(move |_| Msg::RemoveGroup(name2.clone()))>
                                    {"Delete"}
                                </button>
                            </li>
                        }
                    })
                }
                <li>
                    <input class="edit"
                        type="text"
                        placeholder="New Group"
                        value=&self.view.new_group_name
                        oninput=self.link.callback(move |e: InputData| Msg::UpdateGroupName(e.value))
                        onkeypress=self.link.callback(move |e: KeyboardEvent| {
                            if e.key() == "Enter" { Msg::CreateGroup } else { Msg::Nothing }
                    }) />
                </li>
            </ul>
            </div>
        }
    }
    fn render_group(&self) -> Html {
        if let Some(group) = self.state.groups.get(&self.view.current_group) {
            let name = self.view.current_group.to_owned();
            html! {
                <div class="group">
                    <p>{&name}</p>
                    <button onclick=self.link.callback(move |_| Msg::Regenerate)>
                        {"Regenerate"}
                    </button>
                    <button class="delete" onclick=self.link.callback(move |_| Msg::RemoveGroup(name.clone()))>
                        {"Delete Group"}
                    </button>
                    <dl>
                        {for group.iter().map(|entry| {
                            html! {
                                <>
                                <dt>{entry}</dt>
                                <dd>{self.get_random_element(entry)}</dd>
                                </>
                            }
                        })}
                    </dl>
                </div>
            }
        } else {
            html! {
                <div class="group">
                </div>
            }
        }
    }
    fn render_list_item(&self, name: &str) -> Html {
        let name3 = name.to_owned();
        let class = if name == self.view.current_list {
            "selected"
        } else {
            ""
        };
        let buttons = if self.view.current_group != "" {
            let name1 = name.to_owned();
            let name2 = name.to_owned();
            html! {
                <>
                <button class="add" onclick=self.link.callback(move |_| Msg::AddToGroup(name1.clone()))>
                    {"+"}
                </button>
                <button class="remove" onclick=self.link.callback(move |_| Msg::RemoveGroupItem(name2.clone()))>
                    {"-"}
                </button>
                </>
            }
        } else {
            html! {<></>}
        };
        html! {
            <li
                class=class
                onclick=self.link.callback(move |_| Msg::SelectList(name3.clone()))
            >
                {buttons}
                {name}
            </li>
        }
    }
    fn render_lists(&self) -> Html {
        html! {
            <div  class="lists">
            <p>{"Lists"}</p>
            <ul>
                {
                    for self.state.lists.keys().map(|name| {self.render_list_item(name)})
                }
                <li>
                    <input class="edit"
                        type="text"
                        placeholder="New List"
                        value=&self.view.new_list_name
                        oninput=self.link.callback(move |e: InputData| Msg::UpdateListName(e.value))
                        onkeypress=self.link.callback(move |e: KeyboardEvent| {
                            if e.key() == "Enter" { Msg::CreateList } else { Msg::Nothing }
                    }) />
                </li>
            </ul>
            </div>
        }
    }
    fn render_list(&self) -> Html {
        if let Some(list) = self.state.lists.get(&self.view.current_list) {
            let name = self.view.current_list.to_owned();
            html! {
                <div class="list">
                <p>{&name}</p>
                <button class="delete" onclick=self.link.callback(move |_| Msg::RemoveList(name.clone()))>
                    {"Delete List"}
                </button>
                <ul class="entries">
                    {for list.iter().map(|entry| {
                        let name = entry.to_owned();
                        html! {
                                <li>
                                    <button onclick=self.link.callback(move |_| Msg::RemoveListItem(name.clone()))>
                                        {"-"}
                                    </button>
                                    {entry}
                                </li>
                            }
                        })
                    }
                    <li>
                        <input class="edit"
                            type="text"
                            placeholder="New List Item"
                            value=&self.view.add_to_list
                            oninput=self.link.callback(move |e: InputData| Msg::UpdateListAddition(e.value))
                            onkeypress=self.link.callback(move |e: KeyboardEvent| {
                                if e.key() == "Enter" { Msg::AddToList } else { Msg::Nothing }
                        }) />
                    </li>
                </ul>
                </div>
            }
        } else {
            html! {
                <div class="list"></div>
            }
        }
    }
    fn get_random_element(&self, name: &str) -> String {
        // XXX TODO Ew, this is ugly.
        if self.view.cache.borrow().contains_key(name) {
            return self.view.cache.borrow().get(name).unwrap().to_owned();
        };
        let mut rng: OsRng = Default::default();
        let item = self
            .state
            .lists
            .get(name)
            .map(|list| list.iter().choose(&mut rng).unwrap().to_owned())
            .unwrap_or("".to_owned());
        self.view.cache.borrow_mut().insert(name.to_owned(), item);
        self.view.cache.borrow().get(name).unwrap().to_owned()
    }
}
