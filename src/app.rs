use log::*;
use rand::{rngs::OsRng, seq::IteratorRandom};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};
use yew::format::Json;
use yew::prelude::*;
use yew::services::{
    storage::{Area, StorageService},
    DialogService, IntervalService, Task,
};

const KEY: &str = "automatic-spoon.self";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    dialog: DialogService,
    _interval: IntervalService,
    _heartbeat: Box<dyn Task>,
    state: State,
    view: View,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    lists: BTreeMap<String, Vec<Item>>,
    groups: BTreeMap<String, Vec<String>>,
}

#[derive(Default)]
pub struct View {
    current_list: String,
    new_list_name: String,
    current_group: String,
    new_group_name: String,
    cache: BTreeMap<String, Item>,
    current_item: Option<usize>,
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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Item {
    name: Option<String>,
    image: Option<String>,
    link: Option<String>,
    comment: Option<String>,
    // weight: f64?,
}

impl Item {
    pub fn render_chosen(&self) -> Html {
        if let Some(url) = self.link.as_ref() {
            html! {
                <div class="item">
                    <div class="name">
                        <a href=url.as_str() rel="noreferrer noopener" target="_blank">{self.name.as_ref().unwrap_or(url)}</a>
                    </div>
                    {self.image.as_ref().map(|image_url| html!{
                        <div class="image">
                            <a href=url.as_str() rel="noreferrer noopener" target="_blank">
                                <img src=image_url/>
                            </a>
                        </div>
                    }).unwrap_or_default()}
                    {self.comment.as_ref().map(|comment| html!{<div class="comment">{comment}</div>}).unwrap_or_default()}
                </div>
            }
        } else {
            html! {
                <div class={"item"}>
                    {self.name.as_ref().map(|name| html!{<div class="name">{name}</div>}).unwrap_or_default()}
                    {self.image.as_ref().map(|url| html!{<div class="image"><img src=url/></div>}).unwrap_or_default()}
                    {self.comment.as_ref().map(|comment| html!{<div class="comment">{comment}</div>}).unwrap_or_default()}
                </div>
            }
        }
    }
    pub fn render_edit(&self, link: &ComponentLink<App>) -> Html {
        html! {
            <div class="item">
            <ul>
            <li>
                <input id="item-name" class="edit" type="text" placeholder="Name"
                    value=self.name.as_ref().cloned().unwrap_or_default()
                    oninput=link.callback(move |e: InputData| Msg::EditItemName(e.value))
                />
                {self.image.as_ref().map(|url| html!{<div class="image"><img src=url/></div>}).unwrap_or_default()}
            </li>
            <li>
                <input id="item-image" class="edit" type="text" placeholder="Image URL"
                    value=&self.image.as_ref().cloned().unwrap_or_default()
                    oninput=link.callback(move |e: InputData| Msg::EditItemImage(e.value))
                />
            </li>
            <li>
                <input id="item-link" class="edit" type="text" placeholder="Link"
                    value=&self.link.as_ref().cloned().unwrap_or_default()
                    oninput=link.callback(move |e: InputData| Msg::EditItemLink(e.value))
                />
            </li>
            <li>
                <textarea id="item-comment" class="edit" placeholder="Comment"
                    oninput=link.callback(move |e: InputData| Msg::EditItemComment(e.value))
                >
                {&self.comment.as_ref().cloned().unwrap_or_default()}
                </textarea>
            </li>
            </ul>
            </div>
        }
    }
    pub fn render_flash(&self) -> Html {
        match (
            self.image.as_ref(),
            self.name.as_ref(),
            self.comment.as_ref(),
            self.link.as_ref(),
        ) {
            (Some(url), _, _, _) => html! {
                <img src=url/>
            },
            (None, Some(name), _, _) => html! {
                <p>{name}</p>
            },
            (None, None, Some(comment), _) => html! {
                <p>{comment}</p>
            },
            (None, None, None, Some(link)) => html! {
                // This is the flash content, so don't make an actual link
                <p>{link}</p>
            },
            _ => html! {
                <p>{"???"}</p>
            },
        }
    }
}

pub enum Msg {
    CreateItem,
    EditItemName(String),
    EditItemImage(String),
    EditItemLink(String),
    EditItemComment(String),
    FocusItem(usize),
    BlurItem,
    CreateList,
    FocusList(String),
    BlurList,
    UpdateListName(String),
    RemoveList(String),
    RemoveListItem(usize),
    CreateGroup,
    FocusGroup(String),
    BlurGroup,
    AddToGroup(String),
    UpdateGroupName(String),
    RemoveGroup(String),
    RemoveGroupItem(String),
    ThawAllLists,
    FreezeList(String),
    ThawList(String),
    Purge,
    Tick,
    Nothing,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let dialog = DialogService::new();
        let mut _interval = IntervalService::new();
        let _heartbeat =
            Box::new(_interval.spawn(Duration::from_millis(100), link.callback(|_| Msg::Tick)));
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
            _interval,
            _heartbeat,
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
            FocusList(name) => {
                self.view.current_list = name;
            }
            FocusGroup(name) => {
                self.view.current_group = name;
            }
            BlurList => {
                self.view.current_list = "".to_owned();
                self.view.current_item = None;
            }
            BlurGroup => {
                self.view.current_group = "".to_owned();
            }
            CreateItem => {
                self.view.current_item =
                    self.state
                        .lists
                        .get_mut(&self.view.current_list)
                        .map(|list| {
                            list.push(Item::default());
                            list.len() - 1
                        });
            }
            FocusItem(idx) => {
                self.view.current_item = Some(idx);
            }
            BlurItem => {
                self.view.current_item = None;
            }
            AddToGroup(entry) => {
                self.state
                    .groups
                    .get_mut(&self.view.current_group)
                    .map(|group| group.push(entry));
            }
            UpdateListName(text) => {
                self.view.new_list_name = text;
            }
            EditItemName(text) => {
                if let Some(item) = self.get_current_item_mut() {
                    item.name = match text.is_empty() {
                        true => None,
                        false => Some(text),
                    };
                }
            }
            EditItemImage(text) => {
                if let Some(item) = self.get_current_item_mut() {
                    item.image = match text.is_empty() {
                        true => None,
                        false => Some(text),
                    };
                }
            }
            EditItemLink(text) => {
                if let Some(item) = self.get_current_item_mut() {
                    item.link = match text.is_empty() {
                        true => None,
                        false => Some(text),
                    };
                }
            }
            EditItemComment(text) => {
                if let Some(item) = self.get_current_item_mut() {
                    item.comment = match text.is_empty() {
                        true => None,
                        false => Some(text),
                    };
                }
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
                            while let Some(idx) = group.iter().position(|x| *x == name) {
                                group.remove(idx);
                            }
                        }
                    }
                }
            }
            RemoveListItem(name) => {
                self.state
                    .lists
                    .get_mut(&self.view.current_list)
                    .map(|list| list.remove(name));
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
                    .map(|group| {
                        while let Some(idx) = group.iter().position(|x| *x == name) {
                            group.remove(idx);
                        }
                    });
            }
            FreezeList(name) => {
                let new = self.choose_from_list(&name);
                self.view.cache.insert(name, new);
            }
            ThawList(name) => {
                self.view.cache.remove(&name);
            }
            ThawAllLists => {
                self.view.cache.clear();
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
            Tick => {}
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
                {self.render_edit_item()}
                <button class="purge" onclick=self.link.callback(|_| Msg::Purge)>
                    {"Purge Everything"}
                </button>
            </div>
            </>
        }
    }
}

