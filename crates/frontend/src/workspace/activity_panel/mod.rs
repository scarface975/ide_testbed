use std::{pin::Pin, rc::Rc};

use dominator::{clone, events, svg, Dom, EventOptions};
use dominator_bulma::{block, column, columns, icon, icon_text};
use futures::StreamExt;
use futures_signals::{signal::{self, Mutable, Signal, SignalExt}, signal_vec::{MutableVec, SignalVecExt}};

pub mod editor;
pub mod welcome;

const TAB_HEIGHT: u32 = 48;

enum Activity {
    Editor(Rc<editor::Editor>),
    Welcome(Rc<welcome::Welcome>),
}

impl Activity {
    pub fn render(
        this: &Rc<Activity>,
        width: impl Signal<Item = u32> + 'static,
        height: impl Signal<Item = u32> + 'static
    ) -> Pin<Box<dyn Signal<Item = Option<dominator::Dom>>>> {
        match this.as_ref() {
            Activity::Editor(editor) => Box::pin(editor::Editor::render(editor, width, height)),
            Activity::Welcome(welcome) => Box::pin(welcome::Welcome::render(welcome, width, height)),
        }
    }

    pub fn label(&self) -> Dom {
        match self {
            Activity::Editor(editor) => editor.label(),
            Activity::Welcome(welcome) => welcome.label(),
        }
    }

    pub fn icon(&self) -> Dom {
        match self {
            Activity::Editor(editor) => editor.icon(),
            Activity::Welcome(welcome) => welcome.icon(),
        }
    }

    fn render_tab(
        this: &Rc<Activity>,
        panel: &Rc<ActivityPanel>
    ) -> dominator::Dom {
        let close_icon = svg!("svg", {
            .attr("height", "1em")
            .attr("viewBox", "0 0 24 24")
            .child(svg!("path", {
                .attr("d", CLOSE_ICON_PATH)
            }))
        });

        let mouse_over = Mutable::new(false);
        let mouse_over_close = Mutable::new(false);
        let is_active = panel.active_activity.signal_ref(clone!(this => move |active_activity| {
            active_activity.as_ref().is_some_and(|active_activity| Rc::ptr_eq(active_activity, &this))
        }));

        block!("py-3", "px-3", {
            .style("cursor", "pointer")
            .event(clone!(mouse_over => move |_: events::PointerOver| {
                mouse_over.set_neq(true);
            }))
            .event(clone!(mouse_over => move |_: events::PointerOut| {
                mouse_over.set_neq(false);
            }))
            .event(clone!(panel, this => move |_: events::PointerDown| {                
                panel.active_activity.set(Some(this.clone()))
            }))
            .class_signal("has-background-white",signal::or(is_active, mouse_over.signal()))
            .child(icon_text!({
                .child(icon!({
                    .child(this.icon())
                }))
                .child(this.label())
                // HACK DO NOT SHOW THE CLOSE ICON 
                .apply_if(matches!(**this, Activity::Editor(_)), |dom| {
                    dom.child(icon!({
                        .event(clone!(mouse_over_close => move |_: events::PointerOver| {
                            mouse_over_close.set_neq(true);
                        }))
                        .event(clone!(mouse_over_close => move |_: events::PointerOut| {
                            mouse_over_close.set_neq(false);
                        }))
                        .event_with_options(&EventOptions::preventable(), clone!(panel, this => move |ev: events::PointerDown| {
                            ev.stop_propagation();
                            panel.activities.lock_mut().retain(|activity| !Rc::ptr_eq(activity, &this));
                            let mut active_activity = panel.active_activity.lock_mut();
                            if active_activity.as_ref().is_some_and(|active_activity| Rc::ptr_eq(active_activity, &this)) {
                                // simple logic, VS Code is smart and keeps track of the last tab you looked at
                                *active_activity = panel.activities.lock_ref().first().cloned();
                            }
                        }))
                        .class_signal("has-background-white-ter", mouse_over_close.signal())
                        .class_signal("is-invisible", signal::not(mouse_over.signal()))
                        .child(close_icon)
                    }))
                })
            }))
        })
        
    }
}

pub struct ActivityPanel {
    activities: MutableVec<Rc<Activity>>,
    active_activity: Mutable<Option<Rc<Activity>>>
}

impl Default for ActivityPanel {
    fn default() -> Self {
        let welcome = Rc::new(Activity::Welcome(Rc::new(welcome::Welcome::new())));
        
        Self {
            activities: vec![welcome.clone()].into(),
            active_activity: Some(welcome).into()
        }
    }
}

// clicking a file in the explorer opens the file in the editor
// perhaps we just have a channel over which we send mutables? such that content can be synchronised
// how do I determine if a file is already open? Files should be uniquely identifiable from their
// paths

const CLOSE_ICON_PATH: &str = "M19,6.41L17.59,5L12,10.59L6.41,5L5,6.41L10.59,12L5,17.59L6.41,19L12,13.41L17.59,19L19,17.59L13.41,12L19,6.41Z";
//const CHANGED_ICON_PATH: &str = "M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z";

impl ActivityPanel {

    pub fn render(
        this: &Rc<ActivityPanel>,
        workspace_command_rx: crate::WorkspaceCommandReceiver,
        width: impl Signal<Item = u32> + 'static,
        height: impl Signal<Item = u32> + 'static
    ) -> dominator::Dom {

        let activity_count = this.activities.signal_vec_cloned().len().broadcast();
        let width = width.broadcast();
        let height = height.broadcast();

        columns!("is-gapless", "is-mobile", "is-multiline", {
            .future(workspace_command_rx.for_each(clone!(this => move |command| clone!(this => async move {
                match command {
                    crate::WorkspaceCommand::OpenFile(file) => {
                        let mut activities = this.activities.lock_mut();
                        let editor = activities.iter()
                            .find(|activity| match &***activity {
                                Activity::Editor(editor) => Rc::ptr_eq(&editor.file, &file),
                                _ => false,
                            })
                            .cloned()
                            .unwrap_or_else(move || {
                                let editor = Rc::new(Activity::Editor(Rc::new(editor::Editor::new(file))));
                                activities.push_cloned(editor.clone());
                                editor
                            });
                        this.active_activity.set(Some(editor));
                    },
                }
            }))))
            // this takes up the full height but should only display when there are no activities
            // and hence no tab bar
            .child_signal(activity_count.signal().map(clone!(height => move |count| {
                (count == 0).then(|| Self::render_background(height.signal()))
            })))
            // tabs take up one full line
            .child(column!("is-full", {
                .class("has-background-white-ter")
                .child(columns!("is-gapless", "is-mobile", {
                    .children_signal_vec(this.activities.signal_vec_cloned().map(clone!(this => move |activity| {
                        column!("is-narrow", {
                            .child(Activity::render_tab(&activity, &this))
                        })
                    })))
                }))
            }))
            .child_signal(this.active_activity
                .signal_cloned()
                .map(move |activity: Option<Rc<Activity>>| activity
                    .map(clone!(width, height => move |activity| column!("is-full", {
                        .child_signal(Activity::render(
                            &activity,
                            width.signal(),
                            height.signal_ref(|height| height.saturating_sub(TAB_HEIGHT))))
                    })))
                )
            )
        })
    }

    fn render_background(
        height: impl Signal<Item = u32> + 'static
    ) -> Dom {
        column!("is-full", {
            .style_signal("height", height.map(|height| format!("{height}px")))
            .style("background-image", "url('images/background.png')")
            .style("background-repeat", "no-repeat")
            .style("background-position", "center")
            .style("background-size", "auto 40%")
        })
    }
}
