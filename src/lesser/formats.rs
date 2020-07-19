#[derive(Debug)]
pub(crate) enum Message {
    Empty,
    ScrollDownPage,
    ScrollDown,
    ScrollUpPage,
    ScrollUp,
    ScrollLeftPage,
    ScrollLeft,
    ScrollRightPage,
    ScrollRight,
    ScrollToBeginning,
    ScrollToEnd,
    Exit,
    Reload,
}