impl App {
    fn get_current_list(&self) -> Option<&Vec<Item>> {
        self.state.lists.get(&self.view.current_list)
    }
    fn get_current_list_mut(&mut self) -> Option<&mut Vec<Item>> {
        self.state.lists.get_mut(&self.view.current_list)
    }
    fn get_current_item_mut(&mut self) -> Option<&mut Item> {
        let maybe_index = self.view.current_item.clone();
        match (self.get_current_list_mut(), maybe_index) {
            (Some(list), Some(idx)) => list.get_mut(idx),
            _ => None,
        }
    }
    fn get_current_index_and_item(&self) -> Option<(usize, &Item)> {
        let maybe_index = self.view.current_item.clone();
        match (self.get_current_list(), maybe_index) {
            (Some(list), Some(idx)) => list.get(idx).map(|item| (idx, item)),
            _ => None,
        }
    }
    fn render_groups(&self) -> Html {
        html! {
            <div class="groups">
            <p>{"Groups"}</p>
            <ul>
                {
                    for self.state.groups.keys().map(|group| {
                        let name = group.to_owned();
                        let name2 = name.clone();
                        let (class,callback) = if name == self.view.current_group {
                            ("selected", self.link.callback(move |_| Msg::BlurGroup))
                        } else {
                            ("", self.link.callback(move |_| Msg::FocusGroup(name.clone())))
                        };
                        html! {
                            <li
                                class=class
                                onclick=callback
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
                    <button onclick=self.link.callback(move |_| Msg::ThawAllLists)>
                        {"Unlock All Lists"}
                    </button>
                    <button class="delete" onclick=self.link.callback(move |_| Msg::RemoveGroup(name.clone()))>
                        {"Delete Group"}
                    </button>
                    <dl>
                        {for group.iter().map(|entry| { self.render_group_element(entry)})}
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
    fn render_group_element(&self, name: &str) -> Html {
        let name2 = name.to_owned();
        match self.view.cache.get(name) {
            Some(item) => html! {
                <>
                <dt>{name}</dt>
                <dd>{item.render_chosen()}
                <button class="delete" onclick=self.link.callback(move |_| Msg::ThawList(name2.clone()))>
                    {"Unlock"}
                </button>
                </dd>
                </>
            },
            None => {
                let item = self.choose_from_list(&name);
                html! {
                    <>
                    <dt>{name}</dt>
                    <dd
                        onclick=self.link.callback(move |_| Msg::FreezeList(name2.clone()) )
                    >{item.render_flash()}
                    </dd>
                    </>
                }
            }
        }
    }
    fn render_list_name(&self, name: &str) -> Html {
        let name3 = name.to_owned();
        let (class, callback) = if name == self.view.current_list {
            ("selected", self.link.callback(move |_| Msg::BlurList))
        } else {
            (
                "",
                self.link.callback(move |_| Msg::FocusList(name3.clone())),
            )
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
                onclick=callback
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
                    for self.state.lists.keys().map(|name| {self.render_list_name(name)})
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
    fn render_list_entry(&self, idx: usize, item: &Item) -> Html {
        let name = item
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format!("{}", idx));
        let idx2 = idx;
        let (class, callback) = if Some(idx) == self.view.current_item {
            ("selected", self.link.callback(|_| Msg::BlurItem))
        } else {
            ("", self.link.callback(move |_| Msg::FocusItem(idx2)))
        };
        html! {
            <li class=class
                onclick=callback
            >
                <button onclick=self.link.callback(move |_| Msg::RemoveListItem(idx))>
                    {"-"}
                </button>
                {name}
            </li>
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
                    {for list.iter().enumerate().map(|(idx, item)| self.render_list_entry(idx, &item))}
                    <li>
                        <button onclick=self.link.callback(move |_| Msg::CreateItem)>
                            {"+"}
                        </button>
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
    fn render_edit_item(&self) -> Html {
        if let Some((_idx, item)) = self.get_current_index_and_item() {
            html! {
                <div class="edit-item">
                {item.render_edit(&self.link)}
                </div>
            }
        } else {
            html! {
                <div class="edit-item"></div>
            }
        }
    }
    fn choose_from_list(&self, name: &str) -> Item {
        let mut rng: OsRng = Default::default();
        let item: Item = self
            .state
            .lists
            .get(name)
            .map(|list| list.iter().choose(&mut rng).unwrap().to_owned())
            .unwrap_or_default();
        item
    }
}
