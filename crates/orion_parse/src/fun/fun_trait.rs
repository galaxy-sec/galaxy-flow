use winnow::ModalResult;

pub trait Fun0Builder {
    fn fun_name() -> &'static str;
    fn build() -> Self;
}

pub trait Fun1Builder {
    type ARG1;
    fn args1(data: &mut &str) -> ModalResult<Self::ARG1>;
    fn fun_name() -> &'static str;
    fn build(args: Self::ARG1) -> Self;
}

pub trait Fun2Builder {
    type ARG1;
    type ARG2;
    fn args1(data: &mut &str) -> ModalResult<Self::ARG1>;
    fn args2(data: &mut &str) -> ModalResult<Self::ARG2>;
    fn fun_name() -> &'static str;
    fn build(args: (Self::ARG1, Self::ARG2)) -> Self;
}

pub trait WnTake<T> {
    fn parse_next(input: &mut &str) -> ModalResult<T>;
}
