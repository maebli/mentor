use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

const ORIGINAL: &str = "Original plan";

fn prefix(tool: &str) -> String {
    format!("mentor:plans:{tool}:")
}

fn source_key(tool: &str, name: &str) -> String {
    if name.is_empty() {
        format!("mentor:{tool}")
    } else {
        format!("{}{name}", prefix(tool))
    }
}

pub fn names(tool: &str) -> Vec<String> {
    let Ok(Some(storage)) = window().local_storage() else {
        return Vec::new();
    };
    let prefix = prefix(tool);
    let mut names: Vec<_> = (0..storage.length().unwrap_or_default())
        .filter_map(|index| storage.key(index).ok().flatten())
        .filter_map(|key| key.strip_prefix(&prefix).map(str::to_owned))
        .collect();
    names.sort();
    names
}

pub fn read(tool: &str, name: &str) -> Option<String> {
    LocalStorage::get(source_key(tool, name)).ok()
}

#[derive(Clone, Copy)]
pub struct Plans {
    tool: &'static str,
    active: RwSignal<String>,
    pub text: RwSignal<String>,
    message: RwSignal<String>,
}

impl Plans {
    pub fn open(tool: &'static str, default: &str) -> Self {
        let last: String = LocalStorage::get(format!("mentor:active:{tool}")).unwrap_or_default();
        let (name, source) = match read(tool, &last) {
            Some(source) => (last, source),
            None => (
                String::new(),
                read(tool, "").unwrap_or_else(|| default.into()),
            ),
        };
        let plans = Self {
            tool,
            active: RwSignal::new(name),
            text: RwSignal::new(source),
            message: RwSignal::new(String::new()),
        };
        Effect::new(move |_| {
            let source = plans.text.get();
            let name = plans.active.get();
            if LocalStorage::set(source_key(tool, &name), &source).is_err() {
                plans
                    .message
                    .set("Browser storage is unavailable. Use Save text to keep your work.".into());
            }
        });
        plans
    }

    pub fn filename(self, extension: &str) -> String {
        let name = self.active.get_untracked();
        if name.is_empty() {
            return format!("{}.{extension}", self.tool);
        }
        let safe_name: String = name
            .chars()
            .map(|ch| {
                if ch.is_alphanumeric() || matches!(ch, ' ' | '-' | '_') {
                    ch
                } else {
                    '_'
                }
            })
            .collect();
        format!("{}-{safe_name}.{extension}", self.tool)
    }

    fn select(self, name: String, source: String) {
        self.active.set(name.clone());
        self.text.set(source);
        self.message.set(String::new());
        let _ = LocalStorage::set(format!("mentor:active:{}", self.tool), name);
    }
}

#[component]
pub fn PlanPicker(plans: Plans) -> impl IntoView {
    let available = RwSignal::new(names(plans.tool));
    let adding = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let field = NodeRef::<leptos::html::Input>::new();
    let select = NodeRef::<leptos::html::Select>::new();

    let switch_plan = move |ev| {
        let name = event_target_value(&ev);
        if let Some(source) = read(plans.tool, &name) {
            plans.select(name, source);
        } else {
            plans
                .message
                .set("Could not open this plan in browser storage.".into());
        }
    };
    let save_copy = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name = draft.get_untracked();
        let name = name.trim();
        if name.is_empty() {
            plans.message.set("Enter a name for this plan.".into());
            return;
        }
        if name.eq_ignore_ascii_case(ORIGINAL) || read(plans.tool, name).is_some() {
            plans
                .message
                .set("A plan already has that name. Choose another name.".into());
            return;
        }
        let source = plans.text.get_untracked();
        if LocalStorage::set(source_key(plans.tool, name), &source).is_err() {
            plans
                .message
                .set("Could not save this copy. Use Save text to keep your work.".into());
            return;
        }
        available.set(names(plans.tool));
        plans.select(name.to_owned(), source);
        draft.set(String::new());
        adding.set(false);
        if let Some(select) = select.get() {
            let _ = select.focus();
        }
    };

    view! {
        <div class="plan-picker">
            <label for="active-plan">"Plan"</label>
            <select id="active-plan" node_ref=select on:change=switch_plan
                on:focus=move |_| available.set(names(plans.tool))>
                <option value="" prop:selected=move || plans.active.with(String::is_empty)>{ORIGINAL}</option>
                {move || available.get().into_iter().map(|name| {
                    let value = name.clone();
                    view! { <option value=value prop:selected=move || plans.active.with(|active| active == &name)>{name.clone()}</option> }
                }).collect_view()}
            </select>
            <button type="button" class="plan-button" aria-expanded=move || adding.get().to_string()
                on:click=move |_| {
                    adding.set(!adding.get_untracked());
                    plans.message.set(String::new());
                    if adding.get_untracked() {
                        queue_microtask(move || {
                            if let Some(input) = field.get_untracked() {
                                let _ = input.focus();
                            }
                        });
                    }
                }>{move || if adding.get() { "Cancel" } else { "Save a copy" }}</button>
            <form hidden=move || !adding.get() on:submit=save_copy>
                <label for="plan-name">"New plan name"</label>
                <input id="plan-name" node_ref=field required maxlength="80" autocomplete="off"
                    prop:value=move || draft.get() on:input=move |ev| draft.set(event_target_value(&ev)) />
                <button class="plan-button" type="submit">"Create plan"</button>
            </form>
            <span role="status" class="plan-status">{move || plans.message.get()}</span>
        </div>
    }
}
