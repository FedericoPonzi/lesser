#[derive(Debug)]
pub(crate) enum Message {
    ScrollDownPage,
    ScrollDown,
    ScrollUpPage,
    ScrollUp,
    ScrollLeftPage,
    ScrollLeft,
    ScrollRightPage,
    ScrollRight,
    Exit,
    Reload,
}
