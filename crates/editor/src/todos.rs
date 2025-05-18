use crate::Editor;
use anyhow::Result;
use gpui::{actions, AppContext, DismissEvent, div, prelude::*, WindowContext};
use project::{Project, TodoEntry};
use ui::{prelude::*, ListState, ListView, Icon, IconButton, IconName};
use workspace::{Panel, Workspace, panel};

actions!(todos, [ShowTodos, ToggleFocus]);

impl Editor {
    pub fn show_todos(&mut self, _: &ShowTodos, cx: &mut WindowContext<'_, Editor>) {
        if let Some(project) = &self.project {
            let weak_editor = cx.entity_id().into();
            let workspace = self.workspace();
            
            cx.spawn(|mut cx| async move {
                let todos = project.update(&mut cx, |project, cx| {
                    project.search_todos(cx)
                })?;

                cx.update(|cx| {
                    if let Some(workspace) = workspace {
                        workspace.update(cx, |workspace, cx| {
                            workspace.show_panel::<TodosPanel>(cx);
                            if let Some(panel) = workspace.panel::<TodosPanel>(cx) {
                                panel.update(cx, |panel, cx| {
                                    panel.set_todos(todos, cx);
                                });
                            }
                        })?;
                    }
                    Ok::<_, anyhow::Error>(())
                })?;
                
                Ok(())
            })
            .detach_and_log_err(cx);
        }
    }
}

pub struct TodosPanel {
    todos: Vec<TodoEntry>,
    list_state: ListState,
}

impl TodosPanel {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
            list_state: ListState::default(),
        }
    }

    pub fn set_todos(&mut self, todos: Vec<TodoEntry>, cx: &mut WindowContext) {
        self.todos = todos;
        self.list_state.scroll_to_top(cx);
    }
}

impl Panel for TodosPanel {
    fn persistent_name() -> &'static str {
        "TODOs" 
    }

    fn position(&self) -> panel::Position {
        panel::Position::Right
    }

    fn icon(&self) -> Option<IconName> {
        Some(IconName::Check)
    }

    fn toggle_action() -> Box<dyn gpui::Action> {
        ToggleFocus.boxed_clone()
    }

    fn render(&mut self, cx: &mut WindowContext) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .w_full()
                    .px_2()
                    .py_1()
                    .border_b()
                    .border_color(cx.theme().colors().border)
                    .child(h_flex().gap_1().child("TODOs").child(format!("({})", self.todos.len())))
            )
            .child(
                ListView::new()
                    .size_full()
                    .flex_grow()
                    .tracks(["list"])
                    .on_click(move |event, cx| {
                        if let Some(todo) = self.todos.get(event.item.index) {
                            cx.window_context().update(|cx| {
                                if let Some(workspace) = workspace::Workspace::find_focused(cx) {
                                    workspace.update(cx, |workspace, window, cx| {
                                        workspace.open_path(
                                            todo.path.clone(),
                                            Some(todo.range.start),
                                            true,
                                            window,
                                            cx,
                                        )
                                    });
                                }
                            });
                        }
                    })
                    .items(
                        &self.list_state,
                        self.todos.len(),
                        move |_index| "list".to_string(),
                        move |ix, _item_id, cx| {
                            let todo = &self.todos[ix];
                            div()
                                .w_full()
                                .px_2()
                                .py_1()
                                .hover(|style| style.bg(cx.theme().colors().highlight_primary))
                                .cursor_pointer()
                                .child(
                                    v_flex()
                                        .gap_0p5()
                                        .child(todo.content.to_string())
                                        .child(
                                            div()
                                                .text_color(cx.theme().colors().text_muted)
                                                .text_xs()
                                                .child(format!("{}:{}", todo.path.path.display(), todo.range.start.row + 1))
                                        )
                                )
                                .into_any_element()
                        },
                    ),
            )
    }
}
