use gpui::*;
use project::{Project, TodoEntry};
use workspace::{Workspace, Panel};

actions!(search_todos, [OpenTodosPanel]);

pub struct TodoPanel {
    todos: Vec<TodoEntry>,
    focus_handle: FocusHandle,
}

impl TodoPanel {
    pub fn new(cx: &mut WindowContext) -> Self {
        let todos = cx
            .workspace()
            .project()
            .read(cx)
            .search_todos(cx)
            .unwrap_or_default();
        Self {
            todos,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Panel for TodoPanel {
    fn persistent_name() -> &'static str {
        "TODOs"
    }
    fn icon(&self) -> Option<IconName> {
        Some(IconName::Check)
    }
    // ...implement other required Panel methods as needed...
}

impl FocusableView for TodoPanel {
    fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TodoPanel {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl Element {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_1()
            .px_2()
            .child(
                h_flex()
                    .w_full()
                    .py_2()
                    .child(StyledText::new(format!("TODOs ({})", self.todos.len())).heading())
            )
            .child(
                list::build(ListState::new(
                    &self.todos,
                    cx,
                    |todo, | ListItem::new(format!("todo-{}", todo.line))
                        .inset_0()
                        .py_1()
                        .child(StyledText::new(todo.text.clone())),
                ))
            )
    }
}

pub fn register(cx: &mut App) {
    cx.registeraction(|workspace: &mut Workspace, : &OpenTodosPanel, window, cx| {
        let panel = cx.new(|cx| TodoPanel::new(cx));
        workspace.add_panel(panel, window, cx);
    });
}