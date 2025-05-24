use crate::Editor;
use anyhow::Result;
use gpui::{
    actions, div, h_flex, list, px, v_flex, AppContext, Command, DismissEvent, div, prelude::*, 
    ListState, ListView, WindowContext, SharedString, Button,
};
use project::{Project, TodoEntry, TodoKind}; 
use ui::{prelude::*, ListHeader, ListItem};
use workspace::{Panel, Workspace, panel, toolbar::ToolbarItemLocation};
use regex::Regex;
use log::{debug, info};

actions!(todos, [ShowTodos, ToggleFocus, ToggleTodosPanel]);

pub struct ToggleTodosPanel;

impl Command for ToggleTodosPanel {}

impl Editor {
    pub fn init(cx: &mut AppContext) {
        debug!("Registering ToggleTodosPanel command and toolbar item");
        cx.register_action(ToggleTodosPanel::default());
        cx.bind_command(ToggleTodosPanel, Self::handle_toggle_todos);

        // Register toolbar button
        workspace::register_toolbar_item(
            ToolbarItemLocation::End,
            |_workspace, cx| {
                Button::new(
                    cx,
                    "Show TODOs",
                    IconName::Check,
                    ToggleTodosPanel::default(),
                )
            },
        );
    }

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

    pub fn handle_toggle_todos(&mut self, _: &ToggleTodosPanel, cx: &mut WindowContext) {
        debug!("ToggleTodosPanel command triggered");
        if let Some(workspace) = self.workspace() {
            workspace.update(cx, |workspace, cx| {
                if workspace.has_panel::<TodosPanel>(cx) {
                    debug!("Toggling existing TodosPanel");
                    workspace.toggle_panel_focus::<TodosPanel>(cx);
                } else {
                    debug!("Creating new TodosPanel");
                    self.show_todos(&ShowTodos, cx);
                }
            });
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

    fn todo_count(&self) -> (usize, usize) {
        let mut todos = 0;
        let mut fixmes = 0;
        for todo in &self.todos {
            match todo.kind {
                TodoKind::Todo => todos += 1,
                TodoKind::Fixme => fixmes += 1,
            }
        }
        (todos, fixmes)
    }
}

impl Panel for TodosPanel {
    fn persistent_name() -> &'static str {
        "TODOs"
    }

    fn icon(&self) -> Option<IconName> {
        Some(IconName::Check)
    }

    fn icon_tooltip(&self, _: &Window, _: &App) -> Option<&'static str> {
        Some("TODOs and FIXMEs")
    }

    fn toggle_action() -> Box<dyn gpui::Action> {
        ToggleFocus.boxed_clone()
    }

    fn position(&self) -> panel::Position {
        panel::Position::Right
    }

    fn render(&mut self, cx: &mut WindowContext) -> impl IntoElement {
        let (todo_count, fixme_count) = self.todo_count();
        
        v_flex()
            .size_full()
            .gap_2()
            .p_2()
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_1()
                            .child("TODOs")
                            .text_color(cx.theme().colors().text)
                            .child(
                                div()
                                    .text_color(cx.theme().colors().text_muted)
                                    .child(format!("{todo_count}"))
                            )
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child("FIXMEs")
                            .text_color(cx.theme().colors().text)
                            .child(
                                div()
                                    .text_color(cx.theme().colors().text_muted)
                                    .child(format!("{fixme_count}"))
                            )
                    )
            )
            .child(
                ListView::new()
                    .size_full()
                    .track_scroll()
                    .track_visible_items()
                    .on_click(move |event, cx| {
                        if let Some(todo) = self.todos.get(event.item.index) {
                            cx.window_context().update(|cx| {
                                if let Some(workspace) = workspace::Workspace::find_focused(cx) {
                                    workspace.update(cx, |workspace, window, cx| {
                                        workspace.open_path(
                                            todo.file_path.clone(),
                                            Some(language::Anchor::position(
                                                language::Point::new(todo.line - 1, todo.start_column),
                                            )),
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
                            let icon = match todo.kind {
                                TodoKind::Todo => IconName::Check,
                                TodoKind::Fixme => IconName::Alert,
                            };
                            
                            ListItem::new(ix)
                                .inset_0()
                                .spacing_2()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(Icon::new(icon))
                                        .child(
                                            v_flex()
                                                .gap_0p5()
                                                .child(todo.message.clone())
                                                .child(
                                                    div()
                                                        .text_color(cx.theme().colors().text_muted)
                                                        .text_xs()
                                                        .child(format!(
                                                            "{}:{}",
                                                            todo.file_path.path.display(),
                                                            todo.line
                                                        ))
                                                )
                                        )
                                )
                                .into_any_element()
                        },
                    )
            )
    }
}

pub struct TodoScanner {
    todo_pattern: Regex,
    block_todo_pattern: Regex,
}

impl TodoScanner {
    pub fn new() -> Self {
        Self {
            todo_pattern: Regex::new(r"(?i)//\s*(?:TODO|FIXME)(?::|=|\s)\s*(.+)").unwrap(),
            block_todo_pattern: Regex::new(r"(?i)/\*+\s*(?:TODO|FIXME)(?::|=|\s)\s*([^*]+)\*/").unwrap(),
        }
    }

    pub fn scan_todos(&self, text: &str) -> Vec<TodoLocation> {
        let mut todos = Vec::new();
        
        for (line_num, line) in text.lines().enumerate() {
            // Check for single line todos
            if let Some(cap) = self.todo_pattern.captures(line) {
                if let Some(message) = cap.get(1) {
                    let kind = if line.to_lowercase().contains("fixme") {
                        TodoKind::Fixme
                    } else {
                        TodoKind::Todo 
                    };
                    
                    todos.push(TodoLocation {
                        line: line_num + 1,
                        kind,
                        message: message.as_str().trim().to_string(),
                        start: message.start(),
                        end: message.end()
                    });
                }
            }

            // Check for block todos
            if let Some(cap) = self.block_todo_pattern.captures(line) {
                if let Some(message) = cap.get(1) {
                    let kind = if line.to_lowercase().contains("fixme") {
                        TodoKind::Fixme
                    } else {
                        TodoKind::Todo
                    };

                    todos.push(TodoLocation {
                        line: line_num + 1, 
                        kind,
                        message: message.as_str().trim().to_string(),
                        start: message.start(),
                        end: message.end()
                    });
                }
            }
        }

        todos
    }
}

#[derive(Debug, Clone)]
pub struct TodoLocation {
    pub line: usize,
    pub kind: TodoKind,
    pub message: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodoKind {
    Todo,
    Fixme
}