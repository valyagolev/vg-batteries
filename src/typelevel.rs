pub struct _Condition<const B: bool, T, F>(std::marker::PhantomData<(T, F)>);

pub trait TruthType {
    type ValueType;
}

impl<T, F> TruthType for _Condition<true, T, F> {
    type ValueType = T;
}

impl<T, F> TruthType for _Condition<false, T, F> {
    type ValueType = F;
}

/*
pub struct CoolType<const yes: bool>
where
    _Condition<yes, i8, std::string::String>: TruthType,
{
    pub weird: Condition<yes, i8, String>,
}
*/

pub type Condition<const yes: bool, T, F> = <_Condition<yes, T, F> as TruthType>::ValueType;
