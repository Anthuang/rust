#![warn(clippy::declare_interior_mutable_const)]

use std::borrow::Cow;
use std::cell::Cell;
use std::fmt::Display;
use std::sync::atomic::AtomicUsize;
use std::sync::Once;

const ATOMIC: AtomicUsize = AtomicUsize::new(5); //~ ERROR interior mutable
const CELL: Cell<usize> = Cell::new(6); //~ ERROR interior mutable
const ATOMIC_TUPLE: ([AtomicUsize; 1], Vec<AtomicUsize>, u8) = ([ATOMIC], Vec::new(), 7);
//~^ ERROR interior mutable

macro_rules! declare_const {
    ($name:ident: $ty:ty = $e:expr) => {
        const $name: $ty = $e;
    };
}
declare_const!(_ONCE: Once = Once::new()); //~ ERROR interior mutable

// const ATOMIC_REF: &AtomicUsize = &AtomicUsize::new(7); // This will simply trigger E0492.

const INTEGER: u8 = 8;
const STRING: String = String::new();
const STR: &str = "012345";
const COW: Cow<str> = Cow::Borrowed("abcdef");
//^ note: a const item of Cow is used in the `postgres` package.

const NO_ANN: &dyn Display = &70;

static STATIC_TUPLE: (AtomicUsize, String) = (ATOMIC, STRING);
//^ there should be no lints on this line

#[allow(clippy::declare_interior_mutable_const)]
const ONCE_INIT: Once = Once::new();

// a constant whose type is a concrete type should be linted at the definition site.
trait ConcreteTypes {
    const ATOMIC: AtomicUsize; //~ ERROR interior mutable
    const INTEGER: u64;
    const STRING: String;
    declare_const!(ANOTHER_ATOMIC: AtomicUsize = Self::ATOMIC); //~ ERROR interior mutable
}

impl ConcreteTypes for u64 {
    const ATOMIC: AtomicUsize = AtomicUsize::new(9);
    const INTEGER: u64 = 10;
    const STRING: String = String::new();
}

// a helper trait used below
trait ConstDefault {
    const DEFAULT: Self;
}

// a constant whose type is a generic type should be linted at the implementation site.
trait GenericTypes<T, U> {
    const TO_REMAIN_GENERIC: T;
    const TO_BE_CONCRETE: U;

    const HAVING_DEFAULT: T = Self::TO_REMAIN_GENERIC;
    declare_const!(IN_MACRO: T = Self::TO_REMAIN_GENERIC);
}

impl<T: ConstDefault> GenericTypes<T, AtomicUsize> for u64 {
    const TO_REMAIN_GENERIC: T = T::DEFAULT;
    const TO_BE_CONCRETE: AtomicUsize = AtomicUsize::new(11); //~ ERROR interior mutable
}

// a helper type used below
struct Wrapper<T>(T);

// a constant whose type is an associated type should be linted at the implementation site, too.
trait AssocTypes {
    type ToBeFrozen;
    type ToBeUnfrozen;
    type ToBeGenericParam;

    const TO_BE_FROZEN: Self::ToBeFrozen;
    const TO_BE_UNFROZEN: Self::ToBeUnfrozen;
    const WRAPPED_TO_BE_UNFROZEN: Wrapper<Self::ToBeUnfrozen>;
    // to ensure it can handle things when a generic type remains after normalization.
    const WRAPPED_TO_BE_GENERIC_PARAM: Wrapper<Self::ToBeGenericParam>;
}

impl<T: ConstDefault> AssocTypes for Vec<T> {
    type ToBeFrozen = u16;
    type ToBeUnfrozen = AtomicUsize;
    type ToBeGenericParam = T;

    const TO_BE_FROZEN: Self::ToBeFrozen = 12;
    const TO_BE_UNFROZEN: Self::ToBeUnfrozen = AtomicUsize::new(13); //~ ERROR interior mutable
    const WRAPPED_TO_BE_UNFROZEN: Wrapper<Self::ToBeUnfrozen> = Wrapper(AtomicUsize::new(14)); //~ ERROR interior mutable
    const WRAPPED_TO_BE_GENERIC_PARAM: Wrapper<Self::ToBeGenericParam> = Wrapper(T::DEFAULT);
}

// a helper trait used below
trait AssocTypesHelper {
    type NotToBeBounded;
    type ToBeBounded;

    const NOT_TO_BE_BOUNDED: Self::NotToBeBounded;
}

// a constant whose type is an assoc type originated from a generic param bounded at the definition
// site should be linted at there.
trait AssocTypesFromGenericParam<T>
where
    T: AssocTypesHelper<ToBeBounded = AtomicUsize>,
{
    const NOT_BOUNDED: T::NotToBeBounded;
    const BOUNDED: T::ToBeBounded; //~ ERROR interior mutable
}

impl<T> AssocTypesFromGenericParam<T> for u64
where
    T: AssocTypesHelper<ToBeBounded = AtomicUsize>,
{
    // an associated type could remain unknown in a trait impl.
    const NOT_BOUNDED: T::NotToBeBounded = T::NOT_TO_BE_BOUNDED;
    const BOUNDED: T::ToBeBounded = AtomicUsize::new(15);
}

trait SelfType {
    const SELF: Self;
}

impl SelfType for u64 {
    const SELF: Self = 16;
}

impl SelfType for AtomicUsize {
    // this (interior mutable `Self` const) exists in `parking_lot`.
    // `const_trait_impl` will replace it in the future, hopefully.
    const SELF: Self = AtomicUsize::new(17); //~ ERROR interior mutable
}

// Even though a constant contains a generic type, if it also have a interior mutable type,
// it should be linted at the definition site.
trait BothOfCellAndGeneric<T> {
    // this is a false negative in the current implementation.
    const DIRECT: Cell<T>;
    const INDIRECT: Cell<*const T>; //~ ERROR interior mutable
}

impl<T: ConstDefault> BothOfCellAndGeneric<T> for u64 {
    const DIRECT: Cell<T> = Cell::new(T::DEFAULT);
    const INDIRECT: Cell<*const T> = Cell::new(std::ptr::null());
}

struct Local<T>(T);

// a constant in an inherent impl are essentially the same as a normal const item
// except there can be a generic or associated type.
impl<T> Local<T>
where
    T: ConstDefault + AssocTypesHelper<ToBeBounded = AtomicUsize>,
{
    const ATOMIC: AtomicUsize = AtomicUsize::new(18); //~ ERROR interior mutable
    const COW: Cow<'static, str> = Cow::Borrowed("tuvwxy");

    const GENERIC_TYPE: T = T::DEFAULT;

    const ASSOC_TYPE: T::NotToBeBounded = T::NOT_TO_BE_BOUNDED;
    const BOUNDED_ASSOC_TYPE: T::ToBeBounded = AtomicUsize::new(19); //~ ERROR interior mutable
}

fn main() {}
